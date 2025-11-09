use axum::{extract::Path, Extension};
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

    let next_status = repo
        .update_order_status(
            tenant_type,
            &claims.tenant_hash,
            &order_code,
            action,
            //   target_status,
            &claims.username,
        )
        .await?;

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
) -> Result<Json<ApiResponse<()>>, AppError>
where
    T: SharedMarketCustomerOrderRepository + Send + Sync,
{
    repo.process_order_receipt(
        &return_exchange_dto.receipt,
        &claims.real_name,
        &context.request_id,
        &claims.tenant_type,
    )
    .await?;
    Ok(Json(ApiResponse::new(Some(()), &context)))
}

/// Market or Customer begin to inspect the order
pub(crate) async fn begin_inspect_order<T>(
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

    let tenant_type = TenantType::try_from(claims.tenant_type.as_str())?;

    let action = match tenant_type {
        TenantType::Market => OrderAction::MarketInspect,
        TenantType::Customer => OrderAction::CustomerInspect,
        _ => {
            return Err(AppError::Forbidden(format!(
                "{} 不能执行验收操作",
                claims.tenant_type
            )));
        }
    };

    let next_status = repo
        .begin_inspect_order(&order_code, &claims, action)
        .await?;
    Ok(Json(ApiResponse::new(
        Some(String::from(next_status.to_str())),
        &context,
    )))
}

pub(crate) async fn begin_exchange_inspect_order<T>(
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
        TenantType::Market => OrderAction::MarketInspect,
        TenantType::Customer => OrderAction::CustomerInspect,
        _ => {
            return Err(AppError::Forbidden(format!(
                "{} 不能执行换货验收操作",
                claims.tenant_type
            )));
        }
    };

    debug!(
        "Begin exchange inspect order {} by {} action {}",
        &order_code, tenant_type, action
    );

    let next_status = repo
        .update_order_status(
            tenant_type,
            &claims.tenant_hash,
            &order_code,
            action,
            // target_status,
            &claims.username,
        )
        .await?;
    Ok(Json(ApiResponse::new(
        Some(String::from(next_status.to_str())),
        &context,
    )))
}

/// 市场和客户通过订单验收, 针对主订单
pub(crate) async fn accept_order<T>(
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
        TenantType::Market => OrderAction::MarketAccept,
        TenantType::Customer => OrderAction::Complete,
        _ => {
            return Err(AppError::Forbidden(format!(
                "{} 不能执行签收操作",
                claims.tenant_type
            )));
        }
    };

    debug!(
        "Accept order {} by {} action {}",
        &order_code, tenant_type, action
    );
    let next_status = repo
        .update_order_status(
            tenant_type,
            &claims.tenant_hash,
            &order_code,
            action,
            // target_status,
            &claims.username,
        )
        .await?;
    Ok(Json(ApiResponse::new(
        Some(String::from(next_status.to_str())),
        &context,
    )))
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

    let next_status = repo
        .update_order_status(
            tenant_type,
            &claims.tenant_hash,
            &order_code,
            action,
            &claims.username,
        )
        .await?;

    info!(
        "Order {} return request processed by {} action {}",
        order_code, tenant_type, action
    );

    Ok(Json(ApiResponse::new(
        Some(String::from(next_status.to_str())),
        &context,
    )))
}

pub(crate) async fn exchange_request<T>(
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
    let next_status = repo
        .update_order_status(
            tenant_type,
            &claims.tenant_hash,
            &order_code,
            action,
            //  target_status,
            &claims.username,
        )
        .await?;
    Ok(Json(ApiResponse::new(
        Some(String::from(next_status.to_str())),
        &context,
    )))
}
