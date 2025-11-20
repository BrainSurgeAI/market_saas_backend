use crate::repositories::my_sql_repository::MySqlRepository;
use crate::{
    common::AppError,
    dto::order::{OrderReceipt, ReceiptOperationType},
    map_db_err,
    models::{
        claims::Claims, order_action::OrderAction, order_machine::OrderStateMachine,
        order_status::OrderStatus, tenant_type::TenantType,
    },
};
use async_trait::async_trait;
use rust_decimal::Decimal;
use tracing::{debug, error};

#[async_trait]
pub(crate) trait SharedMarketCustomerOrderRepository: Send + Sync {
    async fn insert_order_inspection_with_aftersales(
        &self,
        receipt: &OrderReceipt,
        operator: &str,
        transaction_id: &str,
        tenant_type: &str,
    ) -> Result<String, AppError>;

    async fn update_order_status(
        &self,
        order_code: &str,
        action: OrderAction,
        claims: &Claims,
    ) -> Result<OrderStatus, AppError>;

    async fn create_after_sales_request(
        &self,
        order_code: &str,
        claims: &Claims,
        action: OrderAction,
        tenant_type: TenantType,
    ) -> Result<OrderStatus, AppError>;

    async fn insert_order_inspection(
        &self,
        order_code: &str,
        claims: &Claims,
        action: OrderAction,
    ) -> Result<OrderStatus, AppError>;

    /// 根据订单的签收、退货和换货情况，计算应该返回的 OrderAction
    ///
    /// 逻辑：
    /// - 如果有一个商品是换货，返回 MarketExchange 或 CustomerExchange（根据租户类型）
    /// - 如果都是退货，返回 MarketReturn 或 CustomerReturn（根据租户类型）
    /// - 其他情况返回 MarketAccept（客户没有 Accept，返回 MarketAccept）
    async fn determine_order_action_by_inspection_result(
        &self,
        order_code: &str,
        claims: &Claims,
    ) -> Result<OrderStatus, AppError>;
}

