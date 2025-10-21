use super::my_sql_repository::MySqlRepository;
use crate::{common::AppError, dto::delivery_staff::DeliveryStaffDTO, map_db_err};
use anyhow::Result;
use async_trait::async_trait;
use tracing::{debug, error};

#[async_trait]
pub trait DeliveryStaffRepository: Send + Sync {
    async fn get_delivery_staff_by_provider(
        &self,
        provider_hash: &str,
    ) -> Result<Vec<DeliveryStaffDTO>, AppError>;

    async fn create_delivery_staff(
        &self,
        provider_hash: &str,
        delivery_staff: &DeliveryStaffDTO,
    ) -> Result<DeliveryStaffDTO, AppError>;

    async fn is_id_card_exists(&self, id_card: &str) -> Result<bool, AppError>;

    async fn disable_or_enable_delivery_staff(
        &self,
        provider_hash: &str,
        id_card: &str,
    ) -> Result<(), AppError>;
}

impl MySqlRepository {
    async fn get_provider_id_by_hash(&self, provider_hash: &str) -> Result<i32, AppError> {
        let record = sqlx::query!(
            r#"SELECT id FROM tenants 
            WHERE name_hash = ? 
            AND deleted_at IS NULL 
            AND status = 'ACTIVE'"#,
            provider_hash
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get provider by hash"))?;

        debug!("Provider ID: {}", record.id);
        if record.id == 0 {
            return Err(AppError::NotFound("Provider not found".to_string()));
        }

        Ok(record.id)
    }
}

#[async_trait]
impl DeliveryStaffRepository for MySqlRepository {
    async fn get_delivery_staff_by_provider(
        &self,
        provider_hash: &str,
    ) -> Result<Vec<DeliveryStaffDTO>, AppError> {
        let provider_id = self.get_provider_id_by_hash(provider_hash).await?;

        let delivery_staff = sqlx::query_as!(
            DeliveryStaffDTO,
            r#"SELECT name, phone, id_card, created_by, status, remark, created_at FROM delivery_staff WHERE provider_id = ?"#,
            provider_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get delivery staff by provider hash"))?;

        Ok(delivery_staff)
    }

    async fn create_delivery_staff(
        &self,
        provider_hash: &str,
        delivery_staff: &DeliveryStaffDTO,
    ) -> Result<DeliveryStaffDTO, AppError> {
        let provider_id = self.get_provider_id_by_hash(provider_hash).await?;

        let result = sqlx::query!(
            "INSERT INTO delivery_staff (name, phone, id_card, created_by, remark, provider_id) VALUES (?, ?, ?, ?, ?, ?)",
            delivery_staff.name, delivery_staff.phone, delivery_staff.id_card, delivery_staff.created_by, delivery_staff.remark, provider_id)
        .execute(&self.pool)
        .await
        .map_err(map_db_err!("Failed to create delivery staff"))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Delivery staff not created".to_string()));
        }

        let delivery_staff_id = result.last_insert_id();

        let delivery_staff = sqlx::query_as!(
            DeliveryStaffDTO,
            r#"SELECT name, phone, id_card, created_by, remark, created_at, status FROM delivery_staff WHERE id = ?"#,
            delivery_staff_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get delivery staff by id"))?;

        Ok(delivery_staff)
    }

    async fn disable_or_enable_delivery_staff(
        &self,
        provider_hash: &str,
        id_card: &str,
    ) -> Result<(), AppError> {
        let provider_id = self.get_provider_id_by_hash(provider_hash).await?;
        debug!("Provider ---ID: {}", provider_id);

        let result = sqlx::query!(
            "UPDATE delivery_staff SET status = 1 - status WHERE id_card = ? AND provider_id = ?",
            id_card,
            provider_id
        )
        .execute(&self.pool)
        .await
        .map_err(map_db_err!("Failed to disable or enable delivery staff"))?;

        debug!("Delivery staff updated: {}", result.rows_affected());

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("No delivery staff updated".to_string()));
        }

        Ok(())
    }

    async fn is_id_card_exists(&self, id_card: &str) -> Result<bool, AppError> {
        let exists = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM delivery_staff WHERE id_card = ?)"#,
            id_card
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_err!("Failed to check if id card exists"))?;

        Ok(exists == 1)
    }
}
