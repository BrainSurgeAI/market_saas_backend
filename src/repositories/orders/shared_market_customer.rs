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
use tracing::{debug, error};

#[async_trait]
pub(crate) trait SharedMarketCustomerOrderRepository: Send + Sync {
    async fn insert_order_inspection_with_aftersales(
        &self,
        receipt: &OrderReceipt,
        operator: &str,
        transaction_id: &str,
        tenant_type: &str,
    ) -> Result<(), AppError>;

    async fn update_order_status(
        &self,
        order_code: &str,
        action: OrderAction,
        claims: &Claims
    ) -> Result<OrderStatus, AppError>;

    async fn begin_inspect_order(
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
    ) -> Result<(), AppError> {
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
            SELECT od.id as detail_id, o.order_status, oi.id as inspection_id, 
                pdi.actual_qty as actual_quantity, pdi.subtotal as total_amount, od.actual_price as actual_price
            FROM orders o 
            INNER JOIN order_details od ON o.id = od.order_id
            INNER JOIN order_inspections oi ON o.id = oi.order_id
            INNER JOIN provider_delivery_items pdi ON od.id = pdi.order_detail_id
            WHERE o.order_code = ? AND od.id = ? AND oi.inspected_by_type = ? FOR UPDATE"#,
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

       // let order_id = record.id;
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
                      (order_detail_id, operation_type, quantity, reason, reason_description, created_by, evidence_images)
                      VALUES (?, ?, ?, ?, ?, ?, ?)"#,
                    detail_id, operation_type_str, record.actual_quantity, receipt.reason, receipt.reason, operator, evidence_json
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
                // 创建换货记录， 换货允许只换部分，所以换货数量按前端传入的换货数量进行更新
                let operation_type_str = String::from(ReceiptOperationType::Exchange);
                let return_exchange_id = sqlx::query!(r#"
                    INSERT INTO return_exchange_records
                    (order_detail_id, operation_type, quantity, reason, reason_description, created_by, evidence_images)
                    VALUES (?, ?, ?, ?, ?, ?, ?)"#,
                    detail_id, operation_type_str, receipt.quantity, receipt.reason, receipt.reason, operator, evidence_json
                  )
                  .execute(&mut *tx)
                  .await
                  .map_err(map_db_err!("Failed to create exchange record"))?
                  .last_insert_id();

                sqlx::query!(r#"
                    INSERT INTO exchange_items(return_exchange_id, product_code, product_name, quantity, price, total_amount)
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
        order_code: &str,
        action: OrderAction,
        claims: &Claims
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

        // 因市场前面验收过，对 order_details 表中的 status 进行重置到待验收
        // if next_status == OrderStatus::MarketDelivering {
        //     sqlx::query!(
        //         r#"UPDATE order_details SET status = 'PENDING' WHERE order_id = ?"#,
        //         order.id
        //     )
        //     .execute(&mut *tx)
        //     .await
        //     .map_err(map_db_err!("Failed to update sub order status"))?;
        // }

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
            action
        )
        .await?;

        debug!("Order status updated to {}", next_status);

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

    async fn begin_inspect_order(
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
            r#"SELECT id, inspection_round FROM order_inspections WHERE order_id = ?"#,
            order.id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to get inspection round"))?;

        let (parent_id, inspection_round) = if let Some(inspection_record) = inspection {
            (Some(inspection_record.id), inspection_record.inspection_round + 1)
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
            TenantType::try_from(claims.tenant_type.as_str())?
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
            action
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

        // 查询订单下所有商品的签收、退货和换货情况
        // 使用 GROUP BY 确保每个商品只统计一次
        let product_statuses = sqlx::query!(
            r#"
            SELECT o.id as order_id, o.order_status,
                od.id as order_detail_id,
                MAX(CASE WHEN rer.operation_type = 'EXCHANGE' THEN 1 ELSE 0 END) as has_exchange,
                MAX(CASE WHEN rer.operation_type = 'RETURN' THEN 1 ELSE 0 END) as has_return,
                MAX(CASE WHEN oii.id IS NOT NULL THEN 1 ELSE 0 END) as has_sign
            FROM orders o
            INNER JOIN order_details od ON o.id = od.order_id
            LEFT JOIN return_exchange_records rer ON od.id = rer.order_detail_id
            LEFT JOIN order_inspections oi ON o.id = oi.order_id AND oi.inspected_by_type = ?
            LEFT JOIN order_inspection_items oii ON oi.id = oii.inspection_id AND od.id = oii.order_detail_id
            WHERE o.order_code = ?
            GROUP BY od.id FOR UPDATE
            "#,
            claims.tenant_type,
            order_code
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to query product inspection status"))?;

        if product_statuses.is_empty() {
            return Err(AppError::NotFound(format!("订单 {} 没有找到商品明细", order_code)));
        }

        let (action, inspection_status) = if product_statuses.iter().any(|status| status.has_exchange == Some(1)) {
            (match tenant_type {
                TenantType::Market => OrderAction::MarketExchange,
                TenantType::Customer => OrderAction::CustomerExchange,
                _ => return Err(AppError::Validation("不支持的租户类型".to_string())),
            }, "PARTIAL")
        } else if product_statuses.iter().any(|status| status.has_sign == Some(1)) {
            (match tenant_type {
                TenantType::Market => OrderAction::MarketAccept,
                TenantType::Customer => OrderAction::Complete,
                _ => return Err(AppError::Validation("不支持的租户类型".to_string())),
            }, "PASS")
        } else if product_statuses.iter().any(|status| status.has_return == Some(1)) {
            (match tenant_type {
                TenantType::Market => OrderAction::MarketReturn,
                TenantType::Customer => OrderAction::CustomerReturn,
                _ => return Err(AppError::Validation("不支持的租户类型".to_string())),
            }, "REJECTED")
        } else {
            (match tenant_type {
                TenantType::Market => OrderAction::MarketAccept,
                TenantType::Customer => OrderAction::Complete,
                _ => return Err(AppError::Validation("不支持的租户类型".to_string())),
            }, "PENDING")
        };

        // update order inspection status to COMPLETED
        sqlx::query!(
            r#"UPDATE order_inspections SET inspection_result = ? WHERE order_id = ? AND inspected_by_type = ?"#,
            inspection_status,
            product_statuses.first().unwrap().order_id,
            claims.tenant_type
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to update order inspection status"))?;

        // update order status
        let next_status = OrderStateMachine::next_state(OrderStatus::try_from(product_statuses.first().unwrap().order_status.as_str())?, action, tenant_type)
            .map_err(|e| {
                error!("Order state transition error: {}", e);
                AppError::Validation(format!(
                    "Order status is not correct: {} -> {}",
                    product_statuses.first().unwrap().order_status,
                    action.description()
                ))
            })?;

        sqlx::query!(
            r#"UPDATE orders SET order_status = ? WHERE id = ?"#,
            next_status.to_str(),
            product_statuses.first().unwrap().order_id
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to update order status"))?;

        // insert order status history
        self.insert_order_status_history(
            &mut tx,
            product_statuses.first().unwrap().order_id,
            product_statuses.first().unwrap().order_status.as_str(),
            next_status,
            claims.real_name.as_str(),
            action
        )
        .await?;

        if action == OrderAction::Complete {
            // 查询 CUSTOMER 签收的数量，更新 order_details 表中的 accepted_quantity 和 actual_amount
            let customer_inspection_items = sqlx::query!(
                r#"
                SELECT 
                    od.id as order_detail_id,
                    COALESCE(SUM(oii.inspected_qty), 0) as total_inspected_qty,
                    od.actual_price
                FROM order_details od
                INNER JOIN orders o ON od.order_id = o.id
                LEFT JOIN order_inspections oi ON o.id = oi.order_id AND oi.inspected_by_type = 'CUSTOMER'
                LEFT JOIN order_inspection_items oii ON oi.id = oii.inspection_id AND od.id = oii.order_detail_id
                WHERE o.order_code = ?
                GROUP BY od.id, od.actual_price
                "#,
                order_code
            )
            .fetch_all(&mut *tx)
            .await
            .map_err(map_db_err!("Failed to query customer inspection items"))?;

            // 更新每个订单明细的 accepted_quantity 和 actual_amount
            for item in &customer_inspection_items {
                let accepted_quantity = item.total_inspected_qty;
                let actual_amount = item.actual_price * accepted_quantity;

                sqlx::query!(
                    r#"UPDATE order_details 
                       SET accepted_quantity = ?, 
                           actual_amount = ?
                       WHERE id = ?"#,
                    accepted_quantity,
                    actual_amount,
                    item.order_detail_id
                )
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to update order detail accepted quantity and actual amount"))?;
            }
        }

        tx.commit()
            .await
            .map_err(map_db_err!("Failed to commit transaction"))?;
        Ok(next_status)

    }
}
