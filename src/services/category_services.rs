use axum::Extension;

use crate::{
    common::{ApiResponse, AppError},
    dto::category::CategoryWithSubCategoriesDTO,
    middleware::context::RequestContext,
    repositories::category_traits::CategoryRepository,
    utils::validate_json_fmt::Json,
};

pub async fn get_categories_tree<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
) -> Result<Json<ApiResponse<Vec<CategoryWithSubCategoriesDTO>>>, AppError>
where
    T: CategoryRepository + Send + Sync,
{
    let categories = repo.list_categories_with_subcategories().await?;
    Ok(Json(ApiResponse::new(Some(categories), &context)))
}
