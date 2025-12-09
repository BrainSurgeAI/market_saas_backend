use crate::repositories::my_sql_repository::MySqlRepository;
use crate::{
    common::AppError,
    map_db_err,
    models::{
        order_action::OrderAction, order_machine::OrderStateMachine, order_status::OrderStatus,
        tenant_type::TenantType,
    },
};
use async_trait::async_trait;
use tracing::{debug, error};

#[async_trait]
pub(crate) trait MarketplaceOrderRepository: Send + Sync {
    /// Marketplace assign order to provider
    ///
    /// # Arguments
    /// * `order_code` - The code of the order
    /// * `provider_id` - The id of the provider
    /// * `assign_username` - The username of the user who assigned the order
    /// * `assign_by` - The name of the user who assigned the order
    ///
    /// # Returns
    /// A result containing the error if the order is not found or the order is expired
    /// or the order is not in the correct status to be assigned to a provider
    async fn assign_order(
        &self,
        order_code: &str,
        provider_id: i32,
        assign_username: &str,
        assign_by: &str,
    ) -> Result<OrderStatus, AppError>;

  
    // async fn get_market_dashboard_stats(
    //     &self,
    //     tenant_hash: &str,
    // ) -> Result<MarketOrderStatisticsResponse, AppError>;
}

#[async_trait]
impl MarketplaceOrderRepository for MySqlRepository {
    async fn assign_order(
        &self,
        order_code: &str,
        provider_id: i32,
        assign_username: &str,
        assign_by: &str,
    ) -> Result<OrderStatus, AppError> {
        use chrono::Local;

        // TODO: check if the provider is active and is belongs with the tenant_relationships table
        //       and check if the self(market) is belongs with the orders market_id
        
        // begin transaction
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Failed to begin transaction to assign order"))?;

        let order_opt = sqlx::query!(
            r#"
            SELECT id, delivery_date, order_status
            FROM orders
            WHERE order_code = ?
            FOR UPDATE
            "#,
            order_code
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to lock order for update"))?;

        let order =
            order_opt.ok_or_else(|| AppError::NotFound(format!("订单 {} 不存在", order_code)))?;

        if order.delivery_date < Local::now().date_naive() {
            return Err(AppError::Validation(format!(
                "不能指派已过期订单 {}",
                order_code
            )));
        }

        let next = OrderStateMachine::next_state(
            OrderStatus::try_from(order.order_status.as_str())?,
            OrderAction::AssignSupplier,
            TenantType::Market,
        )
        .map_err(|e| {
            error!("状态流转错误: {}", e);
            AppError::Validation("Invalid transition".to_string())
        })?;

        debug!("Next status: {:?}", next);

        // Assign order to provider
        sqlx::query!(
            r#"INSERT INTO provider_orders_assignments (order_id, provider_id)
                       VALUES (?, ?)"#,
            order.id,
            provider_id
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to insert provider orders assignments"))?;

        // update order status to assigned, confirmed_at, confirmed_by, market_contact_number
        let affected = sqlx::query!(
            r#"
                UPDATE orders AS o
                JOIN users AS u ON u.username = ?
                SET 
                    o.order_status = ?,
                    o.assigned_by = ?,
                    o.market_contact_number = u.phone
                WHERE 
                    o.id = ?
                AND o.order_status = ?
            "#,
            assign_username, // for JOIN
            next.to_str(),
            assign_by, // confirmed_by field
            order.id,
            order.order_status.as_str()
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to update order status"))?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::Validation(format!(
                "订单 {} 状态已更新或被取消",
                order_code
            )));
        }

