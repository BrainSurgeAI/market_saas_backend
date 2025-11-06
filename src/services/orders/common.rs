use axum::{
    extract::{Path, Query},
    Extension,
};
use tracing::{debug, info};

use crate::{
    common::{ApiResponse, AppError},
    dto::order::{
        ExchangeAndReturnOrderDetailResponse, OrderDetailResponse, OrderQueryParams, OrderResponse,
    },
    middleware::context::RequestContext,
    models::claims::Claims,
    repositories::orders::common::CommonOrderRepository,
    utils::validate_json_fmt::Json,
};

pub(crate) async fn get_orders<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Query(query_params): Query<OrderQueryParams>,
) -> Result<Json<ApiResponse<Vec<OrderResponse>>>, AppError>
where
    T: CommonOrderRepository + Send + Sync,
{
    info!(
        "Getting orders for tenant {} type {} with query params {:?}",
        claims.tenant_hash, claims.tenant_type, query_params
    );

    let orders = repo
        .get_orders_by_tenant(&claims.tenant_hash, &claims.tenant_type, &query_params)
        .await?;

    debug!("Found {} orders", orders.len());
    Ok(Json(ApiResponse::new(Some(orders), &context)))
}

pub(crate) async fn get_order_by_order_code<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Path(order_code): Path<String>,
) -> Result<Json<ApiResponse<Option<OrderDetailResponse>>>, AppError>
where
    T: CommonOrderRepository + Send + Sync,
{
    info!(
        "Getting order by order code {} for tenant {}",
        order_code, claims.tenant_hash
    );

    let order = repo
        .order_by_order_code(&order_code, &claims.tenant_hash)
        .await?;

    Ok(Json(ApiResponse::new(Some(order), &context)))
}

pub(crate) async fn get_exchange_and_return_order_details_by_order_code<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Path(order_code): Path<String>,
) -> Result<Json<ApiResponse<Vec<ExchangeAndReturnOrderDetailResponse>>>, AppError>
where
    T: CommonOrderRepository + Send + Sync,
{
    let order_details = repo
        .get_exchange_and_return_order_details_by_order_code(&order_code)
        .await?;
    Ok(Json(ApiResponse::new(Some(order_details), &context)))
}
