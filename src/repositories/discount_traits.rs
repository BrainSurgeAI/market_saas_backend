use crate::{
    common::AppError,
    dto::discount::{CustomerDiscountCreateRequestDTO, CustomerDiscountResponseDTO},
    map_db_err,
};

use super::my_sql_repository::MySqlRepository;
use async_trait::async_trait;

use rust_decimal::Decimal;
use sqlx::{MySql, QueryBuilder};
use tracing::debug;

#[async_trait]
pub trait DiscountRepository: Send + Sync {
    async fn fetch_customer_discount_list(
        &self,
        tenant_hash: &str,
    ) -> Result<Vec<CustomerDiscountResponseDTO>, AppError>;

    async fn update_discount_by_tenant_and_category(
        &self,
        tenant_hash: &str,
        discount_id: i64,
        discount_rate: &Decimal,
        changed_by: &str,
    ) -> Result<(), AppError>;

    async fn create_discount_by_tenant_and_category(
        &self,
        tenant_hash: &str,
        discount_create_request: &CustomerDiscountCreateRequestDTO,
    ) -> Result<u64, AppError>;
}

#[async_trait]
impl DiscountRepository for MySqlRepository {
    async fn fetch_customer_discount_list(
        &self,
        tenant_hash: &str,
    ) -> Result<Vec<CustomerDiscountResponseDTO>, AppError> {
        let sql = r#"SELECT c.id AS category_id, c.name AS category_name, 
                           ccd.id AS discount_id, ccd.discount_rate, ccd.start_date, ccd.end_date, ccd.status,
                           ccd.created_at, ccd.updated_at FROM customer_category_discounts ccd
                           INNER JOIN categories c ON ccd.category_id = c.id 
                           INNER JOIN tenants t ON t.id = ccd.tenant_id
                           WHERE t.name_hash = ? AND ccd.status = 1 AND (ccd.end_date <= '9999-12-31' OR ccd.end_date >= CURDATE()) 
                           ORDER BY c.sort_order DESC;"#;

        let discounts = sqlx::query_as::<_, CustomerDiscountResponseDTO>(sql)
            .bind(tenant_hash)
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get customer discount list"))?;
        Ok(discounts)
    }

    async fn update_discount_by_tenant_and_category(
        &self,
        _tenant_hash: &str,
        discount_id: i64,
        discount_rate: &Decimal,
        changed_by: &str,
    ) -> Result<(), AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Failed to start transaction"))?;

        let record = sqlx::query!(
            "SELECT tenant_id, discount_rate, category_id FROM customer_category_discounts WHERE id = ?",
            discount_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to get old discount rate"))?;

        if record.is_none() {
            return Err(AppError::NotFound("Discount not found".to_string()));
        }

        let old_discount_rate = record.unwrap();

        let sql =
            r#"UPDATE customer_category_discounts ccd SET ccd.discount_rate = ? WHERE ccd.id = ?;"#;
        sqlx::query(sql)
            .bind(discount_rate)
            .bind(discount_id)
            .execute(&mut *tx)
            .await
            .map_err(map_db_err!(
                "Failed to update discount by tenant and category"
            ))?;

        // Save to history
        sqlx::query!(
            "INSERT INTO customer_category_discount_history (tenant_id, category_id, new_discount_rate, old_discount_rate, change_type, change_reason, changed_by) VALUES (?, ?, ?, ?, ?, ?, ?)",
            old_discount_rate.tenant_id,
            old_discount_rate.category_id,
            &discount_rate,
            &old_discount_rate.discount_rate,
            "UPDATE",
            "Manual update",
            &changed_by
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to insert discount history record"))?;

        // 提交事务
        tx.commit()
            .await
            .map_err(map_db_err!("Failed to commit transaction"))?;

        Ok(())
    }

    async fn create_discount_by_tenant_and_category(
        &self,
        tenant_hash: &str,
        discount_create_request: &CustomerDiscountCreateRequestDTO,
    ) -> Result<u64, AppError> {
        debug!(
            "Creating discounts for tenant: {}, count: {}",
            tenant_hash,
            discount_create_request.discounts.len()
        );

        let tenant_id = sqlx::query!("SELECT id FROM tenants WHERE name_hash = ?", tenant_hash)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get tenant id by name hash"))?;

        if tenant_id.id == 0 {
            return Err(AppError::NotFound("Tenant not found".to_string()));
        }

        // 使用事务来确保原子性操作
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Failed to start transaction"))?;

        // 使用 QueryBuilder 构建批量插入语句
        let mut query_builder: QueryBuilder<MySql> = QueryBuilder::new(
            "INSERT INTO customer_category_discounts (tenant_id, category_id, discount_rate, start_date, end_date, created_by) "
        );

        query_builder.push_values(
            discount_create_request.discounts.iter(),
            |mut b, discount| {
                b.push_bind(tenant_id.id)
                    .push_bind(discount.category_id)
                    .push_bind(discount.discount_rate)
                    .push_bind(&discount.start_date)
                    .push_bind(&discount.end_date)
                    .push_bind(&discount_create_request.created_by);
            },
        );

        let query = query_builder.build();
        let result = query.execute(&mut *tx).await.map_err(map_db_err!(
            "Failed to create discount by tenant and category"
        ))?;

        // 查询刚插入的记录，以获取正确的ID
        let inserted_ids = sqlx::query!(
            "SELECT id, category_id, discount_rate, start_date, end_date 
             FROM customer_category_discounts 
             WHERE tenant_id = ? 
             ORDER BY id DESC 
             LIMIT ?",
            tenant_id.id,
            discount_create_request.discounts.len() as i32
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to fetch inserted discount ids"))?;

        if inserted_ids.is_empty() {
            return Err(AppError::Internal(
                "Failed to get inserted discount ids".to_string(),
            ));
        }

        // 插入历史记录
        let mut history_query_builder: QueryBuilder<MySql> = QueryBuilder::new(
            "INSERT INTO customer_category_discount_history (tenant_id, category_id, new_discount_rate, change_type, change_reason, changed_by) "
        );

        history_query_builder.push_values(inserted_ids.iter(), |mut b, record| {
            b.push_bind(tenant_id.id)
                .push_bind(record.category_id)
                .push_bind(record.discount_rate)
                .push_bind("CREATE")
                .push_bind("CREATE")
                .push_bind(&discount_create_request.created_by);
        });

        let history_query = history_query_builder.build();
        history_query
            .execute(&mut *tx)
            .await
            .map_err(map_db_err!("Failed to insert discount history records"))?;

        // 提交事务
        tx.commit()
            .await
            .map_err(map_db_err!("Failed to commit transaction"))?;

        Ok(result.rows_affected())
    }
}
