use axum::{extract::Path, Extension, Json};

use crate::{
    common::{ApiResponse, AppError},
    dto::{
        discount::{
            CustomerDiscountCreateRequestDTO, CustomerDiscountResponseDTO,
            CustomerDiscountUpdateRequestDTO,
        },
        ValidatedJSON,
    },
    middleware::context::RequestContext,
    models::claims::Claims,
    repositories::discount_traits::DiscountRepository,
};

pub async fn get_customer_discount_list<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Path(tenant_hash): Path<String>,
) -> Result<Json<ApiResponse<Vec<CustomerDiscountResponseDTO>>>, AppError>
where
    T: DiscountRepository + Send + Sync,
{
    let categories = repo.fetch_customer_discount_list(&tenant_hash).await?;
    Ok(Json(ApiResponse::new(Some(categories), &context)))
}

pub async fn update_discount_by_tenant_and_category<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Path((tenant_hash, discount_id)): Path<(String, i64)>,
    ValidatedJSON(body): ValidatedJSON<CustomerDiscountUpdateRequestDTO>,
) -> Result<Json<ApiResponse<()>>, AppError>
where
    T: DiscountRepository + Send + Sync,
{
    if claims.tenant_type != "MARKET" {
        return Err(AppError::Auth(
            "Only market can perform this operation".to_string(),
        ));
    }

    repo.update_discount_by_tenant_and_category(
        &tenant_hash,
        discount_id,
        &body.discount_rate,
        &body.changed_by,
    )
    .await?;
    Ok(Json(ApiResponse::new(Some(()), &context)))
}

pub async fn create_discount_by_tenant_and_category<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Path(tenant_hash): Path<String>,
    ValidatedJSON(body): ValidatedJSON<CustomerDiscountCreateRequestDTO>,
) -> Result<Json<ApiResponse<()>>, AppError>
where
    T: DiscountRepository + Send + Sync,
{
    if claims.tenant_type != "MARKET" {
        return Err(AppError::Auth(
            "Only market can perform this operation".to_string(),
        ));
    }

    repo.create_discount_by_tenant_and_category(&tenant_hash, &body)
        .await?;
    Ok(Json(ApiResponse::new(Some(()), &context)))
}
