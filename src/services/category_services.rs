use axum::Extension;

use crate::{
    common::{ApiResponse, AppError},
    dto::category::CategoryWithSubCategoriesResponseDto,
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
) -> Result<Json<ApiResponse<Vec<CategoryWithSubCategoriesResponseDto>>>, AppError>
where
    T: CategoryRepository + Send + Sync,
{
    let categories = repo.list_categories_with_subcategories().await?;
    Ok(Json(ApiResponse::new(Some(categories), &context)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::category::SubCategoryResponseDto;
    use async_trait::async_trait;
    use sqlx::types::Json as SqlxJson;
    use std::sync::Arc;
    use tokio;

    // Mock repository for testing
    #[derive(Debug, Clone)]
    struct MockCategoryRepository {
        categories: Arc<Vec<CategoryWithSubCategoriesResponseDto>>,
        should_fail: bool,
    }

    impl MockCategoryRepository {
        fn new() -> Self {
            Self {
                categories: Arc::new(vec![]),
                should_fail: false,
            }
        }

        fn with_categories(categories: Vec<CategoryWithSubCategoriesResponseDto>) -> Self {
            Self {
                categories: Arc::new(categories),
                should_fail: false,
            }
        }

        fn with_error() -> Self {
            Self {
                categories: Arc::new(vec![]),
                should_fail: true,
            }
        }
    }

    #[async_trait]
    impl CategoryRepository for MockCategoryRepository {
        async fn list_categories_with_subcategories(&self) -> Result<Vec<CategoryWithSubCategoriesResponseDto>, AppError> {
            if self.should_fail {
                return Err(AppError::internal(
                    "Database connection failed"
                ));
            }
            Ok((*self.categories).clone())
        }
    }

    fn create_test_context() -> RequestContext {
        RequestContext {
            request_id: "test-request-123".to_string(),
            client_ip: Some("127.0.0.1".to_string()),
        }
    }

    fn create_sample_category_data() -> Vec<CategoryWithSubCategoriesResponseDto> {
        vec![
            CategoryWithSubCategoriesResponseDto {
                category_id: 1,
                category_name: "熟食卤味".to_string(),
                subcategories: SqlxJson(vec![
                    SubCategoryResponseDto {
                        id: 2,
                        name: "猪肉卤制品".to_string(),
                    },
                    SubCategoryResponseDto {
                        id: 14,
                        name: "牛肉卤制品".to_string(),
                    },
                ]),
            },
            CategoryWithSubCategoriesResponseDto {
                category_id: 3,
                category_name: "生鲜蔬菜".to_string(),
                subcategories: SqlxJson(vec![
                    SubCategoryResponseDto {
                        id: 4,
                        name: "叶菜类".to_string(),
                    },
                    SubCategoryResponseDto {
                        id: 5,
                        name: "根茎类".to_string(),
                    },
                ]),
            },
        ]
    }

    #[tokio::test]
    async fn test_get_categories_tree_success() {
        // Arrange
        let expected_categories = create_sample_category_data();
        let repo = MockCategoryRepository::with_categories(expected_categories.clone());
        let context = create_test_context();
        let repo_extension = Extension(repo);
        let context_extension = Extension(context.clone());

        // Act
        let result = get_categories_tree::<MockCategoryRepository>(repo_extension, context_extension)
            .await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        let api_response = response.0;

        assert_eq!(api_response.code, 200);
        assert_eq!(api_response.message, "success");
        assert_eq!(api_response.request_id, context.request_id);
        assert!(api_response.data.is_some());

        let actual_categories = api_response.data.as_ref().unwrap();
        assert_eq!(actual_categories.len(), 2);
        assert_eq!(actual_categories[0].category_id, 1);
        assert_eq!(actual_categories[0].category_name, "熟食卤味");
        assert_eq!(actual_categories[0].subcategories.len(), 2);
    }

    #[tokio::test]
    async fn test_get_categories_tree_empty_result() {
        // Arrange
        let repo = MockCategoryRepository::with_categories(vec![]);
        let context = create_test_context();
        let repo_extension = Extension(repo);
        let context_extension = Extension(context);

        // Act
        let result = get_categories_tree::<MockCategoryRepository>(repo_extension, context_extension)
            .await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        let api_response = response.0;

        assert_eq!(api_response.code, 200);
        assert_eq!(api_response.message, "success");
        assert!(api_response.data.is_some());

        let actual_categories = api_response.data.as_ref().unwrap();
        assert_eq!(actual_categories.len(), 0);
    }

    #[tokio::test]
    async fn test_get_categories_tree_repository_error() {
        // Arrange
        let repo = MockCategoryRepository::with_error();
        let context = create_test_context();
        let repo_extension = Extension(repo);
        let context_extension = Extension(context);

        // Act
        let result = get_categories_tree::<MockCategoryRepository>(repo_extension, context_extension)
            .await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Internal(msg) => {
                assert_eq!(msg, "Database connection failed");
            }
            _ => panic!("Expected Internal error"),
        }
    }

    #[tokio::test]
    async fn test_get_categories_tree_large_dataset() {
        // Arrange - Create a large dataset to test performance and boundary conditions
        let mut large_categories = Vec::new();
        for i in 1..=1000 {
            let mut subcategories = Vec::new();
            for j in 1..=50 {
                subcategories.push(SubCategoryResponseDto {
                    id: (i * 1000 + j) as i32,
                    name: format!("SubCategory {}-{}", i, j),
                });
            }

            large_categories.push(CategoryWithSubCategoriesResponseDto {
                category_id: i as i32,
                category_name: format!("Category {}", i),
                subcategories: SqlxJson(subcategories),
            });
        }

        let repo = MockCategoryRepository::with_categories(large_categories);
        let context = create_test_context();
        let repo_extension = Extension(repo);
        let context_extension = Extension(context);

        // Act
        let result = get_categories_tree::<MockCategoryRepository>(repo_extension, context_extension)
            .await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        let api_response = response.0;

        assert_eq!(api_response.code, 200);
        assert!(api_response.data.is_some());

        let actual_categories = api_response.data.as_ref().unwrap();
        assert_eq!(actual_categories.len(), 1000);

        // Verify boundary conditions
        assert_eq!(actual_categories[0].category_id, 1);
        assert_eq!(actual_categories[999].category_id, 1000);
        assert_eq!(actual_categories[0].subcategories.len(), 50);
        assert_eq!(actual_categories[999].subcategories.len(), 50);
    }

    #[tokio::test]
    async fn test_get_categories_tree_empty_subcategories() {
        // Arrange - Test categories with no subcategories
        let categories_with_empty_sub = vec![
            CategoryWithSubCategoriesResponseDto {
                category_id: 1,
                category_name: "Empty Category".to_string(),
                subcategories: SqlxJson(vec![]),
            },
            CategoryWithSubCategoriesResponseDto {
                category_id: 2,
                category_name: "Another Empty Category".to_string(),
                subcategories: SqlxJson(vec![]),
            },
        ];

        let repo = MockCategoryRepository::with_categories(categories_with_empty_sub);
        let context = create_test_context();
        let repo_extension = Extension(repo);
        let context_extension = Extension(context);

        // Act
        let result = get_categories_tree::<MockCategoryRepository>(repo_extension, context_extension)
            .await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        let api_response = response.0;

        assert_eq!(api_response.code, 200);
        assert!(api_response.data.is_some());

        let actual_categories = api_response.data.as_ref().unwrap();
        assert_eq!(actual_categories.len(), 2);
        assert_eq!(actual_categories[0].subcategories.len(), 0);
        assert_eq!(actual_categories[1].subcategories.len(), 0);
    }

    #[tokio::test]
    async fn test_get_categories_tree_single_subcategory() {
        // Arrange - Test boundary condition with exactly one subcategory
        let categories_single_sub = vec![
            CategoryWithSubCategoriesResponseDto {
                category_id: 1,
                category_name: "Single Subcategory".to_string(),
                subcategories: SqlxJson(vec![SubCategoryResponseDto {
                    id: 1,
                    name: "Only Subcategory".to_string(),
                }]),
            },
        ];

        let repo = MockCategoryRepository::with_categories(categories_single_sub);
        let context = create_test_context();
        let repo_extension = Extension(repo);
        let context_extension = Extension(context);

        // Act
        let result = get_categories_tree::<MockCategoryRepository>(repo_extension, context_extension)
            .await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        let api_response = response.0;

        assert_eq!(api_response.code, 200);
        assert!(api_response.data.is_some());

        let actual_categories = api_response.data.as_ref().unwrap();
        assert_eq!(actual_categories.len(), 1);
        assert_eq!(actual_categories[0].subcategories.len(), 1);
        assert_eq!(actual_categories[0].subcategories[0].name, "Only Subcategory");
    }

    #[tokio::test]
    async fn test_get_categories_tree_special_characters() {
        // Arrange - Test boundary condition with special characters and Unicode
        let categories_special = vec![
            CategoryWithSubCategoriesResponseDto {
                category_id: 1,
                category_name: "中文类别".to_string(),
                subcategories: SqlxJson(vec![
                    SubCategoryResponseDto {
                        id: 1,
                        name: "子类别 \"quoted\"".to_string(),
                    },
                    SubCategoryResponseDto {
                        id: 2,
                        name: "Category with 'apostrophe' & symbols!@#$%".to_string(),
                    },
                ]),
            },
        ];

        let repo = MockCategoryRepository::with_categories(categories_special);
        let context = create_test_context();
        let repo_extension = Extension(repo);
        let context_extension = Extension(context);

        // Act
        let result = get_categories_tree::<MockCategoryRepository>(repo_extension, context_extension)
            .await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        let api_response = response.0;

        assert_eq!(api_response.code, 200);
        assert!(api_response.data.is_some());

        let actual_categories = api_response.data.as_ref().unwrap();
        assert_eq!(actual_categories.len(), 1);
        assert_eq!(actual_categories[0].category_name, "中文类别");
        assert_eq!(actual_categories[0].subcategories.len(), 2);
        assert_eq!(actual_categories[0].subcategories[0].name, "子类别 \"quoted\"");
        assert_eq!(actual_categories[0].subcategories[1].name, "Category with 'apostrophe' & symbols!@#$%");
    }

    #[tokio::test]
    async fn test_get_categories_tree_max_id_values() {
        // Arrange - Test boundary condition with maximum ID values
        let categories_max_ids = vec![
            CategoryWithSubCategoriesResponseDto {
                category_id: i32::MAX,
                category_name: "Max ID Category".to_string(),
                subcategories: SqlxJson(vec![
                    SubCategoryResponseDto {
                        id: i32::MAX,
                        name: "Max ID Subcategory".to_string(),
                    },
                    SubCategoryResponseDto {
                        id: i32::MIN,
                        name: "Min ID Subcategory".to_string(),
                    },
                ]),
            },
        ];

        let repo = MockCategoryRepository::with_categories(categories_max_ids);
        let context = create_test_context();
        let repo_extension = Extension(repo);
        let context_extension = Extension(context);

        // Act
        let result = get_categories_tree::<MockCategoryRepository>(repo_extension, context_extension)
            .await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        let api_response = response.0;

        assert_eq!(api_response.code, 200);
        assert!(api_response.data.is_some());

        let actual_categories = api_response.data.as_ref().unwrap();
        assert_eq!(actual_categories.len(), 1);
        assert_eq!(actual_categories[0].category_id, i32::MAX);
        assert_eq!(actual_categories[0].subcategories[0].id, i32::MAX);
        assert_eq!(actual_categories[0].subcategories[1].id, i32::MIN);
    }

    #[tokio::test]
    async fn test_get_categories_tree_long_names() {
        // Arrange - Test boundary condition with very long names
        let long_category_name = "A".repeat(1000);
        let long_subcategory_name = "B".repeat(1000);

        let categories_long_names = vec![
            CategoryWithSubCategoriesResponseDto {
                category_id: 1,
                category_name: long_category_name.clone(),
                subcategories: SqlxJson(vec![
                    SubCategoryResponseDto {
                        id: 1,
                        name: long_subcategory_name.clone(),
                    },
                ]),
            },
        ];

        let repo = MockCategoryRepository::with_categories(categories_long_names);
        let context = create_test_context();
        let repo_extension = Extension(repo);
        let context_extension = Extension(context);

        // Act
        let result = get_categories_tree::<MockCategoryRepository>(repo_extension, context_extension)
            .await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        let api_response = response.0;

        assert_eq!(api_response.code, 200);
        assert!(api_response.data.is_some());

        let actual_categories = api_response.data.as_ref().unwrap();
        assert_eq!(actual_categories.len(), 1);
        assert_eq!(actual_categories[0].category_name, long_category_name);
        assert_eq!(actual_categories[0].subcategories[0].name, long_subcategory_name);
    }

    // Tests for different HTTP status code scenarios
    #[tokio::test]
    async fn test_get_categories_tree_500_database_error() {
        // Arrange - Simulate database connection error (HTTP 500)
        let repo = MockCategoryRepository::with_error();
        let context = create_test_context();
        let repo_extension = Extension(repo);
        let context_extension = Extension(context);

        // Act
        let result = get_categories_tree::<MockCategoryRepository>(repo_extension, context_extension)
            .await;

        // Assert - Should return 500 Internal Server Error
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Internal(msg) => {
                assert_eq!(msg, "Database connection failed");
                assert_eq!(AppError::internal("test").error_code(), 500);
            }
            _ => panic!("Expected Internal error with 500 status code"),
        }
    }

    #[tokio::test]
    async fn test_get_categories_tree_400_bad_request_error() {
        // This test simulates a scenario where validation could fail (HTTP 400)
        // In a real scenario, this might happen if request parameters are invalid
        // Since our function doesn't have direct validation, we test the error structure
        let validation_error = AppError::validation("Invalid request parameters");
        assert_eq!(validation_error.error_code(), 400);
        assert_eq!(validation_error.status_code(), axum::http::StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_get_categories_tree_401_authentication_error() {
        // This test simulates authentication failure (HTTP 401)
        // In real middleware, this would occur before reaching our service
        let auth_error = AppError::auth("Missing or invalid authentication token");
        assert_eq!(auth_error.error_code(), 401);
        assert_eq!(auth_error.status_code(), axum::http::StatusCode::UNAUTHORIZED);
    }

    // Test error response structure matches expected API format
    #[tokio::test]
    async fn test_error_response_structure() {
        // Test that error responses have the correct structure
        let internal_error = AppError::internal("Internal server error");
        let validation_error = AppError::validation("Validation failed");
        let auth_error = AppError::auth("Authentication failed");

        // Verify error codes and status codes
        assert_eq!(internal_error.error_code(), 500);
        assert_eq!(validation_error.error_code(), 400);
        assert_eq!(auth_error.error_code(), 401);

        // Verify error messages
        assert_eq!(internal_error.to_string(), "Internal server error: Internal server error");
        assert_eq!(validation_error.to_string(), "Validation error: Validation failed");
        assert_eq!(auth_error.to_string(), "Authentication failed: Authentication failed");
    }

    // Test error classification (client vs server errors)
    #[tokio::test]
    async fn test_error_classification() {
        // Test client errors (4xx)
        let validation_error = AppError::validation("Bad input");
        let auth_error = AppError::auth("No token");
        let not_found_error = AppError::not_found("Resource not found");
        let conflict_error = AppError::conflict("Resource exists");

        assert!(validation_error.is_client_error());
        assert!(auth_error.is_client_error());
        assert!(not_found_error.is_client_error());
        assert!(conflict_error.is_client_error());
        assert!(!validation_error.is_server_error());

        // Test server errors (5xx)
        let db_error = AppError::Database(sqlx::Error::RowNotFound);
        let internal_error = AppError::internal("Server crash");

        assert!(db_error.is_server_error());
        assert!(internal_error.is_server_error());
        assert!(!db_error.is_client_error());
    }

    // Test API response format for error scenarios
    #[tokio::test]
    async fn test_api_response_error_format() {
        use crate::common::ApiResponse;

        // Test error response structure
        let error_response = ApiResponse::<()>::error(500, "Internal Server Error");

        assert_eq!(error_response.code, 500);
        assert_eq!(error_response.message, "Internal Server Error");
        assert!(error_response.data.is_none());
        assert!(!error_response.request_id.is_empty());
        assert!(!error_response.timestamp.to_string().is_empty());
    }

    // Test response format consistency
    #[tokio::test]
    async fn test_response_format_consistency() {
        use crate::common::ApiResponse;

        let context = create_test_context();

        // Test success response
        let categories = create_sample_category_data();
        let success_response = ApiResponse::new(Some(categories.clone()), &context);

        assert_eq!(success_response.code, 200);
        assert_eq!(success_response.message, "success");
        assert!(success_response.data.is_some());
        assert_eq!(success_response.request_id, context.request_id);

        // Test error response
        let error_response = ApiResponse::<()>::error(404, "Not Found");

        assert_eq!(error_response.code, 404);
        assert_eq!(error_response.message, "Not Found");
        assert!(error_response.data.is_none());
        assert!(!error_response.request_id.is_empty());
    }

    // Test concurrent access scenario (could lead to 500 errors)
    #[tokio::test]
    async fn test_concurrent_category_access() {
        // Arrange - Test concurrent access to the category service
        let categories = create_sample_category_data();
        let repo = MockCategoryRepository::with_categories(categories);

        // Act - Simulate multiple concurrent requests
        let mut handles = Vec::new();
        for _ in 0..10 {
            let repo_clone = repo.clone();
            let context_clone = create_test_context();

            let handle = tokio::spawn(async move {
                let repo_extension = Extension(repo_clone);
                let context_extension = Extension(context_clone);

                get_categories_tree::<MockCategoryRepository>(repo_extension, context_extension).await
            });
            handles.push(handle);
        }

        // Assert - All requests should succeed (200 status)
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
            let response = result.unwrap();
            let api_response = response.0;
            assert_eq!(api_response.code, 200);
        }
    }

    // Test boundary scenario that could cause 500 errors
    #[tokio::test]
    async fn test_extreme_data_boundary_conditions() {
        // Test with extremely deep nesting that could cause server issues
        let mut extreme_subcategories = Vec::new();
        for i in 0..1000 {
            extreme_subcategories.push(SubCategoryResponseDto {
                id: i,
                name: format!("Extreme Subcategory {}", i),
            });
        }

        let extreme_categories = vec![
            CategoryWithSubCategoriesResponseDto {
                category_id: 999999,
                category_name: "Extreme Boundary Test".repeat(100), // Very long name
                subcategories: SqlxJson(extreme_subcategories),
            },
        ];

        let repo = MockCategoryRepository::with_categories(extreme_categories);
        let context = create_test_context();
        let repo_extension = Extension(repo);
        let context_extension = Extension(context);

        // This should handle the extreme case gracefully or return 500 if it fails
        let result = get_categories_tree::<MockCategoryRepository>(repo_extension, context_extension)
            .await;

        match result {
            Ok(response) => {
                // If successful, verify the data integrity
                let api_response = response.0;
                assert_eq!(api_response.code, 200);
                assert!(api_response.data.is_some());

                let categories = api_response.data.as_ref().unwrap();
                assert_eq!(categories.len(), 1);
                assert_eq!(categories[0].subcategories.len(), 1000);
            }
            Err(AppError::Internal(_)) => {
                // If it fails due to server limitations, ensure it's a 500 error
                // This is acceptable behavior for extreme boundary conditions
            }
            Err(_) => {
                panic!("Expected either success or Internal server error");
            }
        }
    }
}
