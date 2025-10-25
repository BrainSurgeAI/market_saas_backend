use axum::{extract::Query, Extension};
use tracing::debug;

use crate::{
    common::{ApiResponse, AppError},
    dto::price::{AproxPriceParam, PriceAnnouncement, PriceQueryParams, PriceStatusPaginationParams},
    dto::products::ProductDailyPriceComparisonDTO,
    middleware::context::RequestContext,
    repositories::price_traits::PriceRepository,
    utils::validate_json_fmt::Json,
    models::claims::Claims
};

/// Get price announcements
///
/// This function fetches all price announcements from the database.
/// This is public API, no authentication required
/// # Parameters
///
/// * `query`: The query parameters for the price announcements.
/// PriceQueryParams includes:
/// - `date`: The date of the price announcements.
/// - `category_l1`: The level one category ID.
/// - `category_l3`: The level three category ID.
/// - `name`: The name of the product.
///
/// # Returns
///
/// A vector of `PriceAnnouncement` objects.
///
/// # Error
///
/// Returns an `AppError` if the database query fails.
pub(crate) async fn get_price_announcements<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Query(query): Query<PriceQueryParams>,
) -> Result<Json<ApiResponse<Vec<PriceAnnouncement>>>, AppError>
where
    T: PriceRepository + Send + Sync,
{
    if let Err(validation_errors) = query.validate() {
        return Err(AppError::Validation(validation_errors.to_string()));
    }

    let product_prices = repo
        .find_price_announcements_by_category_product_name_and_date(&query)
        .await?;
    Ok(Json(ApiResponse::new(Some(product_prices), &context)))
}

/// # 变更产品每日价格状态
///
/// 此接口用于变更产品每日价格的状态，可以将状态从 PENDING 变更为 APPROVE、REJECTED 或 PUBLISHED。
///
/// ## 请求路径
/// `PATCH /api/v1/tenants/{tenant_id}/products/aprox_price`
///
/// ## 路径参数
/// - `tenant_id`: 租户ID
/// - `product_code`: 产品编码
///
/// ## 请求体
/// - `status`: 要变更的状态，可选值为 APPROVE、REJECTED 或 PUBLISHED
///
/// ## 返回数据
/// 成功变更状态后返回空数据
///
/// ## 权限要求
/// 需要 `price:update` 权限

pub(crate) async fn aprox_price<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Json(query): Json<AproxPriceParam>,
) -> Result<Json<ApiResponse<()>>, AppError>
where
    T: PriceRepository + Send + Sync,
{
    debug!("AproxPrice: {:?}", query);
    repo.aprox_price(&query).await?;
    Ok(Json(ApiResponse::new(Some(()), &context)))
}


/// 获取产品价格
pub async fn get_pricer_published_prices<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<PriceStatusPaginationParams>,
) -> Result<Json<ApiResponse<Vec<ProductDailyPriceComparisonDTO>>>, AppError>
where
    T: PriceRepository + Send + Sync,
{
    let mut query_with_defaults = PriceStatusPaginationParams::default();
    query_with_defaults.merge_from(&query);

    debug!("合并后的查询参数: {:?}", query_with_defaults);
    let product_prices = repo
        .user_daily_price_comparison(
            &claims.username,
            &claims.roles[0],
            &query_with_defaults,
        )
        .await?;
    Ok(Json(ApiResponse::new(Some(product_prices), &context)))
}