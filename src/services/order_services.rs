use axum::{
    extract::Path,
    Extension,
};

use crate::{
    common::{ApiResponse, AppError},
    dto::{
        order::{
            AcceptedOrderResponseDTO,
            ProductsSummaryWithOrdersDTO,
        },
    },
    middleware::context::RequestContext,
    models::{claims::Claims, tenant_type::TenantType},
    repositories::order_traits::OrderRepository,
    utils::validate_json_fmt::Json,
};

/// 客户创建订单
// pub(crate) async fn create_order<T>(
//     Extension(repo): Extension<T>,
//     Extension(context): Extension<RequestContext>,
//     Extension(claims): Extension<Claims>,
//     Json(order): Json<CreateOrderDTO>,
// ) -> Result<Json<ApiResponse<OrderResponse>>, AppError>
// where
//     T: OrderRepository + Send + Sync,
// {
//     let order_response = repo.create_order(&claims.tenant_hash, &order).await?;
//     info!(
//         "User {} create order {} success",
//         claims.username, order_response.order_code
//     );
//     Ok(Json(ApiResponse::new(Some(order_response), &context)))
// }

// pub(crate) async fn get_orders<T>(
//     Extension(repo): Extension<T>,
//     Extension(context): Extension<RequestContext>,
//     Extension(claims): Extension<Claims>,
//     Query(query_params): Query<OrderQueryParams>,
// ) -> Result<Json<ApiResponse<Vec<OrderResponse>>>, AppError>
// where
//     T: OrderRepository + Send + Sync,
// {
//     let orders = repo
//         .get_orders_by_tenant(&claims.tenant_hash, &claims.tenant_type, &query_params)
//         .await?;
//     Ok(Json(ApiResponse::new(Some(orders), &context)))
// }

// pub(crate) async fn get_order_by_order_code<T>(
//     Extension(repo): Extension<T>,
//     Extension(context): Extension<RequestContext>,
//     Extension(claims): Extension<Claims>,
//     Path(order_code): Path<String>,
// ) -> Result<Json<ApiResponse<Option<OrderDetailResponse>>>, AppError>
// where
//     T: OrderRepository + Send + Sync,
// {
//     let order = repo
//         .order_by_order_code(&order_code, &claims.tenant_hash)
//         .await?;
//     Ok(Json(ApiResponse::new(Some(order), &context)))
// }

// /// Dispatch order to provider
// pub(crate) async fn assign_order_to_provider<T>(
//     Extension(repo): Extension<T>,
//     Extension(context): Extension<RequestContext>,
//     Path(order_code): Path<String>,
//     Json(dispatch_order_dto): Json<DispatchOrderDTO>,
// ) -> Result<Json<ApiResponse<()>>, AppError>
// where
//     T: OrderRepository + Send + Sync,
// {
//     repo.assign_order(
//         &order_code,
//         dispatch_order_dto.provider_id,
//         &dispatch_order_dto.confirmed_by,
//     )
//     .await?;
//     Ok(Json(ApiResponse::new(Some(()), &context)))
// }

/// Update order status to processing
// pub(crate) async fn start_preparing<T>(
//     Extension(repo): Extension<T>,
//     Extension(context): Extension<RequestContext>,
//     Extension(claims): Extension<Claims>,
//     Path(order_code): Path<String>,
//     ValidatedJSON(delivery_staff_id_dto): ValidatedJSON<DeliveryStaffIdDTO>,
// ) -> Result<Json<ApiResponse<String>>, AppError>
// where
//     T: OrderRepository + Send + Sync,
// {
//     let next_status = repo
//         .order_start_progress(
//             &order_code,
//             &claims.username,
//             &claims.tenant_hash,
//             &delivery_staff_id_dto.id_card,
//         )
//         .await?;
//     Ok(Json(ApiResponse::new(
//         Some(next_status.to_str().to_string()),
//         &context,
//     )))
// }

// pub(crate) async fn start_exchange_preparing<T>(
//     Extension(repo): Extension<T>,
//     Extension(context): Extension<RequestContext>,
//     Extension(claims): Extension<Claims>,
//     Path(order_code): Path<String>,
// ) -> Result<Json<ApiResponse<String>>, AppError>
// where
//     T: OrderRepository + Send + Sync,
// {
//     let next_status = repo
//         .order_start_exchange_progress(&order_code, &claims.username, &claims.tenant_hash)
//         .await?;
//     Ok(Json(ApiResponse::new(
//         Some(next_status.to_str().to_string()),
//         &context,
//     )))
// }

