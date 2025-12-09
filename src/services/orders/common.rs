use axum::{
    extract::{Path, Query},
    Extension,
    response::IntoResponse,
};

use tracing::info;
use chrono::NaiveDate;

use crate::{
    common::{ApiResponse, AppError},
    dto::order::{
        ExchangeAndReturnOrderDetailResponse, OrderQueryParams, OrderResponse, DashboardQueryParams,
    },
    middleware::context::RequestContext,
    models::claims::Claims,
    models::tenant_type::TenantType,
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

    Ok(Json(ApiResponse::new(Some(orders), &context)))
}

pub(crate) async fn get_order_by_order_code<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Path(order_code): Path<String>,
) -> Result<impl IntoResponse, AppError>
where
    T: CommonOrderRepository + Send + Sync,
{
    info!(
        "Getting order by order code {} for tenant {}",
        order_code, claims.tenant_hash
    );

    match TenantType::try_from(claims.tenant_type.as_str())? {
        TenantType::Provider => {
            let order = repo.order_by_order_code_for_provider(&order_code, &claims.tenant_hash).await?;
            return Ok(Json(ApiResponse::new(Some(serde_json::to_value(order)?), &context)));
        }
        TenantType::Market => {
            let order = repo.order_by_order_code_for_market(&order_code, &claims.tenant_hash).await?;
            return Ok(Json(ApiResponse::new(Some(serde_json::to_value(order)?), &context)));
        }
        TenantType::Customer => {
            let order = repo.order_by_order_code_for_customer(&order_code, &claims.tenant_hash).await?;
            return Ok(Json(ApiResponse::new(Some(serde_json::to_value(order)?), &context)));
        }
    }
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


pub(crate) async fn get_dashboard_stats<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Query(query_params): Query<DashboardQueryParams>,
) -> Result<impl IntoResponse, AppError>
where
    T: CommonOrderRepository + Send + Sync,
{
    // Parse date if provided
    let date = if let Some(date_str) = &query_params.date {
        Some(
            NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .map_err(|_| AppError::Validation("Invalid date format. Use YYYY-MM-DD".to_string()))?,
        )
    } else {
        None
    };

    match TenantType::try_from(claims.tenant_type.as_str())? {
        TenantType::Provider => {
            let stats = repo.get_provider_dashboard_stats(&claims.tenant_hash, date).await?;
            return Ok(Json(ApiResponse::new(Some(serde_json::to_value(stats)?), &context)));
        }
        TenantType::Market => {
            let stats = repo.get_market_dashboard_stats(&claims.tenant_hash).await?;
            return Ok(Json(ApiResponse::new(Some(serde_json::to_value(stats)?), &context)));
        }
        TenantType::Customer => {
            let stats = repo.get_customer_dashboard_stats(&claims.tenant_hash, date).await?;
            return Ok(Json(ApiResponse::new(Some(serde_json::to_value(stats)?), &context)));
        }
    }
}