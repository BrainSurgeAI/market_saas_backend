use axum::{extract::Query, Extension};
use tracing::debug;

use crate::{
    common::{ApiResponse, AppError},
    dto::price::{AproxPriceParam, PriceAnnouncement, PriceQueryParams},
    middleware::context::RequestContext,
    repositories::price_traits::PriceRepository,
    utils::validate_json_fmt::Json,
};

/// # 每日价格公示数据接口
///
/// 此接口提供市场每日价格公示数据，支持按类别和产品名称筛选。
///
/// ## 请求路径
/// `GET /api/v1/fetch_price_announcements`
///
/// ## 查询参数
/// - `category_l1`: 可选，一级类别ID
/// - `category_l3`: 可选，三级类别ID
/// - `name`: 可选，产品名称（模糊匹配）
/// - `date`: 可选，价格日期，默认为当天
///
/// ## 返回数据
/// 返回符合条件的产品价格列表，包含类别、产品名称、价格区间等信息
///
/// ## 权限要求 无

pub async fn fetch_price_announcements<T>(
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

    let product_prices = repo.find_by_category_product_name_and_date(&query).await?;
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

pub async fn aprox_price<T>(
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