// pub(crate) async fn deliver_to_market<T>(
//     Extension(repo): Extension<T>,
//     Extension(context): Extension<RequestContext>,
//     Extension(claims): Extension<Claims>,
//     Path(order_code): Path<String>,
//     Json(deliver_to_market_dto): Json<DeliverToMarketDTO>,
// ) -> Result<Json<ApiResponse<()>>, AppError>
// where
//     T: OrderRepository + Send + Sync,
// {
//     debug!("Deliver to market: {:?}", deliver_to_market_dto);
//     repo.deliver_to_market(
//         &order_code,
//         &claims.username,
//         &claims.tenant_hash,
//         &deliver_to_market_dto,
//     )
//     .await?;
//     Ok(Json(ApiResponse::new(Some(()), &context)))
// }

/// Get accepted orders by provider
pub async fn get_after_sale_orders_by_provider<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Path(tenant_hash): Path<String>,
) -> Result<Json<ApiResponse<Vec<AcceptedOrderResponseDTO>>>, AppError>
where
    T: OrderRepository + Send + Sync,
{
    let tenant_type = TenantType::try_from(claims.tenant_type.as_str())
        .map_err(|_| AppError::Validation("无效的租户类型".to_string()))?;
    let orders = repo
        .get_after_sale_orders_by_tenant(&tenant_hash, &tenant_type)
        .await?;
    Ok(Json(ApiResponse::new(Some(orders), &context)))
}

// pub async fn inspect_sub_orders<T>(
//     Extension(repo): Extension<T>,
//     Extension(context): Extension<RequestContext>,
//     Extension(claims): Extension<Claims>,
//     Json(return_exchange_dto): Json<OrderReceiptDTO>,
// ) -> Result<Json<ApiResponse<()>>, AppError>
// where
//     T: OrderRepository + Send + Sync,
// {
//     // let tenant_type = TenantType::try_from(claims.tenant_type.as_str())?;
//     // let action = match return_exchange_dto.receipt.operation_type {
//     //     ReceiptOperationType::Sign => {
//     //         if tenant_type == TenantType::Market {
//     //             OrderAction::MarketAccept
//     //         } else {
//     //             OrderAction::Complete
//     //         }
//     //     }
//     //     ReceiptOperationType::Return => {
//     //         if tenant_type == TenantType::Market {
//     //             OrderAction::MarketReturn
//     //         } else {
//     //             OrderAction::CustomerReturn
//     //         }
//     //     }
//     //     ReceiptOperationType::Exchange => {
//     //         if tenant_type == TenantType::Market {
//     //             OrderAction::MarketExchange
//     //         } else {
//     //             OrderAction::CustomerExchange
//     //         }
//     //     }
//     // };

//     repo.process_order_receipt(
//         &return_exchange_dto.receipt,
//         &return_exchange_dto.operate_by,
//         &context.request_id,
//     )
//     .await?;
//     Ok(Json(ApiResponse::new(Some(()), &context)))
// }

// /// Market or Customer begin to inspect the order
// pub(crate) async fn begin_inspect_order<T>(
//     Extension(repo): Extension<T>,
//     Extension(context): Extension<RequestContext>,
//     Extension(claims): Extension<Claims>,
//     Path(order_code): Path<String>,
//     //  Json(update_order_status_dto): Json<UpdateOrderStatusDTO>,
// ) -> Result<Json<ApiResponse<String>>, AppError>
// where
//     T: OrderRepository + Send + Sync,
// {
//     info!(
//         "Inspect order {} by market {} user {}",
//         &order_code, &claims.tenant_type, &claims.username
//     );

//     let tenant_type = TenantType::try_from(claims.tenant_type.as_str())?;
//     if tenant_type != TenantType::Market && tenant_type != TenantType::Customer {
//         return Err(AppError::Forbidden(format!(
//             "{} 不能执行验收操作",
//             claims.tenant_type
//         )));
//     }

//     let action = if tenant_type == TenantType::Market {
//         OrderAction::MarketInspect
//     } else {
//         OrderAction::CustomerInspect
//     };


//     let next_status = repo
//         .update_order_status(
//             tenant_type,
//             &claims.tenant_hash,
//             &order_code,
//             action,
//             // target_status,
//             &claims.username,
//         )
//         .await?;

//     Ok(Json(ApiResponse::new(
//         Some(String::from(next_status.to_str())),
//         &context,
//     )))
// }

