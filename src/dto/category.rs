use serde::{Deserialize, Serialize};
use sqlx::{types::Json, FromRow};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub(crate) struct CategoryDTO {
    pub(crate) id: i32,

    #[sqlx(rename = "level1_category")]
    pub(crate) level_one_category: String,
}

/// DTO for category with its subcategories, used in API responses.
///
/// This structure represents a category and its nested subcategories, serialized as JSON for frontend consumption.
/// It is derived from database rows and used in endpoints like category listing.
///
/// # Fields
///
/// * `category_id` - The unique ID of the parent category.
/// * `category_name` - The name of the parent category.
/// * `subcategories` - JSON array of subcategories under this category.
///
/// # Example
///
/// ```json
/// {
///   "id": 1,
///   "category": "Vegetables",
///   "subCategories": [
///     { "id": 2, "name": "Root Vegetables" },
///     { "id": 3, "name": "Leafy Greens" }
///   ]
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub(crate) struct CategoryWithSubCategoriesResponseDto {
    #[serde(rename = "id")]
    pub(crate) category_id: i32,

    #[serde(rename = "category")]
    pub(crate) category_name: String,

    #[serde(rename = "subCategories")]
    pub(crate) subcategories: Json<Vec<SubCategoryResponseDto>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub(crate) struct SubCategoryResponseDto {
    pub(crate) id: i32,

    pub(crate) name: String,
}
