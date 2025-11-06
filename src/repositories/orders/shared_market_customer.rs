use crate::repositories::my_sql_repository::MySqlRepository;
use crate::{
    common::AppError,
    dto::order::{OrderReceipt, ReceiptOperationType},
    map_db_err,
    models::{
        order_action::OrderAction, order_machine::OrderStateMachine, order_status::OrderStatus,
        tenant_type::TenantType,
    },
};
use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::{MySql, QueryBuilder};
use tracing::{debug, error, info};

#[async_trait]
pub(crate) trait SharedMarketCustomerOrderRepository: Send + Sync {
    async fn process_order_receipt(
        &self,
        receipt: &OrderReceipt,
        operator: &str,
        transaction_id: &str,
    ) -> Result<(), AppError>;

    async fn update_order_status(
        &self,
        tenant_type: TenantType,
        tenant_hash: &str,
        order_code: &str,
        action: OrderAction,
        operator: &str,
    ) -> Result<OrderStatus, AppError>;
}

#[async_trait]
impl SharedMarketCustomerOrderRepository for MySqlRepository {
    async fn process_order_receipt(
        &self,
        receipt: &OrderReceipt,
        operator: &str,
        transaction_id: &str,
    ) -> Result<(), AppError> {
        // 验证订单存在且状态正确
        let record = sqlx::query!(
            r#"SELECT o.id, od.id as detail_id, o.order_status, od.actual_quantity, od.actual_price
               FROM orders o 
               JOIN order_details od ON o.id = od.order_id 
               WHERE o.order_code = ? AND od.id = ?"#,
            receipt.order_code,
            receipt.order_detail_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!("Failed to find order"))?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "Order {} with order detail id {} not found",
                receipt.order_code, receipt.order_detail_id
            ))
        })?;

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

        let order_id = record.id;
        let detail_id = record.detail_id;
        let actual_quantity = record.actual_quantity.unwrap_or(Decimal::from(0));
        let actual_price = record.actual_price;

        let actual_quantity_diff = actual_quantity - receipt.quantity;
        let actual_amount_diff = actual_price * actual_quantity_diff;

        match receipt.operation_type {
            ReceiptOperationType::Sign => {
                // 处理正常签收
                debug!("Process normal sign receipt: {:?}", receipt);
                sqlx::query!(
                    r#"UPDATE order_details 
                      SET status = 'SIGN', 
                        receipt_quantity = ?, 
                        receipt_date = NOW(), 
                        receipt_notes = ?,
                        receipt_evidence = ?,
                        actual_quantity = ?,
                        actual_amount = actual_amount - ?
                      WHERE id = ?"#,
                    receipt.quantity, // 签收数量
                    receipt.reason,
                    evidence_json,
                    receipt.quantity,
                    actual_amount_diff,
                    detail_id
                )
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to update receipt status"))?;

                // 处理签收数量与实际数量不一致的情况，更新订单实际总金额
                sqlx::query!(
                    r#"UPDATE orders SET actual_amount = actual_amount - ? WHERE id = ?"#,
                    actual_amount_diff,
                    order_id
                )
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to update order total amount"))?;
            }
            ReceiptOperationType::Return => {
                // 处理退货
                sqlx::query!(
                    r#"UPDATE order_details 
                      SET status = 'RETURNED', 
                        receipt_quantity = actual_quantity - ?, 
                        receipt_date = NOW(), 
                        receipt_notes = ?,
                        receipt_evidence = ?,
                        actual_quantity = actual_quantity - ?,
                        actual_amount = ?    
                      WHERE id = ?"#,
                    receipt.quantity,
                    receipt.reason,
                    evidence_json,
                    receipt.quantity, // 退货数量
                    actual_amount_diff,
                    detail_id
                )
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to update return status"))?;

                // 处理签收数量与实际数量不一致的情况，更新订单实际总金额
                sqlx::query!(
                    r#"UPDATE orders SET actual_amount = actual_amount - ? WHERE id = ?"#,
                    receipt.quantity * actual_price,
                    order_id
                )
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to update order total amount"))?;

                // 创建退货记录
                let operation_type_str = String::from(ReceiptOperationType::Return);
                let return_exchange_id = sqlx::query!(
                    r#"INSERT INTO return_exchange_records 
                      (order_detail_id, operation_type, quantity, reason, reason_description, created_by, evidence_images)
                      VALUES (?, ?, ?, ?, ?, ?, ?)"#,
                    detail_id, operation_type_str, receipt.quantity, receipt.reason, receipt.reason, operator, evidence_json
                )
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to create return record"))?
                .last_insert_id();

                sqlx::query!(
                    r#"INSERT INTO refund_records (return_exchange_id, amount, method, transaction_id, status, created_by)
                      VALUES (?, ?, ?, ?, ?, ?)"#,
                    return_exchange_id, actual_amount_diff, "ORIGINAL", transaction_id, "PENDING", operator
                )
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to create refund record"))?;
            }
            ReceiptOperationType::Exchange => {
                sqlx::query!(
                    r#"UPDATE order_details 
                      SET status = 'EXCHANGED', 
                        receipt_quantity = quantity - ?, 
                        receipt_date = NOW(), 
                        receipt_notes = ?,
                        receipt_evidence = ?,
                        actual_quantity = quantity - ?,
                        actual_amount = actual_amount + ?
                      WHERE id = ?"#,
                    receipt.quantity,
                    receipt.reason,
                    evidence_json,
                    receipt.quantity,
                    actual_amount_diff,
                    detail_id
                )
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to update exchange status"))?;

                // 创建换货记录
                let operation_type_str = String::from(ReceiptOperationType::Exchange);
                let return_exchange_id = sqlx::query!(
                    r#"INSERT INTO return_exchange_records 
                      (order_detail_id, operation_type, quantity, reason, reason_description, created_by, evidence_images)
                      VALUES (?, ?, ?, ?, ?, ?, ?)"#,
                    detail_id, operation_type_str, receipt.quantity, receipt.reason, receipt.reason, operator, evidence_json
                )
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to create exchange record"))?
                .last_insert_id();

                sqlx::query!(
                            r#"INSERT INTO exchange_items
                              (return_exchange_id, product_code, product_name, quantity, price, total_amount)
                              VALUES (?, ?, ?, ?, ?, ?)"#,
                    return_exchange_id, receipt.product_code, receipt.product_name, receipt.quantity, actual_price, actual_price * receipt.quantity
                )
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to create exchange item"))?;
            }
        }

        tx.commit()
            .await
            .map_err(map_db_err!("Failed to commit transaction"))?;
        Ok(())
    }

    async fn update_order_status(
        &self,
        tenant_type: TenantType,
        _tenant_hash: &str,
        order_code: &str,
        action: OrderAction,
        operator: &str,
    ) -> Result<OrderStatus, AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Failed to begin transaction"))?;

        // Use SELECT FOR UPDATE to lock the record, prevent concurrent modification
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

        // 因市场前面验收过，对 order_details 表中的 status 进行重置到待验收
        if next_status == OrderStatus::MarketDelivering {
            sqlx::query!(
                r#"UPDATE order_details SET status = 'PENDING' WHERE order_id = ?"#,
                order.id
            )
            .execute(&mut *tx)
            .await
            .map_err(map_db_err!("Failed to update sub order status"))?;
        }

        sqlx::query!(
            r#"UPDATE orders SET order_status = ? WHERE id = ?"#,
            next_status.to_str(),
            order.id
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to update order status"))?;

        sqlx::query!(
            r#"INSERT INTO order_status_history (order_id, from_status, to_status, changed_by, change_reason ) VALUES (?, ?, ?, ?, ?)"#,
            order.id, order.order_status, next_status.to_str(), operator, action.description()
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to insert order status history"))?;

        debug!("Order status updated to {}", next_status);

        // 如果是客户开始验收，则需要重置子订单状态到待验收
        // if tenant_type == TenantType::Customer && action == OrderAction::CustomerInspect {
        //     sqlx::query!(
        //         r#"UPDATE order_details SET status = 'PENDING' WHERE order_id = ?"#,
        //         order.id
        //     )
        //     .execute(&mut *tx)
        //     .await
        //     .map_err(map_db_err!("Failed to reset sub order status"))?;
        // }

        // 如果客户直接确认收货，则将子订单更新到
        if tenant_type == TenantType::Customer && action == OrderAction::Complete {
            sqlx::query!(
                r#"UPDATE order_details SET status = 'SIGN' WHERE order_id = ?"#,
                order.id
            )
            .execute(&mut *tx)
            .await
            .map_err(map_db_err!("Failed to update sub order status"))?;
        }

        tx.commit()
            .await
            .map_err(map_db_err!("Failed to commit transaction"))?;
        Ok(next_status)
    }
}
