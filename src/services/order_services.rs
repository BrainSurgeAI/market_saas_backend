use axum::{extract::Path, Extension};

use crate::{
    common::{ApiResponse, AppError},
    dto::order::{
        AcceptedOrderResponseDTO, ProductsSummaryWithOrdersDTO, ProviderDashboardStatsDTO,
    },
    middleware::context::RequestContext,
    models::{claims::Claims, tenant_type::TenantType},
    repositories::order_traits::OrderRepository,
    utils::validate_json_fmt::Json,
};

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

pub async fn get_provider_preparation_summary<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<Vec<ProductsSummaryWithOrdersDTO>>>, AppError>
where
    T: OrderRepository + Send + Sync,
{
    use tracing::debug;
    debug!("Getting preparation summary for tenant_hash: {}", claims.tenant_hash);
    let summary = repo
        .fetch_preparation_summary(&claims.tenant_hash)
        .await?;
    debug!("Preparation summary result count: {}", summary.len());
    Ok(Json(ApiResponse::new(Some(summary), &context)))
}

pub async fn get_provider_dashboard_stats<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<ProviderDashboardStatsDTO>>, AppError>
where
    T: OrderRepository + Send + Sync,
{
    use tracing::debug;
    debug!("Getting dashboard stats for tenant_hash: {}", claims.tenant_hash);
    let stats = repo
        .get_provider_dashboard_stats(&claims.tenant_hash)
        .await?;
    debug!("Dashboard stats retrieved successfully");
    Ok(Json(ApiResponse::new(Some(stats), &context)))
}
