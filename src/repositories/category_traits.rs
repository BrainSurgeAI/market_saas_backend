use crate::{common::AppError, dto::category::CategoryWithSubCategoriesDTO, map_db_err};

use super::my_sql_repository::MySqlRepository;
use async_trait::async_trait;

use tracing::error;

#[async_trait]
pub trait CategoryRepository: Send + Sync {

    /// List categories with subcategories
    ///
    /// This function fetches all categories with their subcategories from the database.
    ///
    /// # Returns
    ///
    /// A vector of `CategoryWithSubCategoriesDTO` objects.
    ///
    /// # Errors
    ///
    /// Returns an `AppError` if the database query fails.
    async fn list_categories_with_subcategories(&self) -> Result<Vec<CategoryWithSubCategoriesDTO>, AppError>;
}

#[async_trait]
impl CategoryRepository for MySqlRepository {
    async fn list_categories_with_subcategories(&self) -> Result<Vec<CategoryWithSubCategoriesDTO>, AppError> {
        let sql = r#"
        SELECT c1.id as category_id, c1.name as category_name,
        JSON_ARRAYAGG(
            JSON_OBJECT(
                'id', c2.id,
                'name', c2.name
            )
        ) as subcategories
        FROM 
            categories c1
        LEFT JOIN 
            categories c2 ON c1.id = c2.parent_id AND c2.level = 2
        WHERE 
            c1.level = 1
        GROUP BY 
            c1.id, c1.name
        ORDER BY c1.sort_order DESC;"#;

        let categories = sqlx::query_as::<_, CategoryWithSubCategoriesDTO>(sql)
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err!("Failed to fetch categories with sub categories"))?;
        Ok(categories)
    }
}
