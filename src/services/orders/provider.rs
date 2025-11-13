use axum::{extract::Path, Extension};
use tracing::debug;

use crate::{
    common::{ApiResponse, AppError},
    dto::{delivery_staff::DeliveryStaffIdDTO, order::{DeliverToMarketDTO, ExchangeDTO, ExchangeItemUpdateDTO}, ValidatedJSON},
    middleware::context::RequestContext,
    models::{claims::Claims, tenant_type::TenantType},
    repositories::orders::provider::ProviderOrderRepository,
    utils::validate_json_fmt::Json,
};

pub(crate) async fn start_preparing<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Path(order_code): Path<String>,
    ValidatedJSON(delivery_staff_id_dto): ValidatedJSON<DeliveryStaffIdDTO>,
) -> Result<Json<ApiResponse<String>>, AppError>
where
    T: ProviderOrderRepository + Send + Sync,
{
    let next_status = repo
        .mark_order_as_processing(&order_code, &claims, Some(delivery_staff_id_dto.id_card.as_str()))
        .await?;
    Ok(Json(ApiResponse::new(
        Some(next_status.to_str().to_string()),
        &context,
    )))
}

pub(crate) async fn start_exchange_preparing<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Path(order_code): Path<String>,
) -> Result<Json<ApiResponse<String>>, AppError>
where
    T: ProviderOrderRepository + Send + Sync,
{
    let next_status = repo
        .mark_order_as_processing(&order_code, &claims, None)
        .await?;
    Ok(Json(ApiResponse::new(
        Some(next_status.to_str().to_string()),
        &context,
    )))
}

pub(crate) async fn deliver_to_market<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Path(order_code): Path<String>,
    ValidatedJSON(deliver_to_market_dto): ValidatedJSON<DeliverToMarketDTO>,
) -> Result<Json<ApiResponse<()>>, AppError>
where
    T: ProviderOrderRepository + Send + Sync,
{
    debug!("Deliver to market: {:?}", deliver_to_market_dto);
    repo.deliver_to_market(&order_code, &claims.tenant_hash, &deliver_to_market_dto)
        .await?;
    Ok(Json(ApiResponse::new(Some(()), &context)))
}


pub(crate) async fn exchange_deliver_to_market<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Path(order_code): Path<String>,
    ValidatedJSON(exchange_dto): ValidatedJSON<ExchangeDTO>,
) -> Result<Json<ApiResponse<()>>, AppError>
where
    T: ProviderOrderRepository + Send + Sync,
{
    let tenant_type = TenantType::try_from(claims.tenant_type.as_str())?;
    if tenant_type != TenantType::Provider {
        return Err(AppError::Forbidden(format!(
            "{} 不能执行换货操作",
            claims.tenant_type
        )));
    }

    repo.exchange_deliver_to_market(
        &order_code,
        &claims.real_name,
        &claims.tenant_hash,
        &exchange_dto,
    )
    .await?;
    Ok(Json(ApiResponse::new(Some(()), &context)))
}

pub(crate) async fn update_exchange_item_actual_quantity<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    ValidatedJSON(exchange_item_update_dto): ValidatedJSON<ExchangeItemUpdateDTO>,
) -> Result<Json<ApiResponse<()>>, AppError>
where
    T: ProviderOrderRepository + Send + Sync,
{
    repo.update_exchange_item_actual_quantity(&claims.real_name, &exchange_item_update_dto)
        .await?;
    Ok(Json(ApiResponse::new(Some(()), &context)))
}