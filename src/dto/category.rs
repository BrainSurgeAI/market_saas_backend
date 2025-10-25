use serde::{Deserialize, Serialize};
use sqlx::{types::Json, FromRow};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CategoryDTO {
    pub id: i32,

    #[sqlx(rename = "level1_category")]
    pub level_one_category: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CategoryWithSubCategoriesDTO {
    #[serde(rename = "id")]
    pub category_id: i32,

    #[serde(rename = "category")]
    pub category_name: String,

    #[serde(rename = "subCategories")]
    pub subcategories: Json<Vec<SubCategoryDTO>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SubCategoryDTO {
    pub id: i32,

    pub name: String,
}
