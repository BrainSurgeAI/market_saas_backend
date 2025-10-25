use serde::{Deserialize, Serialize};
use sqlx::{types::Json, FromRow};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub(crate) struct CategoryDTO {
    pub(crate) id: i32,

    #[sqlx(rename = "level1_category")]
    pub(crate) level_one_category: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub(crate) struct CategoryWithSubCategoriesDTO {
    #[serde(rename = "id")]
    pub(crate) category_id: i32,

    #[serde(rename = "category")]
    pub(crate) category_name: String,

    #[serde(rename = "subCategories")]
    pub(crate) subcategories: Json<Vec<SubCategoryDTO>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub(crate) struct SubCategoryDTO {
    pub(crate) id: i32,

    pub(crate) name: String,
}