        // 根据订单id查询provider_orders_assignments表的id,并根据id查询该tenant对应的所有用户的id
        let user_ids = sqlx::query!(
            r#"
            SELECT u.id
            FROM users u
            INNER JOIN provider_orders_assignments poa ON u.tenant_id = poa.provider_id
            WHERE poa.order_id = ?
            "#,
            order.id
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to get user ids"))?
        .into_iter()
        .map(|row| row.id)
        .collect::<Vec<i32>>();

        // 批量插入messages表，category为'新订单',title为'新订单分配', content为'您有新的订单{order_code}需要处理'
        if !user_ids.is_empty() {
            let content = format!("您有新的订单 {} 需要处理", order_code);
            let mut query_builder = sqlx::QueryBuilder::new(
                "INSERT INTO messages (user_id, category, title, content) "
            );
            
            query_builder.push_values(user_ids.iter(), |mut b, user_id| {
                b.push_bind(user_id)
                    .push_bind("新订单")
                    .push_bind("新订单分配")
                    .push_bind(&content);
            });
            
            query_builder
                .build()
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to batch insert messages"))?;
        }

        // insert order status history
        self.insert_order_status_history(
            &mut tx,
            order.id,
            order.order_status.as_str(),
            next,
            assign_by,
            OrderAction::AssignSupplier,
        )
        .await?;

        tx.commit()
            .await
            .map_err(map_db_err!("Failed to commit transaction"))?;

        Ok(next)
    }

    // async fn get_market_dashboard_stats(
    //     &self,
    //     tenant_hash: &str,
    // ) -> Result<MarketOrderStatisticsResponse, AppError> {
    //     use crate::dto::order::{
    //         Metadata, PendingAssignmentMetadata, TrendItem, Trends,
    //     };

    //     // Get market tenant id
    //     let market_id = self
    //         .get_tenant_id_by_tenant_hash_and_tenant_type(tenant_hash, "MARKET")
    //         .await?;

    //     let today = Local::now().date_naive();
    //     let yesterday = today
    //         .pred_opt()
    //         .ok_or_else(|| AppError::Internal("无法计算昨天的日期".to_string()))?;

    //     // Query all order statistics in one query
    //     let order_stats = sqlx::query!(
    //         r#"
    //         SELECT 
    //             COUNT(CASE WHEN DATE(created_at) = ? THEN 1 END) as new_orders_today,
    //             COUNT(CASE WHEN DATE(created_at) = ? AND order_status = 'PENDING' THEN 1 END) as new_orders_pending_today,
    //             COUNT(CASE WHEN DATE(created_at) = ? THEN 1 END) as yesterday_total_orders,
    //             COUNT(CASE WHEN DATE(created_at) = ? AND (order_status = 'EXCHANGE_DELIVERING' OR order_status = 'SUPPLIER_DELIVERING') THEN 1 END) as pending_inspection,
    //             COUNT(CASE WHEN DATE(created_at) = ? AND order_status = 'COMPLETED' THEN 1 END) as completed_today,
    //             COUNT(CASE WHEN DATE(created_at) = ? AND order_status = 'COMPLETED' THEN 1 END) as completed_yesterday
    //         FROM orders
    //         WHERE market_id = ?
    //         AND DATE(created_at) IN (?, ?)
    //         AND deleted_at IS NULL
    //         "#,
    //         today,
    //         today,
    //         yesterday,
    //         today,
    //         today,
    //         yesterday,
    //         market_id,
    //         today,
    //         yesterday
    //     )
    //     .fetch_one(&self.pool)
    //     .await
    //     .map_err(map_db_err!("Failed to get order statistics"))?;

    //     let new_orders_today = order_stats.new_orders_today;
    //     let yesterday_total_orders = order_stats.yesterday_total_orders;
    //     let pending_assignment = order_stats.new_orders_pending_today; // Same as new_orders_today (both are PENDING status)
    //     let pending_inspection = order_stats.pending_inspection;
    //     let completed_today = order_stats.completed_today;
    //     let completed_yesterday = order_stats.completed_yesterday;

