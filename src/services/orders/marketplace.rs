use axum::{extract::Path, Extension};
use tracing::{debug, info};

use crate::{
    common::{ApiResponse, AppError},
    dto::{order::DispatchOrderDTO, ValidatedJSON},
    middleware::context::RequestContext,
    models::{claims::Claims, order_action::OrderAction, tenant_type::TenantType},
    repositories::orders::{
        marketplace::MarketplaceOrderRepository,
        shared_market_customer::SharedMarketCustomerOrderRepository,
    },
    utils::validate_json_fmt::Json,
};

/// Dispatch order to provider
pub(crate) async fn assign_order_to_provider<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Path(order_code): Path<String>,
    ValidatedJSON(dispatch_order_dto): ValidatedJSON<DispatchOrderDTO>,
) -> Result<Json<ApiResponse<()>>, AppError>
where
    T: MarketplaceOrderRepository + Send + Sync,
{
    debug!("Assign order to provider: {:?}", dispatch_order_dto);

    repo.assign_order(
        &order_code,
        dispatch_order_dto.provider_id,
        &dispatch_order_dto.confirmed_by,
    )
    .await?;

    info!(
        "Order {} assigned to provider {}",
        order_code, dispatch_order_dto.provider_id
    );

    Ok(Json(ApiResponse::new(Some(()), &context)))
}

pub(crate) async fn deliver_to_customer<T>(
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
        TenantType::Market => OrderAction::DeliverToCustomer,
        _ => {
            return Err(AppError::Forbidden(format!(
                "{} 不能执行配送到客户操作",
                claims.tenant_type
            )));
        }
    };

    debug!(
        "Deliver to customer {} by {} action {}",
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
        "Order {} delivered to customer by {} action {}",
        order_code, tenant_type, action
    );

    Ok(Json(ApiResponse::new(
        Some(String::from(next_status.to_str())),
        &context,
    )))
}
