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
            }
            _=> debug!("Process receipt: {:?}", receipt),
        }

        sqlx::query!(
            r#"INSERT INTO order_inspection_items (inspection_id, order_detail_id, inspected_qty, remarks, result) VALUES (?, ?, ?, ?, ?)"#,
            inspection_id, detail_id, receipt.quantity, receipt.reason, ReceiptOperationType::to_str(receipt.operation_type.clone())
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to insert order inspection item"))?;

        // 验证订单中所有商品是否已经验收完毕
        // 算法：
        // 1.从指定的order_id查看order_details表的记录数是否与order_inspections表当前round对应的order_inspection_items表的记录数一致
        // 2. 如果不一致，则返回 PENDING
        // 3. 如果一致,获取order_inspections当前round的order_inspection_items表的 result 字段，通过result字段判断返回
        //    a.如果result记录中至少一个是EXCHANGE，则返回 EXCHANGE
        //    b.如果result记录中都是RETURN，则返回 REJECTED
        //    c.如果result记录中都是SIGN，则返回 PASS
        //    d.返回PARTIAL

        // 获取当前的最大轮次
        let current_round = sqlx::query!(
            r#"SELECT MAX(inspection_round) as max_round FROM order_inspections WHERE order_id = ? AND inspected_by_type = ?"#,
            order_id, tenant_type
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to get current inspection round"))?
        .max_round
        .unwrap_or(1);

        // 获取当前用户应该验收的订单详情总数
        let tenant_type_enum = TenantType::try_from(tenant_type)?;
        let order_details_count: i64 = match tenant_type_enum {
            TenantType::Market => {
                sqlx::query!(
                    r#"SELECT COUNT(*) as count FROM provider_delivery_items pdi
                     INNER JOIN provider_deliveries pd ON pdi.delivery_id = pd.id
                     WHERE pd.assignment_id = ? AND pd.delivery_round = ?"#,
                    order_id, current_round
                )
                .fetch_one(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to get order details count"))?
                .count
            }
            TenantType::Customer => {
                sqlx::query!(
                    r#"SELECT COUNT(*) as count
                     FROM order_inspections customer_oi
                     INNER JOIN order_inspections parent_oi ON customer_oi.parent_id = parent_oi.id
                     INNER JOIN order_inspection_items parent_items ON parent_oi.id = parent_items.inspection_id
                     WHERE customer_oi.order_id = ?
                     AND customer_oi.inspected_by_type = 'CUSTOMER'
                     AND parent_items.result != 'RETURN'"#,
                    order_id
                )
                .fetch_one(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to get order details count"))?
                .count
            }
            TenantType::Provider => {
                return Err(AppError::BadRequest(format!("Provider tenant type not supported for this operation")));
            }
        };

        // 获取当前轮次的检验项目总数
        let inspection_items_count = sqlx::query!(
            r#"SELECT COUNT(*) as count FROM order_inspection_items oii
               INNER JOIN order_inspections oi ON oii.inspection_id = oi.id
               WHERE oi.order_id = ? AND oi.inspection_round = ? AND oi.inspected_by_type = ?"#,
            order_id, current_round, tenant_type
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to get inspection items count"))?
        .count;

        let inspect_status = if order_details_count != inspection_items_count {
            "PENDING"
        } else {
            // 获取当前轮次的检验结果
            let inspection_results = sqlx::query!(
                r#"SELECT oii.result FROM order_inspection_items oii
                   INNER JOIN order_inspections oi ON oii.inspection_id = oi.id
                   WHERE oi.order_id = ? AND oi.inspection_round = ?"#,
                order_id, current_round
            )
            .fetch_all(&mut *tx)
            .await
            .map_err(map_db_err!("Failed to get inspection results"))?;

            // 分析结果
            let has_exchange = inspection_results.iter().any(|r| r.result == "EXCHANGE");
            let all_return = inspection_results.iter().all(|r| r.result == "RETURN");
            let all_sign = inspection_results.iter().all(|r| r.result == "SIGN");

            if has_exchange {
                "EXCHANGE"
            } else if all_return {
                "REJECTED"
            } else if all_sign {
                "PASS"
            } else {
                "PARTIAL"
            }
        };

        // 更新检验结果到order_inspections表
        if inspect_status != "PENDING" {
            sqlx::query!(
                r#"UPDATE order_inspections SET inspection_result = ? WHERE order_id = ? AND inspection_round = ?"#,
                inspect_status, order_id, current_round
            )
            .execute(&mut *tx)
            .await
            .map_err(map_db_err!("Failed to update inspection result"))?;
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