    //     // Calculate newOrders trend
    //     let new_orders_trend = if yesterday_total_orders > 0 {
    //         let diff = new_orders_today as f64 - yesterday_total_orders as f64;
    //         let percentage = (diff / yesterday_total_orders as f64) * 100.0;
    //         let value = if percentage >= 0.0 {
    //             format!("+{:.1}%", percentage)
    //         } else {
    //             format!("{:.1}%", percentage)
    //         };
    //         Some(TrendItem {
    //             value,
    //             up: percentage >= 0.0,
    //         })
    //     } else {
    //         Some(TrendItem {
    //             value: format!("+{}", new_orders_today),
    //             up: true,
    //         })
    //     };

    //     // Query today's and yesterday's exceptions in one query
    //     let exceptions_stats = sqlx::query!(
    //         r#"
    //         SELECT 
    //             COUNT(DISTINCT CASE WHEN DATE(rer.created_at) = ? THEN o.id END) as exceptions_today,
    //             COUNT(DISTINCT CASE WHEN DATE(rer.created_at) = ? THEN o.id END) as exceptions_yesterday
    //         FROM return_exchange_records rer
    //         INNER JOIN order_details od ON rer.order_detail_id = od.id
    //         INNER JOIN orders o ON od.order_id = o.id
    //         WHERE o.market_id = ?
    //         AND DATE(rer.created_at) IN (?, ?)
    //         AND o.deleted_at IS NULL
    //         "#,
    //         today,
    //         yesterday,
    //         market_id,
    //         today,
    //         yesterday
    //     )
    //     .fetch_one(&self.pool)
    //     .await
    //     .map_err(map_db_err!("Failed to get exceptions statistics"))?;

    //     let exceptions_today = exceptions_stats.exceptions_today;
    //     let exceptions_yesterday = exceptions_stats.exceptions_yesterday;

    //     // Calculate exceptions trend
    //     let exceptions_trend = if exceptions_yesterday > 0 {
    //         let diff = exceptions_today as f64 - exceptions_yesterday as f64;
    //         let percentage = (diff / exceptions_yesterday as f64) * 100.0;
    //         let value = if percentage >= 0.0 {
    //             format!("+{:.1}%", percentage)
    //         } else {
    //             format!("{:.1}%", percentage)
    //         };
    //         Some(TrendItem {
    //             value,
    //             up: percentage >= 0.0,
    //         })
    //     } else if exceptions_today > 0 {
    //         Some(TrendItem {
    //             value: format!("+{}", exceptions_today),
    //             up: true,
    //         })
    //     } else {
    //         let diff = exceptions_today - exceptions_yesterday;
    //         Some(TrendItem {
    //             value: if diff >= 0 {
    //                 format!("+{}", diff)
    //             } else {
    //                 format!("{}", diff)
    //             },
    //             up: diff >= 0,
    //         })
    //     };


    //     // Calculate completed trend
    //     let completed_trend = if completed_yesterday > 0 {
    //         let diff = completed_today as f64 - completed_yesterday as f64;
    //         let percentage = (diff / completed_yesterday as f64) * 100.0;
    //         let value = if percentage >= 0.0 {
    //             format!("+{:.1}%", percentage)
    //         } else {
    //             format!("{:.1}%", percentage)
    //         };
    //         Some(TrendItem {
    //             value,
    //             up: percentage >= 0.0,
    //         })
    //     } else if completed_today > 0 {
    //         Some(TrendItem {
    //             value: format!("+{}", completed_today),
    //             up: true,
    //         })
    //     } else {
    //         Some(TrendItem {
    //             value: "+0%".to_string(),
    //             up: true,
    //         })
    //     };

    //     Ok(MarketOrderStatisticsResponse {
    //         new_orders: new_orders_today,
    //         pending_assignment,
    //         pending_inspection,
    //         exceptions: exceptions_today,
    //         completed: completed_today,
    //         trends: Trends {
    //             new_orders: new_orders_trend,
    //             exceptions: exceptions_trend,
    //             completed: completed_trend,
    //         },
    //         metadata: Metadata {
    //             pending_assignment: PendingAssignmentMetadata {
    //                 subtitle: "急需处理".to_string(),
    //                 active: pending_assignment > 0,
    //             },
    //         },
    //     })
    // }
}
