use axum::{extract::Path, Extension, response::IntoResponse};
use tracing::{debug, info};

use crate::{
    common::{ApiResponse, AppError},
    dto::{order::OrderReceiptDTO, ValidatedJSON},
    middleware::context::RequestContext,
    models::claims::Claims,
    models::{order_action::OrderAction, tenant_type::TenantType},
    repositories::orders::shared_market_customer::SharedMarketCustomerOrderRepository,
    utils::validate_json_fmt::Json,
};

/// Cancel order
pub(crate) async fn cancel_order<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Path(order_code): Path<String>,
) -> Result<Json<ApiResponse<String>>, AppError>
where
    T: SharedMarketCustomerOrderRepository + Send + Sync,
{
    let tenant_type = TenantType::try_from(claims.tenant_type.as_str())?;
    let action = match tenant_type {
        TenantType::Market => OrderAction::Cancel,
        TenantType::Customer => OrderAction::Cancel,
        _ => {
            return Err(AppError::Forbidden(format!(
                "{} 不能执行取消操作",
                claims.tenant_type
            )));
        }
    };

    debug!(
        "Cancel order {} by {} action {}",
        &order_code, tenant_type, action
    );

    let next_status = repo.update_order_status(&order_code, action, &claims).await?;

    info!(
        "Order {} cancelled by {} action {}",
        order_code, tenant_type, action
    );

    Ok(Json(ApiResponse::new(
        Some(String::from(next_status.to_str())),
        &context,
    )))
}

pub(crate) async fn inspect_sub_orders<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    ValidatedJSON(return_exchange_dto): ValidatedJSON<OrderReceiptDTO>,
) -> Result<Json<ApiResponse<String>>, AppError>
where
    T: SharedMarketCustomerOrderRepository + Send + Sync,
{
    let inspect_status = repo.insert_order_inspection_with_aftersales(
        &return_exchange_dto.receipt,
        &claims.real_name,
        &context.request_id,
        &claims.tenant_type,
    )
    .await?;
    Ok(Json(ApiResponse::new(Some(inspect_status), &context)))
}

/// Market or Customer begin to inspect the order
pub(crate) async fn start_order_inspection<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Path(order_code): Path<String>,
) -> Result<Json<ApiResponse<String>>, AppError>
where
    T: SharedMarketCustomerOrderRepository + Send + Sync,
{
    info!(
        "Inspect order {} by market {} user {}",
        &order_code, &claims.tenant_type, &claims.username
    );

    let next_status = repo.init_order_inspection_with_items(&order_code, &claims).await?;
    Ok(Json(ApiResponse::new(
        Some(String::from(next_status.to_str())),
        &context,
    )))
}

pub(crate) async fn get_order_inspections<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Path(order_code): Path<String>,
) -> Result<impl IntoResponse, AppError>
where
    T: SharedMarketCustomerOrderRepository + Send + Sync,
{
    let inspections = repo.get_order_inspections(&order_code, &claims).await?;
    Ok(Json(ApiResponse::new(Some(inspections), &context)))
}

pub(crate) async fn return_order<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Path(order_code): Path<String>,
) -> Result<Json<ApiResponse<String>>, AppError>
where
    T: SharedMarketCustomerOrderRepository + Send + Sync,
{
    let tenant_type = TenantType::try_from(claims.tenant_type.as_str())?;
    let action = match tenant_type {
        TenantType::Market => OrderAction::MarketReturn,
        TenantType::Customer => OrderAction::CustomerReturn,
        _ => {
            return Err(AppError::Forbidden(format!(
                "{} 不能执行退货操作",
                claims.tenant_type
            )));
        }
    };

    debug!(
        "Return order {} by {} action {}",
        &order_code, tenant_type, action
    );

    let next_status = repo.update_order_status(&order_code, action, &claims).await?;

    info!(
        "Order {} return request processed by {} action {}",
        order_code, tenant_type, action
    );

    Ok(Json(ApiResponse::new(
        Some(String::from(next_status.to_str())),
        &context,
    )))
}

pub(crate) async fn create_after_sales_request<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Path(order_code): Path<String>,
) -> Result<Json<ApiResponse<String>>, AppError>
where
    T: SharedMarketCustomerOrderRepository + Send + Sync,
{
    let tenant_type = TenantType::try_from(claims.tenant_type.as_str())?;
    let action = match tenant_type {
        TenantType::Market => OrderAction::MarketExchange,
        TenantType::Customer => OrderAction::CustomerExchange,
        _ => {
            return Err(AppError::Forbidden(format!(
                "{} 不能执行换货操作",
                claims.tenant_type
            )));
        }
    };

    debug!(
        "Exchange {} by {} action {}",
        &order_code, tenant_type, action
    );
    let next_status = repo.create_after_sales_request(&order_code, &claims, action, tenant_type).await?;
    Ok(Json(ApiResponse::new(
        Some(String::from(next_status.to_str())),
        &context,
    )))
}

pub(crate) async fn get_order_inspection_history<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Path(order_code): Path<String>,
) -> Result<impl IntoResponse, AppError>
where
    T: SharedMarketCustomerOrderRepository + Send + Sync,
{
    let inspection_history = repo.get_order_inspection_history(&order_code, &claims).await?;
    Ok(Json(ApiResponse::new(Some(serde_json::to_value(inspection_history)?), &context)))
}