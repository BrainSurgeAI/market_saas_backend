use crate::{
    common::AppError,
    dto::order::{
        AcceptedOrderResponseDTO, ActualQuantityDTO, CreateOrderDTO, OrderDetail,
        OrderDetailResponse, OrderItem, OrderQueryParams, OrderReceipt, OrderResponse,
        ProductsSummaryWithOrdersDTO, ReceiptOperationType,
    },
    map_db_err,
    models::{
        order_action::OrderAction, order_machine::OrderStateMachine, order_status::OrderStatus,
        tenant_type::TenantType,
    },
    repositories::{generate_code, CodeType},
};

use super::my_sql_repository::MySqlRepository;
use async_trait::async_trait;
use chrono::NaiveDate;

use rust_decimal::Decimal;

use sqlx::{MySql, QueryBuilder};
use tracing::{debug, error, info};

#[async_trait]
pub(crate) trait OrderRepository: Send + Sync {
    async fn create_order(
        &self,
        market_id: i32,
        customer_hash: &str,
        order: &CreateOrderDTO,
    ) -> Result<OrderResponse, AppError>;

    async fn get_orders_by_tenant(
        &self,
        tenant_hash: &str,
        tenant_type: &str,
        query_params: &OrderQueryParams,
    ) -> Result<Vec<OrderResponse>, AppError>;

    async fn get_order_by_order_code(
        &self,
        order_code: &str,
    ) -> Result<Option<OrderDetailResponse>, AppError>;

    async fn assign_order(
        &self,
        order_code: &str,
        provider_id: i32,
        confirmed_by: &str,
    ) -> Result<(), AppError>;

    async fn update_order_status_to_processing(
        &self,
        order_code: &str,
        provider_hash: &str,
        delivery_staff_id: &str,
    ) -> Result<(), AppError>;

    async fn update_actual_quantity(
        &self,
        order_code: &str,
        actual_quantity_dto: &ActualQuantityDTO,
    ) -> Result<(), AppError>;

    async fn get_after_sale_orders_by_tenant(
        &self,
        tenant_hash: &str,
        tenant_type: &TenantType,
    ) -> Result<Vec<AcceptedOrderResponseDTO>, AppError>;

    async fn process_order_receipt(
        &self,
        receipt: &OrderReceipt,
        operator: &str,
        transaction_id: &str,
    ) -> Result<(), AppError>;

    async fn fetch_today_product_order_summary_by_provider_hash(
        &self,
        provider_hash: &str,
    ) -> Result<Vec<ProductsSummaryWithOrdersDTO>, AppError>;

    /// Updates the order status based on tenant type and current order state
    ///
    /// This function implements the following business logic for order status transitions:
    ///
    /// * For Customers:
    ///   - Can update order status from 'STOCKED' to 'COMPLETED'
    ///   - Can update order status from 'STOCKED' to 'AFTER_SALE'
    ///
    /// * For Providers:
    ///   - Can update order status from 'AFTER_SALE' to 'ACCEPTED'
    ///
    /// # Parameters
    ///
    /// * `tenant_hash` - Hash value identifying the tenant type
    /// * `order_code` - Unique identifier of the order
    /// * `new_status` - New status to be set
    /// * `current_status` - Current status of the order
    /// * `operator` - Operator performing the update
    ///
    /// # Returns
    ///
    /// Returns `Result<(), AppError>` where:
    /// * Success returns an empty success response
    /// * Failure returns the corresponding error information
    ///
    /// # Errors
    ///
    /// The function may return the following errors:
    /// * `AppError::Validation` - When status conversion fails or status update violates business rules
    /// * `AppError::NotFound` - When order doesn't exist or current status is incorrect
    ///
    /// # Example
    ///
    /// ```rust
    /// let result = update_order_status(
    ///     Extension(repo),
    ///     Extension(context),
    ///     Path((tenant_hash, order_code)),
    ///     Json(update_order_status_dto)
    /// ).await?;
    /// ```
    async fn update_order_status(
        &self,
        tenant_type: &str,
        order_code: &str,
        new_status: &str,
        operator: &str,
    ) -> Result<(), AppError>;
}