// pub(crate) async fn begin_exchange_inspect_order<T>(
//     Extension(repo): Extension<T>,
//     Extension(context): Extension<RequestContext>,
//     Extension(claims): Extension<Claims>,
//     Path(order_code): Path<String>,
// ) -> Result<Json<ApiResponse<String>>, AppError>
// where
//     T: OrderRepository + Send + Sync,
// {
//     let tenant_type = TenantType::try_from(claims.tenant_type.as_str())?;
//     let action = match tenant_type {
//         TenantType::Market => OrderAction::MarketInspect,
//         TenantType::Customer => OrderAction::CustomerInspect,
//         _ => {
//             return Err(AppError::Forbidden(format!(
//                 "{} 不能执行换货验收操作",
//                 claims.tenant_type
//             )));
//         }
//     };

//     debug!(
//         "Begin exchange inspect order {} by {} action {}",
//         &order_code, tenant_type, action
//     );

//     let next_status = repo
//         .update_order_status(
//             tenant_type,
//             &claims.tenant_hash,
//             &order_code,
//             action,
//             // target_status,
//             &claims.username,
//         )
//         .await?;
//     Ok(Json(ApiResponse::new(
//         Some(String::from(next_status.to_str())),
//         &context,
//     )))
// }


// /// 市场和客户通过订单验收, 针对主订单
// pub(crate) async fn accept_order<T>(
//     Extension(repo): Extension<T>,
//     Extension(context): Extension<RequestContext>,
//     Extension(claims): Extension<Claims>,
//     Path(order_code): Path<String>,
// ) -> Result<Json<ApiResponse<String>>, AppError>
// where
//     T: OrderRepository + Send + Sync,
// {
//     let tenant_type = TenantType::try_from(claims.tenant_type.as_str())?;
//     let action = match tenant_type {
//         TenantType::Market => OrderAction::MarketAccept,
//         TenantType::Customer => OrderAction::Complete,
//         _ => {
//             return Err(AppError::Forbidden(format!(
//                 "{} 不能执行签收操作",
//                 claims.tenant_type
//             )));
//         }
//     };

//     debug!(
//         "Accept order {} by {} action {}",
//         &order_code, tenant_type, action
//     );
//     let next_status = repo
//         .update_order_status(
//             tenant_type,
//             &claims.tenant_hash,
//             &order_code,
//             action,
//             // target_status,
//             &claims.username,
//         )
//         .await?;
//     Ok(Json(ApiResponse::new(
//         Some(String::from(next_status.to_str())),
//         &context,
//     )))
// }

// pub(crate) async fn exchange_request<T>(
//     Extension(repo): Extension<T>,
//     Extension(context): Extension<RequestContext>,
//     Extension(claims): Extension<Claims>,
//     Path(order_code): Path<String>,
// ) -> Result<Json<ApiResponse<String>>, AppError>
// where
//     T: OrderRepository + Send + Sync,
// {
//     let tenant_type = TenantType::try_from(claims.tenant_type.as_str())?;
//     let action = match tenant_type {
//         TenantType::Market => OrderAction::MarketExchange,
//         TenantType::Customer => OrderAction::CustomerExchange,
//         _ => {
//             return Err(AppError::Forbidden(format!(
//                 "{} 不能执行换货操作",
//                 claims.tenant_type
//             )));
//         }
//     };

//     debug!(
//         "Exchange {} by {} action {}",
//         &order_code, tenant_type, action
//     );
//     let next_status = repo
//         .update_order_status(
//             tenant_type,
//             &claims.tenant_hash,
//             &order_code,
//             action,
//             //  target_status,
//             &claims.username,
//         )
//         .await?;
//     Ok(Json(ApiResponse::new(
//         Some(String::from(next_status.to_str())),
//         &context,
//     )))
// }

// pub(crate) async fn deliver_to_customer<T>(
//     Extension(repo): Extension<T>,
//     Extension(context): Extension<RequestContext>,
//     Extension(claims): Extension<Claims>,
//     Path(order_code): Path<String>,
// ) -> Result<Json<ApiResponse<String>>, AppError>
// where
//     T: OrderRepository + Send + Sync,
// {
//     let tenant_type = TenantType::try_from(claims.tenant_type.as_str())?;
//     let action = match tenant_type {
//         TenantType::Market => OrderAction::DeliverToCustomer,
//         _ => {
//             return Err(AppError::Forbidden(format!(
//                 "{} 不能执行配送到客户操作",
//                 claims.tenant_type
//             )));
//         }
//     };
//     debug!(
//         "Deliver to customer {} by {} action {}",
//         &order_code, tenant_type, action
//     );
//     let next_status = repo
//         .update_order_status(
//             tenant_type,
//             &claims.tenant_hash,
//             &order_code,
//             action,
//             //   target_status,
//             &claims.username,
//         )
//         .await?;
//     Ok(Json(ApiResponse::new(
//         Some(String::from(next_status.to_str())),
//         &context,
//     )))
// }

