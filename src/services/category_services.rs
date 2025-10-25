use axum::Extension;

use crate::{
    common::{ApiResponse, AppError},
    dto::category::CategoryWithSubCategoriesDTO,
    middleware::context::RequestContext,
    repositories::category_traits::CategoryRepository,
    utils::validate_json_fmt::Json,
};

/// Retrieve the hierarchical category tree with subcategories.
///
/// This endpoint fetches all categories and their nested subcategories from the database,
/// constructing a tree structure for frontend rendering. It is designed for authenticated
/// requests where the user has read access to categories.
///
/// # Parameters
///
/// * `repo` - The category repository implementation for data access.
/// * `context` - The request context containing user information and tracing spans.
///
/// # Returns
///
/// A JSON response containing a vector of [`CategoryWithSubCategoriesDTO`] objects
/// wrapped in an [`ApiResponse`].
///
/// # Errors
///
/// Returns an `AppError` if the repository query fails (e.g., database connection error)
/// or if the user lacks permission (handled via middleware).
///
/// # Examples
///
/// ```http
/// GET /api/v1/categories-tree
/// Authorization: Bearer <token>
/// ```
///
/// Response:
/// ```json
/// {
///   "code": 200,
///   "message": "success",
///   "data": [
///     {
///       "id": 1,
///        "category": "熟食卤味",
///       "subCategories": [
///         {
///           "id": 2,
///          "name": "猪肉卤制品"
///         },
///        {
///           "id": 14,
///         "name": "牛肉卤制品"
///         }
///       ]
///     }
///   ]
/// }
/// ```
#[utoipa::path(
    get,
    path = "/api/v1/categories-tree",
    tag = "categories",
    responses(
        (status = 200, description = "Successfully retrieved category tree",
        example = json!({
            "code": 200,
            "message": "success",
            "data": [
              {
                "id": 1,
                "category": "熟食卤味",
                "subCategories": [
                  {
                    "id": 2,
                    "name": "猪肉卤制品"
                  },
                  {
                    "id": 14,
                    "name": "牛肉卤制品"
                  }
                ]
              }
            ]
        })),
        (status = 401, description = "Unauthorized"),
        (status = 400, description = "Bad Request that missing token"),
        (status = 500, description = "Internal errors")
    ),
    security(("bearer" = []))
)]
pub(crate) async fn get_categories_tree<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
) -> Result<Json<ApiResponse<Vec<CategoryWithSubCategoriesDTO>>>, AppError>
where
    T: CategoryRepository + Send + Sync,
{
    let categories = repo.list_categories_with_subcategories().await?;
    Ok(Json(ApiResponse::new(Some(categories), &context)))
}