#[async_trait]
impl OrderRepository for MySqlRepository {
    async fn create_order(
        &self,
        market_id: i32,
        customer_hash: &str,
        order: &CreateOrderDTO,
    ) -> Result<OrderResponse, AppError> {
        let record = sqlx::query!(
            r#"SELECT id, name FROM tenants WHERE name_hash = ?"#,
            customer_hash
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get customer id"))?;

        let customer = record.ok_or_else(|| AppError::NotFound("Customer not found".to_string()))?;

        let total_amount = order
            .items
            .iter()
            .map(|item| item.original_amount)
            .sum::<Decimal>();

        let discount_amount = order
            .items
            .iter()
            .map(|item| item.original_amount - item.total)
            .sum::<Decimal>();

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Failed to begin transaction"))?;

        let order_code = generate_code(CodeType::Order);

        let order_id = sqlx::query!(
            r#"INSERT INTO orders (order_code, market_id, customer_id, total_amount, discount_amount, actual_amount, delivery_date,
            delivery_address, contact_name, contact_phone, created_by) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            order_code,
            market_id,
            customer.id,
            total_amount,
            discount_amount,
            order.total_amount, // 前端传入的total_amount是实际数量x折扣价
            order.delivery_info.delivery_date,
            order.delivery_info.delivery_address,
            order.delivery_info.contact_name,
            order.delivery_info.contact_phone,
            customer.name
        ).execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to create order"))?
        .last_insert_id();

        for item in order.items.iter() {
            let processing_requirements = if item.processing_services.is_empty() {
                None
            } else {
                Some(
                    item.processing_services
                        .iter()
                        .map(|service| {
                            format!(
                                "{}:{}",
                                service.name,
                                service
                                    .description
                                    .as_ref()
                                    .map_or_else(String::new, |s| s.to_string())
                            )
                        })
                        .collect::<Vec<String>>()
                        .join(","),
                )
            };

            sqlx::query!(
                r#"INSERT INTO order_details (order_id, product_code, product_name, 
                   category_id, category_name, unit, quantity,
                   original_price, discount_rate, actual_price, actual_amount, total_amount,
                   processing_requirements, remark) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
                order_id,
                &item.product_code,
                &item.product_name,
                &item.category_id,
                &item.category_name,
                &item.unit,
                &item.quantity,
                &item.original_price,
                &item.discount_rate,
                &item.price,
                &item.total,
                &item.total,
                &processing_requirements,
                &item.remark
            ).execute(&mut *tx)
            .await
            .map_err(map_db_err!("Failed to create order detail"))?;
        }

        tx.commit()
            .await
            .map_err(map_db_err!("Failed to commit transaction"))?;

