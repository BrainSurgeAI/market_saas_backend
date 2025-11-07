use axum::{extract::Path, Extension};
use tracing::info;

use crate::{
    common::{ApiResponse, AppError},
    dto::{
        order::CreateOrderDTO,
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
    ValidatedJSON(order): ValidatedJSON<CreateOrderDTO>,
) -> Result<Json<ApiResponse<String>>, AppError>
where
    T: CustomerOrderRepository + Send + Sync,
{
    let order_code = repo.create_order(&claims.tenant_hash, &order).await?;
    info!(
        "User {} create order {} success",
        claims.username, order_code
    );
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
