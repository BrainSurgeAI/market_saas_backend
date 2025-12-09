use axum::{extract::{Path, Query}, Extension};

use chrono::NaiveDate;
use tracing::debug;

use crate::{
    common::{ApiResponse, AppError},
    dto::{
        order::{CreateOrderRequestDTO, DashboardQueryParams, DashboardStatsDTO},
        ValidatedJSON,
    },
    middleware::context::RequestContext,
    models::{claims::Claims, tenant_type::TenantType},
    repositories::orders::customer::CustomerOrderRepository,
    utils::validate_json_fmt::Json,
};

pub(crate) async fn create_order<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    ValidatedJSON(order): ValidatedJSON<CreateOrderRequestDTO>,
) -> Result<Json<ApiResponse<String>>, AppError>
where
    T: CustomerOrderRepository + Send + Sync,
{
    debug!("Create order: {:?}", order);

    let order_code = repo.create_order(&claims, &order).await?;
    Ok(Json(ApiResponse::created(Some(order_code), &context)))
}

#[allow(dead_code)]
pub(crate) async fn exchange_deliver_to_market<T>(
    Extension(_repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Path(_order_code): Path<String>,
  //  ValidatedJSON(exchange_dto): ValidatedJSON<ExchangeDTO>,
) -> Result<Json<ApiResponse<()>>, AppError>
where
    T: CustomerOrderRepository + Send + Sync,
{
    let tenant_type = TenantType::try_from(claims.tenant_type.as_str())?;
    if tenant_type != TenantType::Customer {
        return Err(AppError::Forbidden(format!(
            "{} 不能执行换货操作",
            claims.tenant_type
        )));
    }

    
    Ok(Json(ApiResponse::new(Some(()), &context)))
}

pub(crate) async fn get_dashboard_stats<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Query(query_params): Query<DashboardQueryParams>,
) -> Result<Json<ApiResponse<DashboardStatsDTO>>, AppError>
where
    T: CustomerOrderRepository + Send + Sync,
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

    let stats = repo.get_customer_dashboard_stats(&claims, date).await?;
    Ok(Json(ApiResponse::new(Some(stats), &context)))
}