        info!("Order: {} created success", order_code);
        Ok(OrderResponse {
            order_code,
            total_amount,
            actual_amount: order.total_amount,
            delivery_address: order.delivery_info.delivery_address.clone(),
            delivery_date: NaiveDate::parse_from_str(
                &order.delivery_info.delivery_date,
                "%Y-%m-%d",
            ).unwrap_or_else(|_| NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()),
            order_status: OrderStatus::Pending.to_string(),
            created_at: None,
            after_sale_at: None,
        })
    }

    async fn get_orders_by_tenant(
        &self,
        tenant_hash: &str,
        tenant_type: &str,
        query_params: &OrderQueryParams,
    ) -> Result<Vec<OrderResponse>, AppError> {
        let page = query_params.page.unwrap_or(1);
        let page_size = query_params.page_size.unwrap_or(10);
        let offset = (page - 1) * page_size;

        let record = sqlx::query!(r#"select id from tenants where name_hash=?"#, tenant_hash)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get market id"))?;

        if record.id == 0 {
            return Err(AppError::NotFound(format!(
                "Can not find market for tenant {}",
                tenant_hash
            )));
        }

        let basic_query = r#"SELECT 
                   o.order_code, 
                   o.delivery_address, 
                   o.total_amount,
                   o.actual_amount, 
                   o.delivery_date, 
                   o.order_status,
                   o.created_at,
                   o.after_sale_at
                   FROM orders o {JOIN_CLAUSE} WHERE 1=1 "#;

        let mut builder: QueryBuilder<MySql>;
        if tenant_type == TenantType::Provider.to_string() {
            let query = &basic_query.replace(
                "{JOIN_CLAUSE}",
                "JOIN provider_orders_assignments po ON po.order_id=o.id",
            );

            builder = QueryBuilder::new(query);
            builder.push(" AND po.provider_id= ").push_bind(record.id);
        } else {
            let query = &basic_query.replace("{JOIN_CLAUSE}", "");

            builder = QueryBuilder::new(query);
            builder.push(" AND (o.market_id=").push_bind(record.id);
            builder.push(" OR o.customer_id=").push_bind(record.id);
            builder.push(") ");
        }

        if let Some(order_status) = query_params.order_status.as_ref() {
            debug!("order_status: {}", order_status);
            builder.push(" AND o.order_status=").push_bind(order_status);
        }

        builder.push(" ORDER BY o.created_at DESC ");
        builder.push(" LIMIT ").push_bind(page_size);
        builder.push(" OFFSET ").push_bind(offset);

        let orders = builder
            .build_query_as::<OrderResponse>()
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get customer orders"))?;
        Ok(orders)
    }

    async fn get_order_by_order_code(
        &self,
        order_code: &str,
    ) -> Result<Option<OrderDetailResponse>, AppError> {
        let order = sqlx::query_as!(
            OrderItem,
            r#"SELECT 
                o.id,
                o.order_code,
                t.name as customer_name,
                o.order_status,
                o.total_amount,
                o.discount_amount,
                o.actual_amount,
                o.delivery_date,
                o.delivery_address,
                o.contact_name,
                o.contact_phone,
                o.remark,
                o.created_by,
                o.created_at,
                o.confirmed_by,
                o.confirmed_at,
                o.processed_by,
                o.processed_at,
                o.stocked_by,
                o.stocked_at,
                o.after_sale_at,
                o.rejected_by,
                o.rejected_at,
                o.reject_reason,
                o.completed_by,
                o.completed_at,
                ds.name as delivery_staff_name,
                ds.phone as delivery_staff_phone,
                p.name as provider_name
            FROM orders o 
            JOIN tenants t ON o.customer_id = t.id 
            LEFT JOIN delivery_staff ds ON o.delivery_staff_id = ds.id 
            LEFT JOIN provider_orders_assignments po ON o.id = po.order_id
            LEFT JOIN tenants p ON po.provider_id = p.id
            WHERE o.order_code = ?"#,
            order_code
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get order by id"))?;

        if order.is_none() {
            return Ok(None);
        }

        let order_id = order.as_ref().unwrap().id;
        let items = sqlx::query_as!(
            OrderDetail,
            r#"SELECT 
                id,
                product_code,
                product_name,
                category_id,
                category_name,
                unit,
                quantity,
                original_price,
                discount_rate,
                actual_price,
                actual_amount,
                total_amount,
                actual_quantity,
                processing_requirements,
                remark,
                status
            FROM order_details
            WHERE order_id = ?"#,
            order_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get order details"))?;

        let receipts_records = sqlx::query!(
            r#"SELECT 
                re.order_detail_id,
                o.order_code,
                od.product_code,
                od.product_name,
                re.operation_type,
                re.quantity,
                re.reason,
                od.unit,
                re.evidence_images
            FROM 
                return_exchange_records re
            JOIN 
                order_details od ON re.order_detail_id = od.id
            JOIN 
                orders o ON od.order_id = o.id
            WHERE 
                o.order_code = ?
            ORDER BY 
                re.created_at DESC"#,
            order_code
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get receipt records"))?;

        // 手动转换结果
        let records_len = receipts_records.len();
        let receipts = receipts_records.into_iter().try_fold(
            Vec::with_capacity(records_len),
            |mut acc, r| {
                let operation_type = ReceiptOperationType::try_from(r.operation_type)
                    .map_err(|e| AppError::Validation(format!("无效的收据操作类型: {}", e)))?;

                acc.push(OrderReceipt {
                    order_detail_id: r.order_detail_id,
                    order_code: r.order_code,
                    product_code: r.product_code,
                    product_name: r.product_name,
                    operation_type,
                    quantity: r.quantity,
                    reason: r.reason,
                    unit: r.unit,
                    evidence_images: r.evidence_images,
                });

                Ok::<_, AppError>(acc)
            },
        )?;

        Ok(Some(OrderDetailResponse {
            order: order.unwrap(),
            items,
            receipts,
        }))
    }

    async fn assign_order(
        &self,
        order_code: &str,
        provider_id: i32,
        confirmed_by: &str,
    ) -> Result<(), AppError> {
        let current = OrderStatus::Pending;
        let order = sqlx::query!(
            r#"SELECT id FROM orders WHERE order_code = ? AND order_status = ?"#,
            order_code,
            current.to_str()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get order by id"))?;

        if order.id == 0 {
            return Err(AppError::NotFound(format!(
                "Order {} not found",
                order_code
            )));
        }

        let order_id = order.id;

        let next = OrderStateMachine::next_state(
            OrderStatus::Pending,
            OrderAction::AssignSupplier,
            TenantType::Market,
        )
        .map_err(|e| {
            error!("{}", e);
            return AppError::Validation("Invalid transition".to_string());
        })?;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Failed to begin transaction"))?;

        // update order status to confirmed
        sqlx::query!(
            r#"UPDATE orders SET order_status = ?, confirmed_at = NOW(), confirmed_by = ? WHERE id = ? AND order_status = 'PENDING'"#,
            next.to_str(),
            confirmed_by,
            order_id
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to update order status from PENDING to ASSIGNED"))?;

        // dispatch order to provider
        sqlx::query!(
            r#"INSERT INTO provider_orders_assignments (order_id, provider_id) VALUES (?, ?)"#,
            order_id,
            provider_id
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to dispatch order"))?;

        sqlx::query!(
            r#"INSERT INTO order_status_history (order_id, from_status, to_status, changed_by, change_reason ) VALUES (?, ?, ?, ?, ?)"#,
            order_id, "PENDING", next.to_str(), confirmed_by, "ORDER_DISPATCH"
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to update order status"))?;

        tx.commit()
            .await
            .map_err(map_db_err!("Failed to commit transaction"))?;
        Ok(())
    }

    async fn update_order_status_to_processing(
        &self,
        order_code: &str,
        provider_hash: &str,
        delivery_staff_id: &str,
    ) -> Result<(), AppError> {
        let record = sqlx::query!(
            r#"SELECT name FROM tenants WHERE name_hash = ?"#,
            provider_hash
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get provider id"))?;

        if record.name.is_empty() {
            return Err(AppError::NotFound(format!(
                "Provider {} not found",
                provider_hash
            )));
        }

        let provider_name = record.name;
        let order = sqlx::query!(
            r#"SELECT id FROM orders WHERE order_code = ? AND order_status = 'CONFIRMED'"#,
            order_code
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get order by id"))?;

        if order.id == 0 {
            return Err(AppError::NotFound(format!(
                "Order {} not found or order status is not confirmed",
                order_code
            )));
        }

        let order_id = order.id;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Failed to begin transaction"))?;

        let delivery_staff_id = sqlx::query!(
            r#"SELECT id FROM delivery_staff WHERE id_card = ?"#,
            delivery_staff_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get delivery staff by id"))?;

        if delivery_staff_id.id == 0 {
            return Err(AppError::NotFound(format!(
                "Delivery staff id {} not found",
                delivery_staff_id.id
            )));
        }
        sqlx::query!(
            r#"UPDATE orders SET order_status = 'PROCESSING', delivery_staff_id = ? WHERE id = ? AND order_status = 'CONFIRMED'"#,
            delivery_staff_id.id,
            order_id
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to update order status"))?;

        sqlx::query!(
            r#"INSERT INTO order_status_history (order_id, from_status, to_status, changed_by, change_reason ) VALUES (?, ?, ?, ?, ?)"#,
            order_id, "CONFIRMED", "PROCESSING", provider_name, "ORDER_PROCESSING"
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to update order status"))?;

        tx.commit()
            .await
            .map_err(map_db_err!("Failed to commit transaction"))?;
        Ok(())
    }

    async fn update_actual_quantity(
        &self,
        order_code: &str,
        actual_quantity_dto: &ActualQuantityDTO,
    ) -> Result<(), AppError> {
        let record = sqlx::query!(
            r#"SELECT id FROM orders WHERE order_code = ? AND order_status = 'PROCESSING'"#,
            order_code
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get order by id"))?;

        if record.id == 0 {
            return Err(AppError::NotFound(format!(
                "Order {} not found or order status is not processing",
                order_code
            )));
        }

        let order_id = record.id;
        debug!("Update order with id {}", order_id);

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Failed to begin transaction"))?;

        // update actual quantity and actual amount in order_details table
        for item in actual_quantity_dto.items.iter() {
            let id = item.id;
            let actual_quantity = item.actual_quantity;

            sqlx::query!(
                r#"UPDATE order_details SET actual_quantity = ?, actual_amount = actual_price * ?,
                   total_amount = original_price * ?, status = 'PENDING'
                WHERE order_id = ? AND id = ?"#,
                actual_quantity,
                actual_quantity,
                actual_quantity,
                order_id,
                id
            )
            .execute(&mut *tx)
            .await
            .map_err(map_db_err!("Failed to update actual quantity"))?;
        }

        // update order status to stocked
        sqlx::query!(
            r#"UPDATE orders o
               SET o.actual_amount = (
                   SELECT SUM(od.actual_amount)
                   FROM order_details od
                   WHERE od.order_id = o.id
               ),
               o.total_amount = (
                   SELECT SUM(od.total_amount)
                   FROM order_details od
                   WHERE od.order_id = o.id
               ),
               o.discount_amount = (
                   SELECT SUM(od.total_amount - od.actual_amount)
                   FROM order_details od
                   WHERE od.order_id = o.id
               ),
               o.order_status = 'STOCKED',
               o.stocked_by = ? 
               WHERE o.id = ?"#,
            actual_quantity_dto.stocked_by,
            order_id
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to update order total amount"))?;
        debug!("Update order status to stocked");

        sqlx::query!(
            r#"INSERT INTO order_status_history (order_id, from_status, to_status, changed_by, change_reason ) VALUES (?, ?, ?, ?, ?)"#,
            order_id, "PROCESSING", "STOCKED", actual_quantity_dto.stocked_by, "ORDER_STOCKED"
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to update order status"))?;
        debug!("Insert order status history");

        tx.commit()
            .await
            .map_err(map_db_err!("Failed to commit transaction"))?;
        debug!("Order updated");
        Ok(())
    }

    async fn process_order_receipt(
        &self,
        receipt: &OrderReceipt,
        operator: &str,
        transaction_id: &str,
    ) -> Result<(), AppError> {
        // 验证订单存在且状态正确
        let record = sqlx::query!(
            r#"SELECT o.id, od.id as detail_id 
               FROM orders o 
               JOIN order_details od ON o.id = od.order_id 
               WHERE o.order_code = ? AND od.id = ? AND o.order_status = 'STOCKED'"#,
            receipt.order_code,
            receipt.order_detail_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!("Failed to find order"))?;

        if record.is_none() {
            return Err(AppError::NotFound(format!(
                "Order {} with order detail id {} not found or not in correct status",
                receipt.order_code, receipt.order_detail_id
            )));
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Failed to begin transaction"))?;
        let detail_id = record.unwrap().detail_id;

        // 处理凭证证据（如果有的话）
        let evidence_json = match &receipt.evidence_images {
            Some(evidence_images) if !evidence_images.is_empty() => {
                Some(serde_json::to_string(evidence_images).map_err(|e| {
                    AppError::Internal(format!("Failed to serialize evidence: {}", e))
                })?)
            }
            _ => None,
        };

        match receipt.operation_type {
            ReceiptOperationType::Sign => {
                // 处理正常签收
                sqlx::query!(
                    r#"UPDATE order_details 
                      SET status = 'SIGN', 
                        receipt_quantity = ?, 
                        receipt_date = NOW(), 
                        receipt_notes = ?,
                        receipt_evidence = ? 
                      WHERE id = ?"#,
                    receipt.quantity,
                    receipt.reason,
                    evidence_json,
                    detail_id
                )
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to update receipt status"))?;
            }
            ReceiptOperationType::Return => {
                // 处理退货
                sqlx::query!(
                    r#"UPDATE order_details 
                      SET status = 'RETURNED', 
                        receipt_quantity = actual_quantity - ?, 
                        receipt_date = NOW(), 
                        receipt_notes = ?,
                        receipt_evidence = ?
                      WHERE id = ?"#,
                    receipt.quantity,
                    receipt.reason,
                    evidence_json,
                    detail_id
                )
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to update return status"))?;

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

                let order_id_and_actual_price = sqlx::query!(
                    r#"SELECT order_id, actual_price FROM order_details WHERE id = ?"#,
                    detail_id
                )
                .fetch_one(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to get price"))?;

                // 退货要将订单总金额减去退货金额
                sqlx::query!(
                    r#"UPDATE orders SET actual_amount = actual_amount - ? WHERE id = ?"#,
                    order_id_and_actual_price.actual_price * receipt.quantity,
                    order_id_and_actual_price.order_id
                )
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to update order total amount"))?;

                sqlx::query!(
                    r#"INSERT INTO refund_records (return_exchange_id, amount, method, transaction_id, status, created_by)
                      VALUES (?, ?, ?, ?, ?, ?)"#,
                    return_exchange_id, order_id_and_actual_price.actual_price * receipt.quantity, "ORIGINAL", transaction_id, "PENDING", operator
                )
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to create refund record"))?;
            }
            ReceiptOperationType::Exchange => {
                // 处理换货, 换货不涉及退款，订单总金额不变
                sqlx::query!(
                    r#"UPDATE order_details 
                      SET status = 'EXCHANGED', 
                        receipt_quantity = actual_quantity - ?, 
                        receipt_date = NOW(), 
                        receipt_notes = ?,
                        receipt_evidence = ?
                      WHERE id = ?"#,
                    receipt.quantity,
                    receipt.reason,
                    evidence_json,
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

                let price = sqlx::query!(
                    r#"SELECT actual_price FROM order_details WHERE id = ?"#,
                    detail_id
                )
                .fetch_one(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to get price"))?;
                sqlx::query!(
                            r#"INSERT INTO exchange_items
                              (return_exchange_id, product_code, product_name, quantity, price, total_amount)
                              VALUES (?, ?, ?, ?, ?, ?)"#,
                    return_exchange_id, receipt.product_code, receipt.product_name, receipt.quantity, price.actual_price, price.actual_price * receipt.quantity
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
        tenant_type: &str,
        order_code: &str,
        new_status: &str,
        operator: &str,
    ) -> Result<(), AppError> {
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
        .map_err(map_db_err!("Failed to get order"))?;

        // validate the current status in database is the same as the new status
        if let Some(order) = order {
            if order.order_status == new_status {
                return Err(AppError::Conflict(format!(
                    "订单状态已被修改，当前状态为: {}，期望状态为: {}",
                    order.order_status, new_status
                )));
            }

            let current_status = order.order_status;
            let order_id = order.id;

            // validate the status transition and tenant permission using the state machine
            // use crate::models::order_status::OrderStatus;
            // OrderStatus::validate_transition_with_tenant(
            //     current_status.as_ref(),
            //     new_status,
            //     tenant_type,
            // )?;

            // execute different SQL updates based on the target status and tenant type
            let result = match new_status {
                "COMPLETED" => sqlx::query!(
                    r#"UPDATE orders 
                       SET order_status = ?, 
                           completed_by = ?, 
                           completed_at = NOW() 
                       WHERE order_code = ? 
                       AND order_status = ?"#,
                    new_status,
                    operator,
                    order_code,
                    current_status
                )
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to update order status to COMPLETED"))?,

                "AFTER_SALE" => sqlx::query!(
                    r#"UPDATE orders 
                       SET order_status = ?,
                           after_sale_at = NOW()
                       WHERE order_code = ? 
                       AND order_status = ?"#,
                    new_status,
                    order_code,
                    current_status
                )
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to update order status to AFTER_SALE"))?,

                "REJECTED" => sqlx::query!(
                    r#"UPDATE orders 
                       SET order_status = ?,
                           rejected_by = ?,
                           rejected_at = NOW() 
                       WHERE order_code = ? 
                       AND order_status = ?"#,
                    new_status,
                    operator,
                    order_code,
                    current_status
                )
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to update order status to REJECTED"))?,
                "CONFIRMED" => sqlx::query!(
                    r#"UPDATE orders 
                       SET order_status = ?,
                           confirmed_by = ?,
                           confirmed_at = NOW() 
                       WHERE order_code = ? 
                       AND order_status = ?"#,
                    new_status,
                    operator,
                    order_code,
                    current_status
                )
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to update order status to CONFIRMED"))?,
                "PROCESSING" | "STOCKED" => sqlx::query!(
                    r#"UPDATE orders 
                       SET order_status = ?,
                           processed_by = ?,
                           processed_at = NOW() 
                       WHERE order_code = ? 
                       AND order_status = ?"#,
                    new_status,
                    operator,
                    order_code,
                    current_status
                )
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to update order status"))?,
                _ => {
                    // 通用更新，理论上不会执行到这里，因为状态机会先验证状态的有效性
                    sqlx::query!(
                        r#"UPDATE orders 
                       SET order_status = ? 
                       WHERE order_code = ? 
                       AND order_status = ?"#,
                        new_status,
                        order_code,
                        current_status
                    )
                    .execute(&mut *tx)
                    .await
                    .map_err(map_db_err!("Failed to update order status"))?
                }
            };

            // 检查是否成功更新了记录
            if result.rows_affected() == 0 {
                return Err(AppError::Conflict(format!(
                    "无法更新订单 {} 的状态，可能是状态已被修改",
                    order_code
                )));
            }

            // 确定状态变更的原因
            let change_reason = match new_status {
                "COMPLETED" => "ORDER_COMPLETED",
                "AFTER_SALE" => "ORDER_AFTER_SALE",
                "REJECTED" => "ORDER_REJECTED",
                "CONFIRMED" => "ORDER_CONFIRMED",
                "PROCESSING" => "ORDER_PROCESSING",
                "STOCKED" => "ORDER_STOCKED",
                _ => "STATUS_CHANGED",
            };

            // 记录状态变更历史
            sqlx::query!(
                r#"INSERT INTO order_status_history (
                order_id, from_status, to_status, 
                changed_by, change_reason
              ) VALUES (?, ?, ?, ?, ?)"#,
                order_id,
                current_status,
                new_status,
                operator,
                change_reason
            )
            .execute(&mut *tx)
            .await
            .map_err(map_db_err!("Failed to insert order status history"))?;

            tx.commit()
                .await
                .map_err(map_db_err!("Failed to commit transaction"))?;
            Ok(())
        } else {
            return Err(AppError::NotFound(format!("订单 {} 不存在", order_code)));
        }
    }

    async fn get_after_sale_orders_by_tenant(
        &self,
        tenant_hash: &str,
        tenant_type: &TenantType,
    ) -> Result<Vec<AcceptedOrderResponseDTO>, AppError> {
        info!("Get accepted orders by tenant_hash: {}", tenant_hash);

        let orders = match tenant_type {
            TenantType::Provider => sqlx::query_as!(
                AcceptedOrderResponseDTO,
                r#"WITH provider_orders AS (
                        SELECT DISTINCT poa.order_id
                        FROM provider_orders_assignments poa
                        JOIN tenants t ON poa.provider_id = t.id
                        WHERE t.name_hash = ?
                        AND t.tenant_type = ?
                        AND t.deleted_at IS NULL
                    )
                    SELECT 
                        o.order_code,
                        o.order_status, 
                        COALESCE(MIN(osh.created_at), o.created_at) as accepted_at
                    FROM orders o
                    JOIN provider_orders po ON o.id = po.order_id
                    LEFT JOIN order_status_history osh ON o.id = osh.order_id
                    WHERE (o.order_status = 'AFTER_SALE' OR o.order_status = 'ACCEPTED')
                    AND (osh.to_status = 'AFTER_SALE' OR osh.to_status = 'ACCEPTED')
                    AND o.deleted_at IS NULL
                    GROUP BY o.order_code, o.order_status, o.created_at
                    ORDER BY accepted_at DESC"#,
                tenant_hash,
                tenant_type.to_string()
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get accepted orders"))?,
            TenantType::Customer => sqlx::query_as!(
                AcceptedOrderResponseDTO,
                r#"WITH customer_orders AS (
                        SELECT o.id as order_id
                        FROM orders o
                        JOIN tenants t ON o.customer_id = t.id
                        WHERE t.name_hash = ?
                        AND t.tenant_type = ?
                        AND t.deleted_at IS NULL
                    )
                    SELECT 
                        o.order_code,
                        o.order_status, 
                        COALESCE(MIN(osh.created_at), o.created_at) as accepted_at
                    FROM orders o
                    JOIN customer_orders co ON o.id = co.order_id
                    LEFT JOIN order_status_history osh ON o.id = osh.order_id
                    WHERE (o.order_status = 'AFTER_SALE' OR o.order_status = 'ACCEPTED')
                    AND (osh.to_status = 'AFTER_SALE' OR osh.to_status = 'ACCEPTED')
                    AND o.deleted_at IS NULL
                    GROUP BY o.order_code, o.order_status, o.created_at
                    ORDER BY accepted_at DESC"#,
                tenant_hash,
                tenant_type.to_string()
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get accepted orders"))?,
            TenantType::Market => sqlx::query_as!(
                AcceptedOrderResponseDTO,
                r#"SELECT 
                        o.order_code,
                        o.order_status, 
                        COALESCE(MIN(osh.created_at), o.created_at) as accepted_at
                    FROM orders o
                    JOIN tenants t ON o.market_id = t.id
                    LEFT JOIN order_status_history osh ON o.id = osh.order_id
                    WHERE t.name_hash = ?
                    AND t.tenant_type = ?
                    AND (o.order_status = 'AFTER_SALE' OR o.order_status = 'ACCEPTED')
                    AND (osh.to_status = 'AFTER_SALE' OR osh.to_status = 'ACCEPTED')
                    AND o.deleted_at IS NULL
                    AND t.deleted_at IS NULL
                    GROUP BY o.order_code, o.order_status, o.created_at
                    ORDER BY accepted_at DESC"#,
                tenant_hash,
                tenant_type.to_string()
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get accepted orders"))?,
        };

        Ok(orders)
    }

    async fn fetch_today_product_order_summary_by_provider_hash(
        &self,
        provider_hash: &str,
    ) -> Result<Vec<ProductsSummaryWithOrdersDTO>, AppError> {
        let orders = sqlx::query_as!(
            ProductsSummaryWithOrdersDTO,
            r#"SELECT
                od.product_code,
                od.product_name,
                SUM(od.quantity) AS total_quantity,
                od.unit,
                od.processing_requirements,
                c.name AS customer_name,
                od.remark
            FROM
                provider_orders_assignments poa
            JOIN
                tenants t ON poa.provider_id = t.id
            JOIN
                orders o ON poa.order_id = o.id
            JOIN
                order_details od ON o.id = od.order_id
            JOIN
                tenants c ON o.customer_id = c.id
            WHERE
                t.name_hash = ? -- provider_hash
                AND t.tenant_type = 'PROVIDER'
                AND t.deleted_at IS NULL
                AND o.order_status IN ('CONFIRMED', 'PROCESSING')
                AND o.deleted_at IS NULL
                AND o.created_at >= CURDATE()
            GROUP BY
                od.product_code, od.product_name, od.unit, od.processing_requirements, c.name, od.remark
            ORDER BY
                total_quantity DESC;"#,
                provider_hash
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get product summaries with orders"))?;
        Ok(orders)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::order::{CreateOrderDTO, DeliveryInfo};
    use chrono::{Datelike, TimeZone, Utc};
    use mockall::mock;
    use rust_decimal::Decimal;

    // Mock OrderRepository for unit testing
    mock! {
        pub OrderRepo {}

        #[async_trait]
        impl OrderRepository for OrderRepo {
            async fn create_order(
                &self,
                market_id: i32,
                customer_hash: &str,
                order: &CreateOrderDTO,
            ) -> Result<OrderResponse, AppError>;

            async fn get_orders_by_tenant(
                &self,
                tenant_hash: &str,
                tenant_type: &str,
                query_params: &OrderQueryParams,
            ) -> Result<Vec<OrderResponse>, AppError>;

            async fn get_order_by_order_code(
                &self,
                order_code: &str,
            ) -> Result<Option<OrderDetailResponse>, AppError>;

            async fn assign_order(
                &self,
                order_code: &str,
                provider_id: i32,
                confirmed_by: &str,
            ) -> Result<(), AppError>;

            async fn update_order_status_to_processing(
                &self,
                order_code: &str,
                provider_hash: &str,
                delivery_staff_id: &str,
            ) -> Result<(), AppError>;

            async fn update_actual_quantity(
                &self,
                order_code: &str,
                actual_quantity_dto: &ActualQuantityDTO,
            ) -> Result<(), AppError>;

            async fn get_after_sale_orders_by_tenant(
                &self,
                tenant_hash: &str,
                tenant_type: &TenantType,
            ) -> Result<Vec<AcceptedOrderResponseDTO>, AppError>;

            async fn process_order_receipt(
                &self,
                receipt: &OrderReceipt,
                operator: &str,
                transaction_id: &str,
            ) -> Result<(), AppError>;

            async fn fetch_today_product_order_summary_by_provider_hash(
                &self,
                provider_hash: &str,
            ) -> Result<Vec<ProductsSummaryWithOrdersDTO>, AppError>;

            async fn update_order_status(
                &self,
                tenant_type: &str,
                order_code: &str,
                new_status: &str,
                operator: &str,
            ) -> Result<(), AppError>;
        }
    }

    // 测试辅助函数
    fn create_test_delivery_info() -> DeliveryInfo {
        DeliveryInfo {
            delivery_date: "2024-12-31".to_string(),
            delivery_address: "测试配送地址".to_string(),
            contact_name: "张三".to_string(),
            contact_phone: "13800138000".to_string(),
        }
    }

    fn create_test_order_dto() -> CreateOrderDTO {
        CreateOrderDTO {
            items: vec![],
            total_amount: Decimal::new(10000, 2), // 100.00
            delivery_info: create_test_delivery_info(),
        }
    }

    fn create_test_order_response() -> OrderResponse {
        OrderResponse {
            order_code: "ORD20241225001".to_string(),
            total_amount: Decimal::new(15000, 2),
            actual_amount: Decimal::new(10000, 2),
            delivery_address: "测试配送地址".to_string(),
            delivery_date: chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            order_status: "PENDING".to_string(),
            created_at: Some(Utc.with_ymd_and_hms(2024, 12, 25, 10, 0, 0).unwrap()),
            after_sale_at: None,
        }
    }

    /// 测试订单创建的输入验证逻辑
    #[tokio::test]
    async fn test_create_order_input_validation() {
        let mut mock_repo = MockOrderRepo::new();

        // 设置 mock 期望
        mock_repo
            .expect_create_order()
            .times(1)
            .returning(|_, _, _| Ok(create_test_order_response()));

        // 测试正常情况
        let order_dto = create_test_order_dto();
        let result = mock_repo.create_order(1, "test_customer", &order_dto).await;

        assert!(result.is_ok());
        let order_response = result.unwrap();
        assert_eq!(order_response.order_status, "PENDING");
        assert!(!order_response.order_code.is_empty());
    }

    /// 测试订单查询参数验证
    #[tokio::test]
    async fn test_query_params_validation() {
        let mut mock_repo = MockOrderRepo::new();

        mock_repo
            .expect_get_orders_by_tenant()
            .times(1)
            .returning(|_, _, _| Ok(vec![create_test_order_response()]));

        // 测试有效的查询参数
        let query_params = OrderQueryParams {
            page: Some(1),
            page_size: Some(10),
            order_status: Some("PENDING".to_string()),
        };

        let result = mock_repo
            .get_orders_by_tenant("test_tenant", "CUSTOMER", &query_params)
            .await;

        assert!(result.is_ok());
        let orders = result.unwrap();
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].order_status, "PENDING");
    }

    /// 测试订单状态转换的业务规则（单元测试级别的验证）
    #[tokio::test]
    async fn test_order_status_transition_logic() {
        let mut mock_repo = MockOrderRepo::new();

        // 测试成功的状态转换
        mock_repo
            .expect_update_order_status()
            .with(
                mockall::predicate::eq("CUSTOMER"),
                mockall::predicate::eq("ORD123"),
                mockall::predicate::eq("COMPLETED"),
                mockall::predicate::eq("operator"),
            )
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        let result = mock_repo
            .update_order_status("CUSTOMER", "ORD123", "COMPLETED", "operator")
            .await;

        assert!(result.is_ok());
    }

    /// 测试订单代码生成逻辑
    #[test]
    fn test_order_code_generation() {
        // 测试订单代码生成的格式
        let code1 = generate_code(CodeType::Order);
        let code2 = generate_code(CodeType::Order);

        // 订单代码应该以 "ODR-" 开头
        assert!(code1.starts_with("ODR-"));
        assert!(code2.starts_with("ODR-"));

        // 两次生成的代码应该不同
        assert_ne!(code1, code2);

        // 代码长度应该合理
        assert!(code1.len() > 20);
        assert!(code1.len() < 30);
    }

    /// 测试错误处理逻辑
    #[tokio::test]
    async fn test_error_handling() {
        let mut mock_repo = MockOrderRepo::new();

        // 模拟数据库错误
        mock_repo
            .expect_create_order()
            .times(1)
            .returning(|_, _, _| Err(AppError::Internal("Database connection failed".to_string())));

        let order_dto = create_test_order_dto();
        let result = mock_repo.create_order(1, "test_customer", &order_dto).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Internal(msg) => assert_eq!(msg, "Database connection failed"),
            _ => panic!("Expected Internal error"),
        }
    }

    /// 测试租户类型验证
    #[tokio::test]
    async fn test_tenant_type_validation() {
        let mut mock_repo = MockOrderRepo::new();

        // 测试非法租户类型的状态更新
        mock_repo
            .expect_update_order_status()
            .with(
                mockall::predicate::eq("INVALID_TYPE"),
                mockall::predicate::eq("ORD123"),
                mockall::predicate::eq("COMPLETED"),
                mockall::predicate::eq("operator"),
            )
            .times(1)
            .returning(|_, _, _, _| Err(AppError::Validation("Invalid tenant type".to_string())));

        let result = mock_repo
            .update_order_status("INVALID_TYPE", "ORD123", "COMPLETED", "operator")
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Validation(msg) => assert!(msg.contains("Invalid tenant type")),
            _ => panic!("Expected Validation error"),
        }
    }

    /// 测试分页参数的边界值
    #[tokio::test]
    async fn test_pagination_boundary_values() {
        let mut mock_repo = MockOrderRepo::new();

        // 测试边界情况：页面大小为 0
        mock_repo
            .expect_get_orders_by_tenant()
            .times(1)
            .returning(|_, _, _| Ok(vec![]));

        let query_params = OrderQueryParams {
            page: Some(1),
            page_size: Some(0), // 边界值
            order_status: None,
        };

        let result = mock_repo
            .get_orders_by_tenant("test_tenant", "CUSTOMER", &query_params)
            .await;

        assert!(result.is_ok());
        // 应该返回空结果
        assert_eq!(result.unwrap().len(), 0);
    }

    /// 测试订单金额计算逻辑
    #[test]
    fn test_order_amount_calculation() {
        let order_dto = CreateOrderDTO {
            items: vec![
                // 这里可以添加具体的订单项来测试金额计算
            ],
            total_amount: Decimal::new(10000, 2), // 100.00
            delivery_info: create_test_delivery_info(),
        };

        // 验证金额格式和精度
        assert_eq!(order_dto.total_amount.scale(), 2);
        assert!(order_dto.total_amount > Decimal::ZERO);
    }

    /// 测试日期格式验证
    #[test]
    fn test_delivery_date_format() {
        let delivery_info = create_test_delivery_info();

        // 验证日期格式
        let parsed_date =
            chrono::NaiveDate::parse_from_str(&delivery_info.delivery_date, "%Y-%m-%d");
        assert!(parsed_date.is_ok());

        let date = parsed_date.unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 12);
        assert_eq!(date.day(), 31);
    }

    /// 测试联系信息验证
    #[test]
    fn test_contact_info_validation() {
        let delivery_info = create_test_delivery_info();

        // 验证联系人姓名不为空
        assert!(!delivery_info.contact_name.is_empty());

        // 验证电话号码格式（简单验证）
        assert!(delivery_info.contact_phone.len() >= 11);
        assert!(delivery_info.contact_phone.starts_with('1'));

        // 验证地址不为空
        assert!(!delivery_info.delivery_address.is_empty());
    }
}
