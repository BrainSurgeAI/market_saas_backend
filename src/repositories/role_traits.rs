use super::my_sql_repository::MySqlRepository;
use crate::{
    common::AppError,
    map_db_err,
    models::role::{Permission, Role},
};
use anyhow::Result;
use async_trait::async_trait;
use tracing::error;

#[async_trait]
pub trait RoleRepository: Send + Sync {
    async fn get_roles_by_tenant_type(&self, name: &str) -> Result<Vec<Role>, AppError>;
    async fn get_all_roles(&self) -> Result<Vec<Role>, AppError>;
    async fn get_role_by_id(&self, id: i32) -> Result<Role, AppError>;
    async fn update_role_by_id(&self, role: &Role) -> Result<(), AppError>;

    async fn get_permissions(&self) -> Result<Vec<Permission>, AppError>;
    async fn get_permissions_by_role_id(&self, role_id: i32) -> Result<Vec<Permission>, AppError>;

    /// 更新角色的权限
    ///
    /// # 参数
    /// * `role_id` - 角色ID
    /// * `permission_ids` - 权限ID列表，包含该角色最终拥有的所有权限
    async fn update_role_permissions(
        &self,
        role_id: i32,
        permission_ids: &[i32],
    ) -> Result<(), AppError>;
}

#[async_trait]
impl RoleRepository for MySqlRepository {
    async fn get_roles_by_tenant_type(&self, name: &str) -> Result<Vec<Role>, AppError> {
        let roles = sqlx::query_as::<_, Role>(
            r#"
            SELECT * FROM roles 
            WHERE (tenant_type = ? OR tenant_type IS NULL)
              AND name NOT LIKE '%_ADMIN'
            ORDER BY tenant_type DESC, name
            "#,
        )
        .bind(name)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get roles by tenant type"))?;

        Ok(roles)
    }

    async fn get_all_roles(&self) -> Result<Vec<Role>, AppError> {
        let roles = sqlx::query_as::<_, Role>(
            r#"
            SELECT * FROM roles WHERE id != 1 ORDER BY id;
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get roles"))?;

        Ok(roles)
    }

    async fn get_role_by_id(&self, id: i32) -> Result<Role, AppError> {
        let role = sqlx::query_as!(Role, r#"SELECT * FROM roles WHERE id = ?"#, id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get role by id"))?;

        Ok(role)
    }

    async fn update_role_by_id(&self, role: &Role) -> Result<(), AppError> {
        sqlx::query_as!(
            Role,
            r#"
            UPDATE roles SET name = ?, tenant_type = ?, alias_name = ? WHERE id = ?;
            "#,
            role.name,
            role.tenant_type,
            role.alias_name,
            role.id
        )
        .execute(&self.pool)
        .await
        .map_err(map_db_err!("Failed to update role by id"))?;

        Ok(())
    }
    async fn get_permissions(&self) -> Result<Vec<Permission>, AppError> {
        let permissions = sqlx::query_as::<_, Permission>(
            r#"
            SELECT * FROM permissions ORDER BY id;
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get permissions"))?;

        Ok(permissions)
    }

    async fn get_permissions_by_role_id(&self, role_id: i32) -> Result<Vec<Permission>, AppError> {
        let permissions = sqlx::query_as!(
            Permission,
            r#"
            SELECT p.* FROM permissions p
            JOIN role_permissions rp ON p.id = rp.permission_id
            WHERE rp.role_id = ?;
            "#,
            role_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get permissions by role id"))?;

        Ok(permissions)
    }

    async fn update_role_permissions(
        &self,
        role_id: i32,
        permission_ids: &[i32],
    ) -> Result<(), AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("failed to begin transaction"))?;

        sqlx::query!(r#"DELETE FROM role_permissions WHERE role_id = ?"#, role_id)
            .execute(&mut *tx)
            .await
            .map_err(map_db_err!("failed to delete role permissions"))?;

        if !permission_ids.is_empty() {
            let placeholders: Vec<String> = permission_ids
                .iter()
                .map(|_| "(?, ?)".to_string())
                .collect();

            let query = format!(
                "INSERT INTO role_permissions (role_id, permission_id) VALUES {}",
                placeholders.join(", ")
            );

            let mut query_builder = sqlx::query(&query);

            for &permission_id in permission_ids {
                query_builder = query_builder.bind(role_id).bind(permission_id);
            }

            query_builder
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("failed to insert role permissions"))?;
        }

        tx.commit()
            .await
            .map_err(map_db_err!("failed to commit transaction"))?;

        Ok(())
    }
}
