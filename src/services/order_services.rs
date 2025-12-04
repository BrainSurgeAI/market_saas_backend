use axum::{extract::Path, extract::Query, Extension};
use serde::Deserialize;

use crate::{
    common::{ApiResponse, AppError},
    dto::order::{
        AcceptedOrderResponseDTO, ProductsSummaryWithOrdersDTO, ProviderDashboardStatsDTO,
        ProviderTodayDeliveredProductsDTO,
    },
    middleware::context::RequestContext,
    models::{claims::Claims, tenant_type::TenantType},
    repositories::order_traits::OrderRepository,
    repositories::orders::provider::ProviderOrderRepository,
    utils::validate_json_fmt::Json,
};

/// Dashboard 统计查询参数
#[derive(Debug, Deserialize)]
pub(crate) struct DashboardStatsQueryParams {
    #[serde(rename = "deliveryDate")]
    pub(crate) delivery_date: Option<chrono::NaiveDate>,
}

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
    let summary = repo
        .fetch_preparation_summary(&claims.tenant_hash)
        .await?;
    
    Ok(Json(ApiResponse::new(Some(summary), &context)))
}

pub async fn get_provider_dashboard_stats<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Query(query_params): Query<DashboardStatsQueryParams>,
) -> Result<Json<ApiResponse<ProviderDashboardStatsDTO>>, AppError>
where
    T: ProviderOrderRepository + Send + Sync,
{
    use tracing::debug;
    debug!(
        "Getting dashboard stats for tenant_hash: {}, delivery_date: {:?}",
        claims.tenant_hash, query_params.delivery_date
    );
    let stats = repo
        .get_provider_dashboard_stats(&claims.tenant_hash, query_params.delivery_date)
        .await?;
    debug!("Dashboard stats retrieved successfully");
    Ok(Json(ApiResponse::new(Some(stats), &context)))
}

pub async fn get_provider_today_delivered_products<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<Vec<ProviderTodayDeliveredProductsDTO>>>, AppError>
where
    T: ProviderOrderRepository + Send + Sync,
{
    use tracing::debug;
    debug!(
        "Getting today delivered products for tenant_hash: {}",
        claims.tenant_hash
    );
    let products = repo
        .get_provider_today_delivered_products(&claims.tenant_hash)
        .await?;
    debug!("Found {} delivered products for today", products.len());
    Ok(Json(ApiResponse::new(Some(products), &context)))
}
