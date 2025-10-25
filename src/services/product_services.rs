use crate::{
    common::{ApiResponse, AppError},
    dto::{
        products::{
            PriceStatusDTO, ProcessingFeeDTO,
            ProductDetailResponse, ProductListDTO, ProductListQueryParams, ProductOverviewResponse,
            PriceCreateDTO, UpdateProductRequestDTO,
        },
    },
    middleware::context::RequestContext,
    models::claims::Claims,
    repositories::product_traits::ProductRepository,
    utils::validate_json_fmt::Json,
};
use axum::{
    extract::{Path, Query},
    response::IntoResponse,
    Extension,
};
use tracing::debug;

// 获取产品的部分信息，主要用于产品管理
pub async fn get_products_overview<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Query(query): Query<ProductListQueryParams>,
) -> Result<Json<ApiResponse<ProductOverviewResponse>>, AppError>
where
    T: ProductRepository + Send + Sync,
{
    let products = repo.fetch_products_by_category_id(&query).await?;
    Ok(Json(ApiResponse::new(
        Some(ProductOverviewResponse {
            products: products.0,
            total: products.1,
        }),
        &context,
    )))
}


/// 批量创建产品价格
pub async fn batch_create_product_price<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Path(username): Path<String>,
    Json(product_prices): Json<Vec<PriceCreateDTO>>,
) -> Result<impl IntoResponse, AppError>
where
    T: ProductRepository + Send + Sync,
{
    debug!("product_prices: {:?}", product_prices);
    let result = repo
        .batch_create_product_price(&username, &product_prices)
        .await?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(ApiResponse::new(Some(result), &context)),
    ))
}

/// 获取当日价格状态统计
pub async fn get_product_price_status_stats<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>
) -> Result<Json<ApiResponse<Vec<PriceStatusDTO>>>, AppError>
where
    T: ProductRepository + Send + Sync,
{
    let stats = repo
        .fetch_product_price_status_stats(&claims.username, &claims.roles[0])
        .await?;
    Ok(Json(ApiResponse::new(Some(stats), &context)))
}

pub async fn get_product_list<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Path(tenant_hash): Path<String>,
    Query(query): Query<ProductListQueryParams>,
) -> Result<Json<ApiResponse<Vec<ProductListDTO>>>, AppError>
where
    T: ProductRepository + Send + Sync,
{
    let products = repo.get_product_list(&query, &tenant_hash).await?;
    Ok(Json(ApiResponse::new(Some(products), &context)))
}

/// 根据 tenant_hash 和 product_code 获取产品详情
/// Path: /api/v1/tenants/{tenant_hash}/products/{product_code} 中的tenant_hash是market_hash
/// 只有market类型租户可以维护产品信息，根据tenant_hash与claims.tenant_name对比，如果相同，则断定当前是market类型租户在访问
pub async fn get_product_detail<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Path((_tenant_hash, product_code)): Path<(String, String)>,
) -> Result<Json<ApiResponse<Option<ProductDetailResponse>>>, AppError>
where
    T: ProductRepository + Send + Sync,
{
    // TODO: 当前通过角色判断是否是产品的owner，后续需要根据tenant_hash来判断
    let is_owner = claims.roles[0] != "CUSTOMER";
    let product = repo.get_product_detail(&product_code, is_owner).await?;
    Ok(Json(ApiResponse::new(Some(product), &context)))
}

/// 更新产品状态
pub async fn update_product_status<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Path((_tenant_hash, product_code)): Path<(String, String)>,
) -> Result<Json<ApiResponse<()>>, AppError>
where
    T: ProductRepository + Send + Sync,
{
    repo.deactivate_product(&product_code).await?;
    Ok(Json(ApiResponse::new(Some(()), &context)))
}

/// 更新产品
pub async fn update_product<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Path((_tenant_hash, product_code)): Path<(String, String)>,
    Json(product): Json<UpdateProductRequestDTO>,
) -> Result<Json<ApiResponse<()>>, AppError>
where
    T: ProductRepository + Send + Sync,
{
    debug!("product: {:?}", product);
    repo.update_product(&product_code, &product).await?;
    Ok(Json(ApiResponse::new(Some(()), &context)))
}

/// 获取产品处理费用
pub async fn get_processing_fees<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
) -> Result<Json<ApiResponse<Vec<ProcessingFeeDTO>>>, AppError>
where
    T: ProductRepository + Send + Sync,
{
    let fees = repo.get_processing_fees().await?;
    Ok(Json(ApiResponse::new(Some(fees), &context)))
}