// /// Cancel order
// pub(crate) async fn cancel_order<T>(
//     Extension(repo): Extension<T>,
//     Extension(context): Extension<RequestContext>,
//     Extension(claims): Extension<Claims>,
//     Path(order_code): Path<String>,
// ) -> Result<Json<ApiResponse<String>>, AppError>
// where
//     T: OrderRepository + Send + Sync,
// {
//     let tenant_type = TenantType::try_from(claims.tenant_type.as_str())?;
//     let action = match tenant_type {
//         TenantType::Market => OrderAction::Cancel,
//         TenantType::Customer => OrderAction::Cancel,
//         _ => {
//             return Err(AppError::Forbidden(format!(
//                 "{} 不能执行取消操作",
//                 claims.tenant_type
//             )));
//         }
//     };

//     debug!(
//         "Cancel order {} by {} action {}",
//         &order_code, tenant_type, action
//     );

//     let next_status = repo
//         .update_order_status(
//             tenant_type,
//             &claims.tenant_hash,
//             &order_code,
//             action,
//             //   target_status,
//             &claims.username,
//         )
//         .await?;
//     Ok(Json(ApiResponse::new(
//         Some(String::from(next_status.to_str())),
//         &context,
//     )))
// }

pub async fn get_provider_today_product_order_summary<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Path(provider_hash): Path<String>,
) -> Result<Json<ApiResponse<Vec<ProductsSummaryWithOrdersDTO>>>, AppError>
where
    T: OrderRepository + Send + Sync,
{
    let summary = repo
        .fetch_today_product_order_summary_by_provider_hash(&provider_hash)
        .await?;
    Ok(Json(ApiResponse::new(Some(summary), &context)))
}

// pub(crate) async fn exchange_deliver_to_market<T>(
//     Extension(repo): Extension<T>,
//     Extension(context): Extension<RequestContext>,
//     Extension(claims): Extension<Claims>,
//     Path(order_code): Path<String>,
//     Json(exchange_dto): Json<ExchangeDTO>,
// ) -> Result<Json<ApiResponse<()>>, AppError>
// where
//     T: OrderRepository + Send + Sync,
// {
//     let tenant_type = TenantType::try_from(claims.tenant_type.as_str())?;
//     if tenant_type != TenantType::Provider {
//         return Err(AppError::Forbidden(format!(
//             "{} 不能执行换货操作",
//             claims.tenant_type
//         )));
//     }

//     repo.exchange_deliver_to_market(
//         &order_code,
//         &claims.username,
//         &claims.tenant_hash,
//         &exchange_dto,
//     )
//     .await?;
//     Ok(Json(ApiResponse::new(Some(()), &context)))
// }

