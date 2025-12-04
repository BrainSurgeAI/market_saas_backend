use crate::repositories::my_sql_repository::MySqlRepository;
use crate::{
    common::AppError,
    dto::order::{
        NeedToInspection, NeedToInspectionItem, OrderInspection, OrderInspectionItem,
        OrderReceipt, ReceiptOperationType,
    },
    map_db_err,
    models::{
        claims::Claims, order_action::OrderAction, order_machine::OrderStateMachine,
        order_status::OrderStatus, tenant_type::TenantType,
    },
};
use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::{MySql, QueryBuilder};
use tracing::{debug, error};

use chrono::{DateTime, Utc};

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

    async fn init_order_inspection_with_items(
        &self,
        order_code: &str,
        claims: &Claims,
    ) -> Result<OrderStatus, AppError>;

    async fn get_order_inspections(
        &self,
        order_code: &str,
        claims: &Claims,
    ) -> Result<NeedToInspection, AppError>;

    async fn get_order_inspection_history(
        &self,
        order_code: &str,
        claims: &Claims,
    ) -> Result<Vec<OrderInspection>, AppError>;

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

        //let tenant_type_enum = TenantType::try_from(tenant_type)?;

        let record = sqlx::query!(
            r#"
        SELECT oii.order_detail_id as detail_id, oi.id as inspection_id,
            oii.need_to_inspection  as actual_quantity, od.discounted_unit_price as actual_price
        FROM order_inspection_items oii
        INNER JOIN order_inspections oi ON oii.inspection_id = oi.id
        INNER JOIN orders o ON oi.order_id = o.id
        INNER JOIN order_details od ON oii.order_detail_id = od.id
        WHERE o.order_code = ? AND oii.id = ? AND oi.inspected_by_type = ? FOR UPDATE"#,
            receipt.order_code,
            receipt.order_inspection_id,
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
                receipt.order_code, receipt.order_inspection_id
            ))
        })?;

        let detail_id = record.detail_id;
        let inspection_id = record.inspection_id;
        let actual_quantity = record.actual_quantity;
        let actual_price = record.actual_price;
        let total_amount = record.actual_quantity * record.actual_price;

        match receipt.operation_type {
            ReceiptOperationType::Return => {
                debug!("Return operation type");
                let operation_type_str = String::from(ReceiptOperationType::Return);

                // 退货只能全部退货，所以退货按实际发货量全部退货，不从前端拿退货数量
                let return_exchange_id = sqlx::query!(
                    r#"INSERT INTO return_exchange_records 
                      (order_detail_id, operation_type, quantity, reason, reason_description, created_by, evidence_images, inspection_id)
                      VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
                    detail_id, operation_type_str, actual_quantity, receipt.reason, receipt.reason, operator, evidence_json, inspection_id
                )
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to create return record"))?
                .last_insert_id();

                sqlx::query!(
                    r#"INSERT INTO refund_records (return_exchange_id, amount, method, transaction_id)
                      VALUES (?, ?, ?, ?)"#,
                    return_exchange_id, total_amount, "ORIGINAL", transaction_id
                )
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to create refund record"))?;
            }
            ReceiptOperationType::Exchange => {
                debug!("Exchange operation type");

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
                debug!(
                    "ordered_qty: {},  receipt quantity: {}, exchange_quantity: {}",
                    ordered_qty_record.ordered_qty, receipt.quantity, exchange_quantity
                );

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
            _ => debug!("Process receipt: {:?}", receipt),
        }

        let is_accepted = match receipt.operation_type {
            ReceiptOperationType::Return => false,
            ReceiptOperationType::Exchange => true,
            _ => true,
        };
        sqlx::query!(
            r#"UPDATE order_inspection_items SET inspected_qty = ?, remarks = ?, result = ?, accepted = ? WHERE id = ?"#,
            receipt.quantity, receipt.reason, ReceiptOperationType::to_str(receipt.operation_type.clone()), is_accepted, receipt.order_inspection_id
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to insert order inspection item"))?;

        // 检查 order_inspection_items 表中是否还有未检验的商品
        let uninspected_count: i64 = sqlx::query!(
            r#"SELECT COUNT(*) as count FROM order_inspection_items WHERE inspection_id = ? AND result = 'PENDING'"#,
            inspection_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to get uninspected items"))?
        .count;

        let inspection_result = if uninspected_count == 0 {
            let inspection_results = sqlx::query!(
                r#"SELECT result FROM order_inspection_items WHERE inspection_id = ?"#,
                inspection_id
            )
            .fetch_all(&mut *tx)
            .await
            .map_err(map_db_err!("Failed to get inspection results"))?;

            let has_exchange = inspection_results.iter().any(|r| r.result == "EXCHANGE");
            let has_sign = inspection_results.iter().any(|r| r.result == "SIGN");
            let has_return = inspection_results.iter().any(|r| r.result == "RETURN");

            if has_exchange {
                sqlx::query!(
                    r#"UPDATE order_inspections SET inspection_result = 'EXCHANGE' WHERE id = ?"#,
                    inspection_id
                )
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to update inspection result"))?;

                "EXCHANGE"
            } else if has_sign {
                if has_return {
                    sqlx::query!(
                        r#"UPDATE order_inspections SET inspection_result = 'PARTIAL' WHERE id = ?"#,
                        inspection_id
                    )
                    .execute(&mut *tx)
                    .await
                    .map_err(map_db_err!("Failed to update inspection result"))?;
                    "PARTIAL"
                } else {
                    sqlx::query!(
                        r#"UPDATE order_inspections SET inspection_result = 'PASS' WHERE id = ?"#,
                        inspection_id
                    )
                    .execute(&mut *tx)
                    .await
                    .map_err(map_db_err!("Failed to update inspection result"))?;
                    "PASS"
                }
            } else {
                sqlx::query!(
                    r#"UPDATE order_inspections SET inspection_result = 'REJECTED' WHERE id = ?"#,
                    inspection_id
                )
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to update inspection result"))?;
                "REJECTED"
            }
        } else {
            "PENDING"
        };

        tx.commit()
            .await
            .map_err(map_db_err!("Failed to commit transaction"))?;

        // debug!("Inspection status: {}", inspect_status);

        Ok(inspection_result.to_string())
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

    async fn init_order_inspection_with_items(
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
            r#"SELECT id, order_status FROM orders WHERE order_code = ? FOR UPDATE"#,
            order_code
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to get order"))?
        .ok_or_else(|| AppError::NotFound(format!("Order not found: {}", order_code)))?;

        // Create a new order inspection record
        self.create_order_inspection_record(&mut tx, order.id, claims)
            .await?;

        // Update the order status to MarketInspecting or CustomerInspecting
        let action = if tenant_type == TenantType::Market {
            if order.order_status == OrderStatus::SupplierDelivering.to_str()
                || order.order_status == OrderStatus::ExchangeDelivering.to_str()
            {
                OrderAction::MarketInspect
            } else {
                return Err(AppError::bad_request(format!(
                    "Order status is not correct: {} -> {}",
                    order.order_status,
                    OrderAction::MarketInspect.description()
                )));
            }
        } else {
            // Customer tenant type
            if order.order_status == OrderStatus::MarketDelivering.to_str()
                || order.order_status == OrderStatus::ExchangeNewDelivering.to_str()
            {
                OrderAction::CustomerInspect
            } else {
                return Err(AppError::bad_request(format!(
                    "Order status is not correct: {} -> {}",
                    order.order_status,
                    OrderAction::CustomerInspect.description()
                )));
            }
        };

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
            r#"SELECT o.id, o.order_status, oi.inspection_result, oi.id as inspection_id
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
                // 从order_inspection_items表中获取所有accepted为1的order_detail_id和inspected_qty, 关联order_details表,
                // 用order_details表对应的discounted_unit_price和inspected_qty计算net_amount, 然后更新order_details表的net_amount和orders表的net_amount
                // 使用GROUP BY聚合相同order_detail_id的inspected_qty
                let order_details = sqlx::query!(
                    r#"select oii.order_detail_id, SUM(oii.inspected_qty) as total_inspected_qty, od.discounted_unit_price 
                    from order_inspection_items oii 
                    inner join order_inspections oi on oi.id = oii.inspection_id 
                    inner join order_details od on od.id = oii.order_detail_id 
                    where oi.order_id = ? and oi.inspected_by_type = 'CUSTOMER' and oii.accepted = 1
                    group by oii.order_detail_id, od.discounted_unit_price"#,
                    order.id
                )
                .fetch_all(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to get order details"))?;

                if order_details.is_empty() {
                    return Err(AppError::internal(
                        "No order details found for inspection",
                    ));
                }

                debug!("Order details: {:?}", order_details);

                // 使用批量更新提升性能
                let mut order_net_amount = Decimal::ZERO;
                let mut update_values: Vec<(i32, Decimal, Decimal)> = Vec::new();

                for order_detail in order_details {
                    let inspected_qty = order_detail.total_inspected_qty.unwrap_or(Decimal::ZERO);
                    let net_amount = order_detail.discounted_unit_price * inspected_qty;
                    update_values.push((order_detail.order_detail_id, net_amount, inspected_qty));
                    order_net_amount += net_amount;
                    debug!("order_net_amount: {:?}", order_net_amount);
                }

                // 使用CASE WHEN进行批量更新，同时更新net_amount和accepted_qty，使用QueryBuilder避免SQL注入
                if !update_values.is_empty() {
                    let mut builder: QueryBuilder<MySql> = QueryBuilder::new(
                        "UPDATE order_details SET net_amount = CASE id ",
                    );
                    
                    // 构建第一个 CASE WHEN (net_amount)
                    for (id, net_amount, _) in &update_values {
                        builder.push("WHEN ").push_bind(id).push(" THEN ").push_bind(net_amount).push(" ");
                    }
                    
                    builder.push("END, accepted_qty = CASE id ");
                    
                    // 构建第二个 CASE WHEN (accepted_qty)
                    // 注意：由于 SQL 语法要求先完成第一个 CASE WHEN，再开始第二个，所以需要两次遍历
                    // 这是 SQL 结构决定的，无法合并成一个循环
                    for (id, _, accepted_qty) in &update_values {
                        builder.push("WHEN ").push_bind(id).push(" THEN ").push_bind(accepted_qty).push(" ");
                    }
                    
                    builder.push("END WHERE id IN (");
                    let mut separated = builder.separated(", ");
                    for (id, _, _) in &update_values {
                        separated.push_bind(id);
                    }
                    separated.push_unseparated(")");

                    let sql = builder.sql();
                    debug!("Batch update SQL: {}", sql);

                    builder
                        .build()
                        .execute(&mut *tx)
                        .await
                        .map_err(map_db_err!("Failed to batch update order detail net amount and accepted_qty"))?;
                }

                sqlx::query!(
                    r#"UPDATE orders SET net_amount = ? WHERE id = ?"#,
                    order_net_amount,
                    order.id
                )
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to update order net amount"))?;
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

    // Only for Market tenant type
    async fn get_order_inspections(
        &self,
        order_code: &str,
        claims: &Claims,
    ) -> Result<NeedToInspection, AppError> {
        let tenant_type = TenantType::try_from(claims.tenant_type.as_str())?;
        let response = match tenant_type {
            TenantType::Market => {
                // 获取供应商最近一次配送信息
                let latest_delivery = sqlx::query!(
                    r#"
                    SELECT pd.delivery_type, pd.delivered_at 
                    FROM provider_deliveries pd
                    INNER JOIN provider_orders_assignments poa ON pd.assignment_id = poa.id
                    INNER JOIN orders o ON poa.order_id = o.id
                    WHERE o.order_code = ? AND pd.delivery_status = 'DELIVERED'
                    ORDER BY pd.delivery_round DESC LIMIT 1
                    "#,
                    order_code
                )
                .fetch_optional(&self.pool)
                .await
                .map_err(map_db_err!("Failed to get latest delivery"))?
                .ok_or_else(|| {
                    AppError::NotFound(format!("Latest delivery not found: {}", order_code))
                })?;

                let delivery_type = latest_delivery.delivery_type;
                let delivered_at = latest_delivery
                    .delivered_at
                    .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc));

                let delivery_staff = None; // TODO: 需要从 delivery_staff 表查询

                // 市场用户的需要验收的商品信息，从order_inspection_items表中获取
                let market_inspection = sqlx::query!(
                    r#"
                    SELECT oi.id as inspection_id, oi.inspection_round, oi.inspection_result, oi.inspected_at
                    FROM order_inspections oi
                    INNER JOIN orders o ON oi.order_id = o.id
                    WHERE o.order_code = ? AND oi.inspected_by_type = ?
                    ORDER BY inspection_round DESC LIMIT 1
                    "#,
                order_code, claims.tenant_type)
                .fetch_optional(&self.pool)
                .await
                .map_err(map_db_err!("Failed to get market inspection"))?
                .ok_or_else(|| AppError::NotFound(format!("Market inspection not found: {}", order_code)))?;

                // 获取市场用户的需要验收的商品信息，从order_inspection_items表中获取
                let market_inspection_items = sqlx::query_as!(
                NeedToInspectionItem,
                r#"
                    SELECT oii.id AS inspection_item_id, od.product_code, od.product_name, od.category_id,
                        od.category_name, od.unit, od.discount_rate, od.unit_price, od.ordered_qty,
                        oii.need_to_inspection AS need_to_inspect_qty, oii.inspected_qty,
                        oii.result as inspection_status, od.processing_requirements
                    FROM order_inspection_items oii
                    INNER JOIN order_details od ON oii.order_detail_id = od.id
                    WHERE oii.inspection_id = ?
                    ORDER BY oii.order_detail_id ASC
                "#,
                market_inspection.inspection_id)
                .fetch_all(&self.pool)
                .await
                .map_err(map_db_err!("Failed to get market inspection items"))?;

                NeedToInspection {
                    round: market_inspection.inspection_round,
                    delivery_type: delivery_type,
                    delivered_at: delivered_at,
                    inspection_result: market_inspection.inspection_result,
                    inspection_at: market_inspection
                        .inspected_at
                        .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc)),
                    delivery_staff,
                    items: market_inspection_items,
                }
            }
            TenantType::Customer => {
                let customer_inspection = sqlx::query!(
                    r#"
                    SELECT oi.id as inspection_id, oi.inspection_round, oi.inspection_result, oi.inspected_at
                    FROM order_inspections oi
                    INNER JOIN orders o ON oi.order_id = o.id
                    WHERE o.order_code = ? AND oi.inspected_by_type = ?
                    ORDER BY inspection_round DESC LIMIT 1
                "#, order_code, claims.tenant_type)
                .fetch_optional(&self.pool)
                .await
                .map_err(map_db_err!("Failed to get customer inspection"))?
                .ok_or_else(|| AppError::NotFound(format!("Customer inspection not found: {}", order_code)))?;

                let customer_inspection_items = sqlx::query_as!(
                    NeedToInspectionItem,
                    r#"
                    SELECT oii.id AS inspection_item_id, od.product_code, od.product_name, od.category_id,
                           od.category_name, od.unit, od.discount_rate, od.unit_price, od.ordered_qty,
                           oii.need_to_inspection AS need_to_inspect_qty, oii.inspected_qty,
                           oii.result as inspection_status, od.processing_requirements
                    FROM order_inspection_items oii
                    INNER JOIN order_details od ON oii.order_detail_id = od.id
                    WHERE oii.inspection_id = ?
                    ORDER BY oii.order_detail_id ASC
                    "#,
                    customer_inspection.inspection_id
                ).fetch_all(&self.pool).await.map_err(map_db_err!("Failed to get customer inspection items"))?;

                NeedToInspection {
                    round: customer_inspection.inspection_round,
                    delivery_type: "".to_string(),
                    delivered_at: None,
                    inspection_result: customer_inspection.inspection_result,
                    inspection_at: customer_inspection
                        .inspected_at
                        .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc)),
                    delivery_staff: None,
                    items: customer_inspection_items,
                }
            }
            TenantType::Provider => {
                return Err(AppError::NotFound(format!(
                    "Provider tenant type not supported: {}",
                    claims.tenant_type
                )));
            }
        };
        Ok(response)
    }

    async fn get_order_inspection_history(
        &self,
        order_code: &str,
        claims: &Claims,
    ) -> Result<Vec<OrderInspection>, AppError> {
        // 查询所有验收记录，按验收轮次排序
        let inspections = sqlx::query!(
            r#"
            SELECT oi.id as inspection_id, oi.parent_id, oi.inspection_round, 
                   oi.inspected_by_type, oi.inspected_by_id, oi.inspection_result, oi.inspected_at
            FROM order_inspections oi
            INNER JOIN orders o ON oi.order_id = o.id
            WHERE o.order_code = ? AND oi.inspected_by_type = ?
            ORDER BY oi.inspection_round ASC
            "#,
            order_code,
            claims.tenant_type
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get inspection history"))?;

        if inspections.is_empty() {
            return Ok(Vec::new());
        }

        // 批量查询所有验收项，关联订单明细表获取产品信息
        let inspection_items = sqlx::query!(
            r#"
            SELECT oii.inspection_id, oii.order_detail_id, oii.inspected_qty, oii.remarks,
                   oii.result, oii.need_to_inspection,
                   od.product_code, od.product_name, od.category_name
            FROM order_inspection_items oii
            INNER JOIN order_details od ON oii.order_detail_id = od.id
            WHERE oii.inspection_id IN (
                SELECT oi.id FROM order_inspections oi
                INNER JOIN orders o ON oi.order_id = o.id
                WHERE o.order_code = ? AND oi.inspected_by_type = ?
            )
            ORDER BY oii.inspection_id ASC, oii.order_detail_id ASC
            "#,
            order_code,
            claims.tenant_type
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get inspection items"))?;

        // 将验收项按 inspection_id 分组
        use std::collections::HashMap;
        let mut items_by_inspection: HashMap<i32, Vec<OrderInspectionItem>> = HashMap::new();
        
        for item in inspection_items {
            let inspection_id = item.inspection_id as i32;
            let inspected_qty = item.inspected_qty.unwrap_or(Decimal::ZERO);
            let inspection_item = OrderInspectionItem {
                order_detail_id: item.order_detail_id,
                product_code: item.product_code,
                product_name: item.product_name,
                category_name: item.category_name,
                result: item.result,
                need_to_inspection: item.need_to_inspection,
                inspected_qty: Some(inspected_qty),
                quantity: Some(inspected_qty),
                remark: item.remarks,
            };
            items_by_inspection
                .entry(inspection_id)
                .or_insert_with(Vec::new)
                .push(inspection_item);
        }

        // 构建验收历史记录列表
        let mut inspection_history: Vec<OrderInspection> = Vec::new();
        
        for inspection in inspections {
            let items = items_by_inspection
                .remove(&(inspection.inspection_id as i32))
                .unwrap_or_default();

            let inspection_record = OrderInspection {
                inspection_id: inspection.inspection_id as i32,
                parent_id: inspection.parent_id.map(|p| p as i32),
                inspection_round: inspection.inspection_round,
                inspected_by_type: inspection.inspected_by_type,
                inspected_by_id: inspection.inspected_by_id,
                result: inspection.inspection_result,
                inspected_at: inspection
                    .inspected_at
                    .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc)),
                items,
            };
            inspection_history.push(inspection_record);
        }

        Ok(inspection_history)
    }
}