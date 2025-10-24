use super::my_sql_repository::MySqlRepository;
use crate::{
    common::AppError,
    map_db_err,
    models::role::{MenuConfig, MenuItem, PermissionCreateDto, PermissionResponseDto, Role},
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

    async fn get_permissions(&self) -> Result<Vec<PermissionResponseDto>, AppError>;
    async fn get_permissions_by_role_id(
        &self,
        role_id: i32,
    ) -> Result<Vec<PermissionResponseDto>, AppError>;

    async fn update_permission_by_id(
        &self,
        id: i32,
        permission: &PermissionCreateDto,
    ) -> Result<(), AppError>;
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

    /// 根据角色名称获取菜单配置
    async fn get_menus_by_role_name(&self, role_name: &str) -> Result<Vec<MenuConfig>, AppError>;
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
    async fn get_permissions(&self) -> Result<Vec<PermissionResponseDto>, AppError> {
        let permissions = sqlx::query_as::<_, PermissionResponseDto>(
            r#"
            SELECT * FROM permissions ORDER BY id;
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get permissions"))?;

        Ok(permissions)
    }

    async fn get_permissions_by_role_id(
        &self,
        role_id: i32,
    ) -> Result<Vec<PermissionResponseDto>, AppError> {
        let permissions = sqlx::query_as!(
            PermissionResponseDto,
            r#"
            SELECT p.id, p.name, p.cname, p.description, p.self_only, p.path_pattern, p.http_method FROM permissions p
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

    async fn update_permission_by_id(
        &self,
        id: i32,
        permission: &PermissionCreateDto,
    ) -> Result<(), AppError> {
        sqlx::query_as!(
            Permission,
            r#"
            UPDATE permissions 
            SET name = ?, cname = ?, description = ?, self_only = ?, path_pattern = ?, http_method = ?
            WHERE id = ?;
            "#,
            permission.name,
            permission.cname,
            permission.description,
            permission.self_only,
            permission.path_pattern,
            permission.http_method,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(map_db_err!("Failed to update permission by id"))?;

        Ok(())
    }

    async fn get_menus_by_role_name(&self, role_name: &str) -> Result<Vec<MenuConfig>, AppError> {
        let menu_items = sqlx::query_as!(
            MenuItem,
            r#"
            SELECT DISTINCT mc.id, mc.title, mc.url, mc.icon, mc.is_active, mc.parent_id, mc.sort_order
            FROM menu_config mc
            INNER JOIN role_menu rm ON mc.id = rm.menu_id
            INNER JOIN roles r ON rm.role_id = r.id
            WHERE r.name = ?
            ORDER BY mc.parent_id ASC, mc.sort_order ASC
            "#,
            role_name
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get menus by role name"))?;

        // 构建菜单层次结构
        let mut menu_map: std::collections::HashMap<i32, MenuConfig> = std::collections::HashMap::new();
        let mut children_map: std::collections::HashMap<i32, Vec<MenuConfig>> = std::collections::HashMap::new();

        // 首先处理所有菜单项，创建所有菜单的基础配置
        for item in menu_items {
            let menu_config = MenuConfig {
                id: item.id,
                title: item.title,
                url: item.url,
                icon: item.icon,
                is_active: item.is_active.map(|v| v != 0).unwrap_or(false),
                items: None,
            };

            if item.parent_id.is_none() {
                // 根菜单直接插入到map中
                menu_map.insert(item.id, menu_config);
            } else {
                // 子菜单按父级ID分组
                children_map
                    .entry(item.parent_id.unwrap())
                    .or_insert_with(Vec::new)
                    .push(menu_config);
            }
        }

        // 将子菜单添加到对应的父菜单中
        for (parent_id, children) in children_map {
            if let Some(parent_menu) = menu_map.get_mut(&parent_id) {
                parent_menu.items = Some(children);
            }
        }

        // 只返回根菜单（包含其子菜单）
        let root_menus: Vec<MenuConfig> = menu_map.into_values().collect();

        Ok(root_menus)
    }
}