// pub async fn get_exchange_and_return_order_details_by_order_code<T>(
//     Extension(repo): Extension<T>,
//     Extension(context): Extension<RequestContext>,
//     Path(order_code): Path<String>,
// ) -> Result<Json<ApiResponse<Vec<ExchangeAndReturnOrderDetailResponse>>>, AppError>
// where
//     T: OrderRepository + Send + Sync,
// {
//     let order_details = repo.get_exchange_and_return_order_details_by_order_code(&order_code).await?;
//     Ok(Json(ApiResponse::new(Some(order_details), &context)))
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::dto::order::{
//         CreateOrderDTO, CreateOrderItem, DeliveryInfo, OrderDetail, OrderDetailResponse, OrderItem,
//         OrderQueryParams, OrderReceipt, OrderResponse,
//     };
//     use crate::middleware::context::RequestContext;
//     use crate::models::{claims::Claims, order_status::OrderStatus};
//     use crate::utils::validate_json_fmt::Json;
//     use async_trait::async_trait;
//     use axum::extract::Path;
//     use chrono::NaiveDate;
//     use mockall::mock;
//     use mockall::predicate;
//     use mockall::predicate::*;
//     use rust_decimal::Decimal;

//     mock! {
//         pub OrderRepo {}

//         #[async_trait]
//         impl OrderRepository for OrderRepo {
//             async fn order_start_exchange_progress(&self, order_code: &str, operator: &str, provider_hash: &str) -> Result<OrderStatus, AppError>;
//             async fn exchange_deliver_to_market(&self, order_code: &str, operator: &str, provider_hash: &str, exchange_dto: &ExchangeDTO) -> Result<(), AppError>;
//             async fn get_after_sale_orders_by_tenant(&self, tenant_hash: &str, tenant_type: &TenantType) -> Result<Vec<crate::dto::order::AcceptedOrderResponseDTO>, AppError>;
//             async fn create_order(&self, customer_hash: &str, order: &CreateOrderDTO) -> Result<OrderResponse, AppError>;
//             async fn get_orders_by_tenant(&self, tenant_hash: &str, tenant_type: &str, query_params: &OrderQueryParams) -> Result<Vec<OrderResponse>, AppError>;
//             async fn order_by_order_code(&self, order_code: &str, tenant_hash: &str) -> Result<Option<OrderDetailResponse>, AppError>;
//             async fn assign_order(&self, order_code: &str, provider_id: i32, confirmed_by: &str) -> Result<(), AppError>;
//             async fn order_start_progress(&self, order_code: &str, operator: &str, provider_hash: &str, delivery_staff_id: &str) -> Result<OrderStatus, AppError>;
//             async fn deliver_to_market(&self, order_code: &str, operator: &str, provider_hash: &str, deliver_to_market_dto: &DeliverToMarketDTO) -> Result<(), AppError>;
//             async fn process_order_receipt(&self, receipt: &OrderReceipt, operator: &str, transaction_id: &str) -> Result<(), AppError>;
//             async fn update_order_status(&self, tenant_type: TenantType, tenant_hash: &str, order_code: &str, action: OrderAction, operator: &str) -> Result<OrderStatus, AppError>;
//             async fn fetch_today_product_order_summary_by_provider_hash(&self, provider_hash: &str) -> Result<Vec<ProductsSummaryWithOrdersDTO>, AppError>;
//         }
//     }

//     #[tokio::test]
//     async fn test_create_order_success() {
//         // 1. 准备测试数据

//         let delivery_info = DeliveryInfo {
//             delivery_date: "2023-05-15".to_string(),
//             delivery_address: "测试地址".to_string(),
//             contact_name: "测试用户".to_string(),
//             contact_phone: "13800138000".to_string(),
//         };

//         let items = vec![CreateOrderItem {
//             product_code: "P001".to_string(),
//             product_name: "测试产品".to_string(),
//             category_id: 1,
//             category_name: "测试分类".to_string(),
//             unit: "个".to_string(),
//             quantity: Decimal::new(2, 0),
//             price: Decimal::new(100, 0),
//             original_price: Decimal::new(120, 0),
//             original_amount: Decimal::new(240, 0),
//             discount_rate: Decimal::new(8, 1), // 0.8
//             total: Decimal::new(200, 0),
//             remark: None,
//             processing_services: vec![],
//         }];

//         let order = CreateOrderDTO {
//             total_amount: Decimal::new(200, 0),
//             delivery_info,
//             items,
//         };

//         // 2. 创建Mock实例和请求上下文
//         let mut mock_repo = MockOrderRepo::new();
//         let context = RequestContext {
//             request_id: "test-request-id".to_string(),
//             client_ip: None,
//         };

//         let claims = Claims {
//             tenant_type: "CUSTOMER".to_string(),
//             tenant_name: "测试客户".to_string(),
//             tenant_hash: "test_tenant_hash".to_string(),
//             username: "test_user".to_string(),
//             roles: vec!["CUSTOMER".to_string()],
//             exp: chrono::Utc::now()
//                 .checked_add_signed(chrono::Duration::days(1))
//                 .expect("Invalid timestamp")
//                 .timestamp() as usize,
//             is_super_admin: false,
//         };

//         // 3. 设置mock行为 - 模拟repository返回成功响应
//         mock_repo
//             .expect_create_order()
//             .with(eq("test_tenant_hash".to_string()), predicate::always())
//             .times(1)
//             .returning(|_, _| {
//                 // 返回模拟的成功响应
//                 Ok(OrderResponse {
//                     order_code: "ODR-20230515123456-ABCD".to_string(),
//                     total_amount: Decimal::new(240, 0),
//                     actual_amount: Decimal::new(200, 0),
//                     delivery_date: NaiveDate::from_ymd_opt(2023, 5, 15).unwrap(),
//                     delivery_address: "测试地址".to_string(),
//                     order_status: "PENDING".to_string(),
//                     created_at: None,
//                     after_sale_at: None,
//                 })
//             });

//         // 4. 执行service函数
//         let result = create_order(
//             Extension(mock_repo),
//             Extension(context),
//             Extension(claims),
//             Json(order),
//         )
//         .await;

//         // 5. 验证结果
//         assert!(result.is_ok());
//         let api_response = result.unwrap().0;
//         assert!(api_response.code == 200);

//         let order_response = api_response.data.unwrap();
//         assert!(order_response.order_code.starts_with("ODR-"));
//         assert_eq!(order_response.actual_amount, Decimal::new(200, 0));
//         assert_eq!(order_response.order_status, "PENDING");
//         assert_eq!(order_response.delivery_address, "测试地址");
//     }

//     #[tokio::test]
//     async fn test_create_order_customer_not_found() {
//         // 1. 准备测试数据

//         let order = CreateOrderDTO {
//             total_amount: Decimal::new(200, 0),
//             delivery_info: DeliveryInfo {
//                 delivery_date: "2023-05-15".to_string(),
//                 delivery_address: "测试地址".to_string(),
//                 contact_name: "测试用户".to_string(),
//                 contact_phone: "13800138000".to_string(),
//             },
//             items: vec![], // 简化测试
//         };

//         // 2. 创建Mock实例和请求上下文
//         let mut mock_repo = MockOrderRepo::new();
//         let context = RequestContext {
//             request_id: "test-request-id".to_string(),
//             client_ip: None,
//         };

//         let claims = Claims {
//             tenant_type: "CUSTOMER".to_string(),
//             tenant_name: "测试客户".to_string(),
//             tenant_hash: "test_tenant_hash".to_string(),
//             username: "test_user".to_string(),
//             roles: vec!["CUSTOMER".to_string()],
//             exp: chrono::Utc::now()
//                 .checked_add_signed(chrono::Duration::days(1))
//                 .expect("Invalid timestamp")
//                 .timestamp() as usize,
//             is_super_admin: false,
//         };

//         // 3. 设置mock行为 - 模拟客户不存在的情况
//         mock_repo
//             .expect_create_order()
//             .with(eq("test_tenant_hash".to_string()), predicate::always())
//             .times(1)
//             .returning(|hash, _| Err(AppError::NotFound(format!("Customer {} not found", hash))));

//         // 4. 执行service函数
//         let result = create_order(
//             Extension(mock_repo),
//             Extension(context),
//             Extension(claims),
//             Json(order),
//         )
//         .await;

//         // 5. 验证结果
//         assert!(result.is_err());
//         match result {
//             Err(AppError::NotFound(msg)) => {
//                 assert!(msg.contains("Customer"));
//                 assert!(msg.contains("not found"));
//             }
//             _ => panic!("Expected NotFound error"),
//         }
//     }

//     #[tokio::test]
//     async fn test_get_order_by_order_code_success() {
//         // 1. 准备测试数据

//         let order_code = "ODR-123456".to_string();

//         // 2. 创建Mock实例和请求上下文
//         let mut mock_repo = MockOrderRepo::new();
//         let context = RequestContext {
//             request_id: "test-request-id".to_string(),
//             client_ip: None,
//         };
//         let claims = Claims {
//             tenant_name: "测试客户".to_string(),
//             roles: vec!["CUSTOMER".to_string()],
//             is_super_admin: false,
//             username: "test_user".to_string(),
//             tenant_hash: "test_tenant_hash".to_string(),
//             tenant_type: "CUSTOMER".to_string(),
//             exp: 9999999999,
//         };

//         // 3. 设置mock行为 - 模拟订单查询成功
//         mock_repo
//             .expect_order_by_order_code()
//             .with(eq(order_code.clone()), eq("test_tenant_hash".to_string()))
//             .times(1)
//             .returning(|_, _| {
//                 Ok(Some(OrderDetailResponse {
//                     order: OrderItem {
//                         id: 1,
//                         order_code: "ODR-123456".to_string(),
//                         customer_name: "测试客户".to_string(),
//                         order_status: "PENDING".to_string(),
//                         total_amount: Decimal::new(240, 0),
//                         discount_amount: Decimal::new(40, 0),
//                         actual_amount: Decimal::new(200, 0),
//                         delivery_date: NaiveDate::from_ymd_opt(2023, 5, 15).unwrap(),
//                         delivery_address: "测试地址".to_string(),
//                         contact_name: "测试用户".to_string(),
//                         contact_phone: "13800138000".to_string(),
//                         remark: None,
//                         created_by: "test-user".to_string(),
//                         created_at: chrono::Utc::now(),
//                         confirmed_by: None,
//                         confirmed_at: None,
//                         processed_by: None,
//                         processed_at: None,
//                         stocked_by: None,
//                         stocked_at: None,
//                         after_sale_at: None,
//                         completed_by: None,
//                         completed_at: None,
//                         rejected_by: None,
//                         rejected_at: None,
//                         reject_reason: None,
//                         delivery_staff_name: None,
//                         delivery_staff_phone: None,
//                         provider_name: None,
//                     },
//                     items: vec![OrderDetail {
//                         id: 1,
//                         product_code: "P001".to_string(),
//                         product_name: "测试产品".to_string(),
//                         category_id: 1,
//                         category_name: "测试分类".to_string(),
//                         unit: "个".to_string(),
//                         quantity: Decimal::new(2, 0),
//                         original_price: Decimal::new(120, 0),
//                         discount_rate: Decimal::new(8, 1),
//                         actual_price: Decimal::new(100, 0),
//                         actual_quantity: None,
//                         actual_amount: None,
//                         total_amount: Decimal::new(200, 0),
//                         processing_requirements: None,
//                         remark: None,
//                         status: None,
//                         receipt_quantity: None,
//                     }],
//                     receipts: vec![],
//                 }))
//             });

//         // 4. 执行service函数
//         let result = get_order_by_order_code(
//             Extension(mock_repo),
//             Extension(context),
//             Extension(claims),
//             Path(order_code),
//         )
//         .await;

//         // 5. 验证结果
//         assert!(result.is_ok());
//         let api_response = result.unwrap().0;
//         assert!(api_response.code == 200);

//         let order_detail = api_response.data.unwrap().unwrap();
//         assert_eq!(order_detail.order.order_code, "ODR-123456");
//         assert_eq!(order_detail.order.order_status, "PENDING");
//         assert_eq!(order_detail.items.len(), 1);
//     }

//     #[tokio::test]
//     async fn test_create_order_with_provider_forbidden() {
//         // 测试 PROVIDER 类型租户尝试创建订单（应该失败）
//         let order = CreateOrderDTO {
//             total_amount: Decimal::new(200, 0),
//             delivery_info: DeliveryInfo {
//                 delivery_date: "2023-05-15".to_string(),
//                 delivery_address: "测试地址".to_string(),
//                 contact_name: "测试用户".to_string(),
//                 contact_phone: "13800138000".to_string(),
//             },
//             items: vec![],
//         };

//         let context = RequestContext {
//             request_id: "test-request-id".to_string(),
//             client_ip: None,
//         };

//         let claims = Claims {
//             tenant_type: "PROVIDER".to_string(),
//             tenant_name: "测试供应商".to_string(),
//             tenant_hash: "test_provider_hash".to_string(),
//             username: "test_provider".to_string(),
//             roles: vec!["PROVIDER".to_string()],
//             exp: chrono::Utc::now()
//                 .checked_add_signed(chrono::Duration::days(1))
//                 .expect("Invalid timestamp")
//                 .timestamp() as usize,
//             is_super_admin: false,
//         };

//         let mock_repo = MockOrderRepo::new();

//         let result = create_order(
//             Extension(mock_repo),
//             Extension(context),
//             Extension(claims),
//             Json(order),
//         )
//         .await;

//         // 验证结果应该是权限错误
//         assert!(result.is_err());
//         match result {
//             Err(AppError::Forbidden(msg)) => {
//                 assert!(msg.contains("Only customer type can create order"));
//             }
//             _ => panic!("Expected Forbidden error"),
//         }
//     }

//     #[tokio::test]
//     async fn test_create_order_with_market_forbidden() {
//         // 测试 MARKET 类型租户尝试创建订单（应该失败）
//         let order = CreateOrderDTO {
//             total_amount: Decimal::new(200, 0),
//             delivery_info: DeliveryInfo {
//                 delivery_date: "2023-05-15".to_string(),
//                 delivery_address: "测试地址".to_string(),
//                 contact_name: "测试用户".to_string(),
//                 contact_phone: "13800138000".to_string(),
//             },
//             items: vec![],
//         };

//         let context = RequestContext {
//             request_id: "test-request-id".to_string(),
//             client_ip: None,
//         };

//         let claims = Claims {
//             tenant_type: "MARKET".to_string(),
//             tenant_name: "测试市场".to_string(),
//             tenant_hash: "test_market_hash".to_string(),
//             username: "test_market".to_string(),
//             roles: vec!["MARKET".to_string()],
//             exp: chrono::Utc::now()
//                 .checked_add_signed(chrono::Duration::days(1))
//                 .expect("Invalid timestamp")
//                 .timestamp() as usize,
//             is_super_admin: false,
//         };

//         let mock_repo = MockOrderRepo::new();

//         let result = create_order(
//             Extension(mock_repo),
//             Extension(context),
//             Extension(claims),
//             Json(order),
//         )
//         .await;

//         // 验证结果应该是权限错误
//         assert!(result.is_err());
//         match result {
//             Err(AppError::Forbidden(msg)) => {
//                 assert!(msg.contains("Only customer type can create order"));
//             }
//             _ => panic!("Expected Forbidden error"),
//         }
//     }

//     #[tokio::test]
//     async fn test_create_order_with_invalid_tenant_type() {
//         // 测试无效的租户类型（应该失败）
//         let order = CreateOrderDTO {
//             total_amount: Decimal::new(200, 0),
//             delivery_info: DeliveryInfo {
//                 delivery_date: "2023-05-15".to_string(),
//                 delivery_address: "测试地址".to_string(),
//                 contact_name: "测试用户".to_string(),
//                 contact_phone: "13800138000".to_string(),
//             },
//             items: vec![],
//         };

//         let context = RequestContext {
//             request_id: "test-request-id".to_string(),
//             client_ip: None,
//         };

//         let claims = Claims {
//             tenant_type: "INVALID_TYPE".to_string(),
//             tenant_name: "测试无效类型".to_string(),
//             tenant_hash: "test_invalid_hash".to_string(),
//             username: "test_invalid".to_string(),
//             roles: vec!["INVALID".to_string()],
//             exp: chrono::Utc::now()
//                 .checked_add_signed(chrono::Duration::days(1))
//                 .expect("Invalid timestamp")
//                 .timestamp() as usize,
//             is_super_admin: false,
//         };

//         let mock_repo = MockOrderRepo::new();

//         let result = create_order(
//             Extension(mock_repo),
//             Extension(context),
//             Extension(claims),
//             Json(order),
//         )
//         .await;

//         // 验证结果应该是验证错误（无效的租户类型）
//         assert!(result.is_err());
//         match result {
//             Err(AppError::Validation(msg)) => {
//                 assert!(msg.contains("Unknown tenant type"));
//             }
//             _ => panic!("Expected Validation error"),
//         }
//     }

//     #[tokio::test]
//     async fn test_create_order_with_customer_success() {
//         // 测试 CUSTOMER 类型租户成功创建订单（权限验证通过）
//         let order = CreateOrderDTO {
//             total_amount: Decimal::new(200, 0),
//             delivery_info: DeliveryInfo {
//                 delivery_date: "2023-05-15".to_string(),
//                 delivery_address: "测试地址".to_string(),
//                 contact_name: "测试用户".to_string(),
//                 contact_phone: "13800138000".to_string(),
//             },
//             items: vec![],
//         };

//         let context = RequestContext {
//             request_id: "test-request-id".to_string(),
//             client_ip: None,
//         };

//         let claims = Claims {
//             tenant_type: "CUSTOMER".to_string(),
//             tenant_name: "测试客户".to_string(),
//             tenant_hash: "test_customer_hash".to_string(),
//             username: "test_customer".to_string(),
//             roles: vec!["CUSTOMER".to_string()],
//             exp: chrono::Utc::now()
//                 .checked_add_signed(chrono::Duration::days(1))
//                 .expect("Invalid timestamp")
//                 .timestamp() as usize,
//             is_super_admin: false,
//         };

//         let mut mock_repo = MockOrderRepo::new();
//         mock_repo
//             .expect_create_order()
//             .with(eq("test_customer_hash".to_string()), predicate::always())
//             .times(1)
//             .returning(|_, _| {
//                 Ok(OrderResponse {
//                     order_code: "ODR-20230515123456-TEST".to_string(),
//                     total_amount: Decimal::new(200, 0),
//                     actual_amount: Decimal::new(200, 0),
//                     delivery_date: NaiveDate::from_ymd_opt(2023, 5, 15).unwrap(),
//                     delivery_address: "测试地址".to_string(),
//                     order_status: "PENDING".to_string(),
//                     created_at: None,
//                     after_sale_at: None,
//                 })
//             });

//         let result = create_order(
//             Extension(mock_repo),
//             Extension(context),
//             Extension(claims),
//             Json(order),
//         )
//         .await;

//         // 验证结果成功
//         assert!(result.is_ok());
//         let api_response = result.unwrap().0;
//         assert_eq!(api_response.code, 200);
//         let order_response = api_response.data.unwrap();
//         assert_eq!(order_response.order_status, "PENDING");
//         assert_eq!(order_response.order_code, "ODR-20230515123456-TEST");
//     }
// }