#[async_trait]
impl SharedMarketCustomerOrderRepository for MySqlRepository {
    async fn insert_order_inspection_with_aftersales(
        &self,
        receipt: &OrderReceipt,
        operator: &str,
        transaction_id: &str,
        tenant_type: &str,
    ) -> Result<String, AppError> {
        debug!(
            "Insert order inspection with aftersales receipt: {:?}",
            receipt
        );
        // 处理凭证证据（如果有的话）
        let evidence_json = match &receipt.evidence_images {
            Some(evidence_images) if !evidence_images.is_empty() => {
                Some(serde_json::to_string(evidence_images).map_err(|e| {
                    AppError::Internal(format!("Failed to serialize evidence: {}", e))
                })?)
            }
            _ => None,
        };

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Failed to begin transaction"))?;

        // 验证订单存在且状态正确
        let record = sqlx::query!(r#"
            SELECT o.id, od.id as detail_id, o.order_status, oi.id as inspection_id, 
                pdi.actual_qty as actual_quantity, pdi.subtotal as total_amount, od.discounted_unit_price as actual_price
            FROM orders o 
            INNER JOIN order_details od ON o.id = od.order_id
            INNER JOIN order_inspections oi ON o.id = oi.order_id
            INNER JOIN provider_delivery_items pdi ON od.id = pdi.order_detail_id
            WHERE o.order_code = ? AND od.id = ? AND oi.inspected_by_type = ? ORDER BY oi.inspection_round DESC LIMIT 1 FOR UPDATE"#,
            receipt.order_code,
            receipt.order_detail_id,
            tenant_type
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!(
            "Failed to find order and order detail and inspection"
        ))?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "Order {} with order detail id {} not found",
                receipt.order_code, receipt.order_detail_id
            ))
        })?;

        let order_id = record.id;
        let detail_id = record.detail_id;
        let inspection_id = record.inspection_id;
        let actual_price = record.actual_price;

        match receipt.operation_type {
            ReceiptOperationType::Sign => {
                debug!("Process normal sign receipt: {:?}", receipt);

                // 允许部分签收，所以签收数量按前端传入的签收数量进行更新
                sqlx::query!(
                    r#"INSERT INTO order_inspection_items (inspection_id, order_detail_id, inspected_qty, remarks) VALUES (?, ?, ?, ?)"#,
                    inspection_id, detail_id, receipt.quantity, receipt.reason
                )
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to insert order inspection item"))?;
            }
            ReceiptOperationType::Return => {
                let operation_type_str = String::from(ReceiptOperationType::Return);

                // 退货只能全部退货，所以退货按实际发货量全部退货，不从前端拿退货数量
                let return_exchange_id = sqlx::query!(
                    r#"INSERT INTO return_exchange_records 
                      (order_detail_id, operation_type, quantity, reason, reason_description, created_by, evidence_images, inspection_id)
                      VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
                    detail_id, operation_type_str, record.actual_quantity, receipt.reason, receipt.reason, operator, evidence_json, inspection_id
                )
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to create return record"))?
                .last_insert_id();

                sqlx::query!(
                    r#"INSERT INTO refund_records (return_exchange_id, amount, method, transaction_id)
                      VALUES (?, ?, ?, ?)"#,
                    return_exchange_id, record.total_amount, "ORIGINAL", transaction_id
                )
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to create refund record"))?;
            }
            ReceiptOperationType::Exchange => {
                let ordered_qty_record = sqlx::query!(
                    r#"SELECT ordered_qty FROM order_details WHERE id = ?"#,
                    detail_id
                )
                .fetch_optional(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to get ordered quantity"))?
                .ok_or_else(|| {
                    AppError::NotFound(format!("Ordered quantity not found: {}", detail_id))
                })?;

                let exchange_quantity = ordered_qty_record.ordered_qty - receipt.quantity;

                // if the exchange quantity is greater than 0, create the exchange record
                if exchange_quantity >= Decimal::ZERO {
                    let operation_type_str = String::from(ReceiptOperationType::Exchange);
                    let return_exchange_id = sqlx::query!(r#"
                    INSERT INTO return_exchange_records
                    (order_detail_id, operation_type, quantity, reason, reason_description, created_by, evidence_images, inspection_id)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
                    detail_id, operation_type_str, exchange_quantity, receipt.reason, receipt.reason, operator, evidence_json, inspection_id
                  )
                  .execute(&mut *tx)
                  .await
                  .map_err(map_db_err!("Failed to create exchange record"))?
                  .last_insert_id();

                    sqlx::query!(r#"
                    INSERT INTO exchange_items(return_exchange_id, product_code, product_name, quantity, price, total_amount)
                    VALUES (?, ?, ?, ?, ?, ?)"#,
                    return_exchange_id, receipt.product_code, receipt.product_name, exchange_quantity, actual_price, actual_price * exchange_quantity
                  )
                  .execute(&mut *tx)
                  .await
                  .map_err(map_db_err!("Failed to create exchange item"))?;
                }

                sqlx::query!(
                    r#"INSERT INTO order_inspection_items (inspection_id, order_detail_id, inspected_qty) VALUES (?, ?, ?)"#,
                    inspection_id, detail_id,  receipt.quantity
                )
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to update order inspection item"))?;
            }
        }

        // 验证订单中所有商品是否已经验收完毕
        let inspections_result = sqlx::query!(
            r#"
            WITH latest_inspection AS (
    SELECT id, inspection_round, parent_id, inspected_by_type
    FROM order_inspections
    WHERE order_id = ?
    ORDER BY inspection_round DESC
    LIMIT 1
),
order_detail_count AS (
    SELECT COUNT(*) AS total_details
    FROM order_details
    WHERE order_id = ?
),
current_inspected_count AS (
    SELECT COALESCE(COUNT(*),0) AS inspected_count
    FROM order_inspection_items
    WHERE inspection_id = (SELECT id FROM latest_inspection)
),

current_rer_count AS (
    SELECT 
        COALESCE(CAST(SUM(CASE WHEN operation_type = 'RETURN' THEN 1 ELSE 0 END) AS SIGNED), 0) AS return_count,
        COALESCE(CAST(SUM(CASE WHEN operation_type = 'EXCHANGE' THEN 1 ELSE 0 END) AS SIGNED), 0) AS exchange_count
    FROM return_exchange_records
    WHERE inspection_id = (SELECT id FROM latest_inspection)
),
parent_rer_count AS (
    SELECT 
        CASE 
            WHEN (SELECT parent_id FROM latest_inspection) IS NULL 
            THEN 0
            ELSE (
                SELECT COUNT(*) 
                FROM return_exchange_records 
                WHERE inspection_id = (SELECT parent_id FROM latest_inspection)
            )
        END AS parent_rer
),
required_items AS (
    SELECT 
        CASE 
            WHEN (SELECT parent_id FROM latest_inspection) IS NULL 
            THEN (SELECT total_details FROM order_detail_count)    -- 市场验收
            ELSE (SELECT total_details FROM order_detail_count) 
               - (SELECT parent_rer FROM parent_rer_count)         -- 客户验收
        END AS required_count
)

SELECT
    (SELECT required_count FROM required_items) AS required_items,
    (SELECT inspected_count FROM current_inspected_count) AS inspected_items,
    (SELECT return_count FROM current_rer_count) AS current_return_items,
    (SELECT exchange_count FROM current_rer_count) AS current_exchange_items,
    (
        (SELECT inspected_count FROM current_inspected_count)
        + (SELECT return_count FROM current_rer_count)
        + (SELECT exchange_count FROM current_rer_count)
        = (SELECT required_count FROM required_items)
    ) AS is_all_completed;
            "#,
            order_id,
            order_id,
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to get inspection items"))?
        .ok_or_else(|| AppError::NotFound(format!("Inspection items not found: {}", inspection_id)))?;

        let mut inspect_status = "PENDING";
        if inspections_result.is_all_completed == Some(1) {
            debug!("All inspection items are completed");
            inspect_status = if inspections_result.current_exchange_items > Some(0) {
                "EXCHANGE"
            } else if inspections_result.current_return_items == Some(0) {
                "PASS"
            } else if inspections_result.current_return_items != inspections_result.required_items {
                "PARTIAL"
            } else {
                "REJECTED"
            };

            sqlx::query!(
                r#"UPDATE order_inspections SET inspection_result = ?, inspected_by_type = ? WHERE id = ?"#,
                inspect_status,
                tenant_type,
                inspection_id
            )
            .execute(&mut *tx)
            .await
            .map_err(map_db_err!("Failed to update order inspection status"))?;
        }

        tx.commit()
            .await
            .map_err(map_db_err!("Failed to commit transaction"))?;

        debug!("Inspection status: {}", inspect_status);

        Ok(inspect_status.to_string())
    }

    async fn update_order_status(
        &self,
        order_code: &str,
        action: OrderAction,
        claims: &Claims,
    ) -> Result<OrderStatus, AppError> {
        let tenant_type = TenantType::try_from(claims.tenant_type.as_str())?;
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Failed to begin transaction"))?;

        // Use SELECT FOR UPDATE to lock the record, prevent concurrent modification
        // TODO: check if the order is belongs with the tenant_relationships table
        let order = sqlx::query!(
            r#"SELECT id, order_status FROM orders WHERE order_code = ? FOR UPDATE"#,
            order_code
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to get order"))?
        .ok_or_else(|| AppError::NotFound(format!("Order not found: {}", order_code)))?;

        let current_status = OrderStatus::try_from(order.order_status.as_str())?;

        let next_status = OrderStateMachine::next_state(current_status, action, tenant_type)
            .map_err(|e| {
                error!("Order state transition error: {}", e);
                AppError::Validation(format!(
                    "Order status is not correct: {} -> {}",
                    order.order_status,
                    action.description()
                ))
            })?;

        debug!("Next status: {}", next_status);

        sqlx::query!(
            r#"UPDATE orders SET order_status = ? WHERE id = ?"#,
            next_status.to_str(),
            order.id
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to update order status"))?;

        self.insert_order_status_history(
            &mut tx,
            order.id,
            order.order_status.as_str(),
            next_status,
            claims.real_name.as_str(),
            action,
        )
        .await?;

        debug!("Order status updated to {}", next_status);

        tx.commit()
            .await
            .map_err(map_db_err!("Failed to commit transaction"))?;
        Ok(next_status)
    }

    async fn insert_order_inspection(
        &self,
        order_code: &str,
        claims: &Claims,
        action: OrderAction,
    ) -> Result<OrderStatus, AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Failed to begin transaction"))?;

        // TODO: check if the order is belongs with the tenant_relationships table by tenant_type
        let order = sqlx::query!(
            r#"SELECT id, order_status FROM orders WHERE order_code = ? FOR UPDATE"#,
            order_code
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to get order"))?
        .ok_or_else(|| AppError::NotFound(format!("Order not found: {}", order_code)))?;

        let user_record = sqlx::query!(
            r#"SELECT id FROM users WHERE username = ?"#,
            claims.username
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to get user"))?
        .ok_or_else(|| AppError::NotFound(format!("User not found: {}", claims.username)))?;

        // Before create a new order inspection record, check if order inspections table has a record with the same order_id
        // get the inspection_round from the order inspections table
        let inspection = sqlx::query!(
            r#"SELECT id, inspection_round FROM order_inspections WHERE order_id = ? ORDER BY inspection_round DESC LIMIT 1"#,
            order.id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to get inspection round"))?;

        let (parent_id, inspection_round) = if let Some(inspection_record) = inspection {
            (
                Some(inspection_record.id),
                inspection_record.inspection_round + 1,
            )
        } else {
            (None, 1)
        };
        // Create a new order inspection record
        sqlx::query!(
            r#"INSERT INTO order_inspections (order_id, inspected_by_type, inspected_by_id, inspection_round, parent_id) VALUES (?, ?, ?, ?, ?)"#,
            order.id, claims.tenant_type, user_record.id, inspection_round, parent_id
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to create order inspection record"))?;

        // Update the order status to MarketInspecting or CustomerInspecting
        let next_status = OrderStateMachine::next_state(
            OrderStatus::try_from(order.order_status.as_str())?,
            action,
            TenantType::try_from(claims.tenant_type.as_str())?,
        )
        .map_err(|e| {
            error!("Order state transition error: {}", e);
            AppError::Validation(format!(
                "Order status is not correct: {} -> {}",
                order.order_status,
                action.description()
            ))
        })?;

        sqlx::query!(
            r#"UPDATE orders SET order_status = ? WHERE id = ?"#,
            next_status.to_str(),
            order.id
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to update order status"))?;

        // insert order status history
        self.insert_order_status_history(
            &mut tx,
            order.id,
            order.order_status.as_str(),
            next_status,
            claims.real_name.as_str(),
            action,
        )
        .await?;

        debug!("Order status updated to {}", next_status);
        tx.commit()
            .await
            .map_err(map_db_err!("Failed to commit transaction"))?;
        Ok(next_status)
    }

    async fn determine_order_action_by_inspection_result(
        &self,
        order_code: &str,
        claims: &Claims,
    ) -> Result<OrderStatus, AppError> {
        let tenant_type = TenantType::try_from(claims.tenant_type.as_str())?;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Failed to begin transaction"))?;

        // TODO: check if the order is belongs with the tenant_relationships table by tenant_type
        let order = sqlx::query!(
            r#"SELECT o.id, o.order_status, oi.inspection_result
               FROM orders o
               INNER JOIN order_inspections oi ON o.id = oi.order_id
               WHERE o.order_code = ? AND oi.inspected_by_type = ? ORDER BY oi.inspection_round DESC LIMIT 1 FOR UPDATE"#,
            order_code,
            claims.tenant_type
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to get order"))?
        .ok_or_else(|| AppError::NotFound(format!("Order not found: {}", order_code)))?;

        let current_status = OrderStatus::try_from(order.order_status.as_str())?;

        let action = if order.inspection_result == "EXCHANGE" {
           if tenant_type == TenantType::Market {
            OrderAction::MarketExchange
           } else {
            OrderAction::CustomerExchange
           }
        } else if order.inspection_result == "PASS" || order.inspection_result == "PARTIAL" {
            if tenant_type == TenantType::Market {
                OrderAction::MarketAccept
            } else {
                OrderAction::Complete
            }
        } else {
            OrderAction::Cancel
        };

        let next_status = OrderStateMachine::next_state(current_status, action, tenant_type)
            .map_err(|e| {
                error!("Order state transition error: {}", e);
                AppError::Validation(e.to_string())
            })?;

        sqlx::query!(
            r#"UPDATE orders SET order_status = ? WHERE id = ?"#,
            next_status.to_str(),
            order.id
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to update order status"))?;

        tx.commit()
            .await
            .map_err(map_db_err!("Failed to commit transaction"))?;
        Ok(next_status)
    }

    async fn create_after_sales_request(
        &self,
        order_code: &str,
        claims: &Claims,
        action: OrderAction,
        tenant_type: TenantType,
    ) -> Result<OrderStatus, AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Failed to begin transaction"))?;

        // TODO: check if the order is belongs with the tenant_relationships table by tenant_type
        let order = sqlx::query!(
            r#"SELECT id, order_status FROM orders WHERE order_code = ? FOR UPDATE"#,
            order_code
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to get order"))?
        .ok_or_else(|| AppError::NotFound(format!("Order not found: {}", order_code)))?;

        let current_status = OrderStatus::try_from(order.order_status.as_str())?;

        let next_status = OrderStateMachine::next_state(current_status, action, tenant_type)
            .map_err(|e| {
                error!("Order state transition error: {}", e);
                AppError::Validation(e.to_string())
            })?;

        sqlx::query!(
            r#"UPDATE orders SET order_status = ? WHERE id = ?"#,
            next_status.to_str(),
            order.id
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to update order status"))?;

        // update order inspections status to PARTIAL
        sqlx::query!(
            r#"UPDATE order_inspections SET inspection_result = 'PARTIAL' WHERE order_id = ? AND inspected_by_type = ? AND inspection_result = 'PENDING'"#,
            order.id, claims.tenant_type
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to update order inspections status"))?;

        // insert order status history
        self.insert_order_status_history(
            &mut tx,
            order.id,
            order.order_status.as_str(),
            next_status,
            claims.real_name.as_str(),
            action,
        )
        .await?;

        tx.commit()
            .await
            .map_err(map_db_err!("Failed to commit transaction"))?;
        Ok(next_status)
    }
}
