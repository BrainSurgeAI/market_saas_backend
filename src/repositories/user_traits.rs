use crate::{
    common::AppError,
    dto::users::{UserCreateDto, UserResponseDto, UserUpdateDto},
    map_db_err,
    models::user_auth::UserPermission,
};

use async_trait::async_trait;
use tracing::{debug, error, info};

use super::my_sql_repository::MySqlRepository;

#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Get user permissions by username
    ///
    /// # Arguments
    /// * `username` - Username to lookup
    ///
    /// # Returns
    /// * `Result<Option<UserPermission>, AppError>` - User's permissions if found
    ///   - `Some(UserPermission)` contains the user's ID, username, password hash, roles and permissions
    ///   - `None` if user not found
    ///   - `AppError` on database errors
    ///
    /// # Implementation
    /// Joins users, roles and permissions tables to get all permissions for the user.
    /// Only returns active users (deleted_at is null).
    /// Groups results by user ID to handle multiple roles/permissions.
    async fn get_user_permissions(
        &self,
        username: &str,
    ) -> Result<Option<UserPermission>, AppError>;

    async fn create_user(
        &self,
        name: &str,
        username: &str,
        hashed_password: &str,
        roles: &str,
    ) -> Result<u64, AppError>;

    async fn is_user_exists(&self, username: &str) -> Result<bool, AppError>;

    async fn get_permissions_by_role(&self, role: &str) -> Result<Option<String>, AppError>;

    async fn send_message_to_user(&self, user_id: u64, content: &str) -> Result<(), AppError>;

    async fn get_user_by_username(
        &self,
        username: &str,
    ) -> Result<Option<UserResponseDto>, AppError>;

    async fn update_user(&self, username: &str, profile: &UserUpdateDto) -> Result<(), AppError>;

    async fn disable_user(&self, username: &str) -> Result<(), AppError>;

    async fn enable_user(&self, username: &str) -> Result<(), AppError>;

    async fn reset_password(
        &self,
        username: &str,
        current_password: &str,
        new_password: &str,
    ) -> Result<(), AppError>;

    async fn create_tenant_with_admin(
        &self,
        params: &CreateTenantWithAdminParams,
    ) -> Result<u64, AppError>;

    async fn create_tenant_user(
        &self,
        tenant_hash: &str,
        user: &UserCreateDto,
    ) -> Result<u64, AppError>;
}

#[async_trait]
impl UserRepository for MySqlRepository {
    async fn disable_user(&self, username: &str) -> Result<(), AppError> {
        let result = sqlx::query("UPDATE users SET deleted_at = NOW() WHERE username = ?")
            .bind(username)
            .execute(&self.pool)
            .await
            .map_err(map_db_err!("Error disabling user"))?;

        if result.rows_affected() > 0 {
            info!("User {} disabled", username);
            Ok(())
        } else {
            Err(AppError::NotFound("User not found".to_string()))
        }
    }

    async fn create_tenant_user(
        &self,
        tenant_hash: &str,
        user: &UserCreateDto,
    ) -> Result<u64, AppError> {
        let tenant = sqlx::query!(
            "SELECT id, tenant_type FROM tenants WHERE name_hash = ?",
            tenant_hash
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!("Error fetching tenant id"))?
        .ok_or_else(|| {
            error!("Tenant {} not found", tenant_hash);
            return AppError::NotFound("Tenant not found".to_string());
        })?;

        let user_exists = self.is_user_exists(&user.username).await?;
        if user_exists {
            return Err(AppError::Conflict("User already exists".to_string()));
        }

        let hashed_password = bcrypt::hash(user.password.as_str(), bcrypt::DEFAULT_COST)
            .map_err(|_| AppError::Internal("Error hashing password".to_string()))?;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Error beginning transaction"))?;

        let user_id = sqlx::query!(
            "INSERT INTO users (name, username, password_hash, email, phone, tenant_id) VALUES (?, ?, ?, ?, ?, ?)",
            user.name, user.username, hashed_password, user.email, user.phone, tenant.id
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Error creating user"))?
        .last_insert_id();

        debug!(
            "new user id: {} role {} tenant type {}",
            user_id, user.role, tenant.tenant_type
        );

        let res = sqlx::query(
            "INSERT INTO user_roles (user_id, role_id) 
            SELECT ?, r.id 
            FROM roles r
            WHERE r.name = ? AND (r.tenant_type = ? OR r.tenant_type IS NULL)",
        )
        .bind(user_id)
        .bind(user.role.to_uppercase())
        .bind(&tenant.tenant_type)
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Error inserting user role"))?;

        if res.rows_affected() == 0 {
            error!("Failed to create user role {}", user.role);
            return Err(AppError::Internal("Failed to create user role".to_string()));
        }

        let res = sqlx::query!(
            "UPDATE tenants SET status = 'ACTIVE' WHERE id = ?",
            tenant.id
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to update tenant status"))?;

        if res.rows_affected() > 0 {
            tx.commit()
                .await
                .map_err(map_db_err!("Failed to commit transaction"))?;

            Ok(user_id)
        } else {
            Err(AppError::Internal("Failed to activate tenant".to_string()))
        }
    }

    async fn get_user_permissions(
        &self,
        username: &str,
    ) -> Result<Option<UserPermission>, AppError> {
        sqlx::query_as::<_, UserPermission>(
            "SELECT u.id, u.name as real_name, u.username, u.password_hash, u.is_super_admin,
            GROUP_CONCAT(DISTINCT r.name) as roles, 
            GROUP_CONCAT(DISTINCT p.name) as permissions
            FROM users u INNER JOIN user_roles ur ON u.id = ur.user_id
            INNER JOIN roles r ON ur.role_id = r.id 
            INNER JOIN role_permissions rp ON r.id = rp.role_id 
            INNER JOIN permissions p ON rp.permission_id = p.id 
            WHERE u.username = ? and u.deleted_at is null GROUP BY u.id",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!("Error fetching user permissions"))
    }

    async fn create_user(
        &self,
        name: &str,
        username: &str,
        hashed_password: &str,
        tenant_type: &str,
    ) -> Result<u64, AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Error beginning transaction"))?;

        let user = sqlx::query(
            "INSERT INTO users (name, username, password_hash, tenant_type) VALUES (?, ?, ?, ?)",
        )
        .bind(name)
        .bind(username)
        .bind(hashed_password)
        .bind(tenant_type)
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Error inserting user"))?;

        let user_id: u64 = user.last_insert_id();

        let role_result = sqlx::query(
            "INSERT INTO user_roles (user_id, role_id) 
             SELECT ?, r.id 
             FROM roles r
             WHERE r.name = ? AND (r.tenant_type = ? OR r.tenant_type IS NULL)",
        )
        .bind(user_id)
        .bind(tenant_type)
        .bind(tenant_type)
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Error inserting user role"))?;

        if role_result.rows_affected() == 0 {
            return Err(AppError::Validation(format!(
                "Role {} is not valid for tenant type {}",
                tenant_type, tenant_type
            )));
        }

        tx.commit()
            .await
            .map_err(map_db_err!("Error committing transaction"))?;

        Ok(user_id)
    }

    async fn is_user_exists(&self, username: &str) -> Result<bool, AppError> {
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM users WHERE username = ?)")
            .bind(username)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_err!("Error checking if user exists"))
    }

    async fn get_permissions_by_role(&self, role: &str) -> Result<Option<String>, AppError> {
        let permissions = sqlx::query_scalar::<_, String>(
            "SELECT GROUP_CONCAT(p.name) FROM permissions p 
            INNER JOIN role_permissions rp ON p.id = rp.permission_id   
            INNER JOIN roles r ON rp.role_id = r.id
            WHERE r.name = ?",
        )
        .bind(role)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!("Error fetching permissions"))?;

        Ok(permissions)
    }

    async fn send_message_to_user(&self, user_id: u64, content: &str) -> Result<(), AppError> {
        let _ = sqlx::query("INSERT INTO messages (user_id, content) VALUES (?, ?)")
            .bind(user_id)
            .bind(content)
            .execute(&self.pool)
            .await
            .map_err(map_db_err!("Error sending message to user"))?;

        Ok(())
    }

    async fn get_user_by_username(
        &self,
        username: &str,
    ) -> Result<Option<UserResponseDto>, AppError> {
        sqlx::query_as::<_, UserResponseDto>(
            "SELECT u.id, u.name, u.username, u.email, u.phone, u.deleted_at, t.name as tenant_name, u.created_at, u.updated_at, r.name as role
            FROM users u 
            INNER JOIN tenants t ON u.tenant_id = t.id 
            JOIN user_roles ur ON ur.user_id = u.id
            JOIN roles r ON r.id = ur.role_id
            WHERE u.username = ? AND u.deleted_at is null",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!("Error fetching user by username"))
    }

    async fn reset_password(
        &self,
        username: &str,
        current_password: &str,
        new_password: &str,
    ) -> Result<(), AppError> {
        let password_hash = sqlx::query_scalar!(
            "SELECT password_hash FROM users WHERE username = ?",
            username
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!("Error fetching user"))?;

        let hashed_password =
            password_hash.ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        let is_valid = bcrypt::verify(&current_password, &hashed_password)
            .map_err(|_| AppError::Internal("Error verifying password".to_string()))?;

        if !is_valid {
            return Err(AppError::Auth("Invalid current password".to_string()));
        }

        let new_password_hash = bcrypt::hash(new_password, bcrypt::DEFAULT_COST)
            .map_err(|_| AppError::Internal("Error hashing password".to_string()))?;

        sqlx::query("UPDATE users SET password_hash = ? WHERE username = ?")
            .bind(new_password_hash)
            .bind(username)
            .execute(&self.pool)
            .await
            .map_err(map_db_err!("Error resetting password"))?;

        Ok(())
    }

    async fn create_tenant_with_admin(
        &self,
        params: &CreateTenantWithAdminParams,
    ) -> Result<u64, AppError> {
        // Open a transaction
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Error beginning transaction"))?;

        // Create a tenant
        let tenant =
            sqlx::query("INSERT INTO tenants (name, tenant_type, name_hash) VALUES (?, ?, ?)")
                .bind(&params.tenant_name)
                .bind(&params.tenant_type)
                .bind(&params.tenant_hash_name)
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Error creating tenant"))?;

        let tenant_id: u64 = tenant.last_insert_id();

        // Create a user
        let user = sqlx::query(
            "INSERT INTO users (name, username, password_hash, tenant_id) VALUES (?, ?, ?, ?)",
        )
        .bind(&params.name)
        .bind(&params.username)
        .bind(&params.hashed_password)
        .bind(tenant_id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Error creating user"))?;

        let user_id: u64 = user.last_insert_id();

        // add a role that is the admin of the tenant, validating tenant type
        let role_result = sqlx::query(
            "INSERT INTO user_roles (user_id, role_id) 
            SELECT ?, r.id 
            FROM roles r
            WHERE r.name = ? AND (r.tenant_type = ? OR r.tenant_type IS NULL)",
        )
        .bind(user_id)
        .bind(&params.role)
        .bind(&params.tenant_type)
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Error inserting user role"))?;

        if role_result.rows_affected() == 0 {
            return Err(AppError::Validation(format!(
                "Role {} is not valid for tenant type {}",
                params.role, params.tenant_type
            )));
        }

        // Activate the tenant
        sqlx::query("UPDATE tenants SET status = 'ACTIVE' WHERE id = ?")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(map_db_err!("Failed to update tenant status"))?;

        tx.commit()
            .await
            .map_err(map_db_err!("Error committing transaction"))?;

        Ok(tenant_id)
    }

    async fn update_user(&self, username: &str, profile: &UserUpdateDto) -> Result<(), AppError> {
        let _ = sqlx::query("UPDATE users SET name = ?, email = ?, phone = ? WHERE username = ?")
            .bind(&profile.name)
            .bind(&profile.email)
            .bind(&profile.phone)
            .bind(username)
            .execute(&self.pool)
            .await
            .map_err(map_db_err!("Error updating user profile"));

        Ok(())
    }

    async fn enable_user(&self, username: &str) -> Result<(), AppError> {
        let result = sqlx::query("UPDATE users SET deleted_at = NULL WHERE username = ?")
            .bind(username)
            .execute(&self.pool)
            .await
            .map_err(map_db_err!("Error enabling user"))?;

        if result.rows_affected() > 0 {
            info!("User {} enabled", username);
            Ok(())
        } else {
            Err(AppError::NotFound("User not found".to_string()))
        }
    }
}

#[derive(Default)]
pub struct CreateTenantWithAdminParamsBuilder {
    tenant_type: Option<String>,
    tenant_name: Option<String>,
    tenant_hash_name: Option<String>,
    username: Option<String>,
    hashed_password: Option<String>,
    role: Option<String>,
    name: Option<String>,
}

#[derive(Debug)]
pub struct CreateTenantWithAdminParams {
    pub name: String,
    pub tenant_type: String,
    pub tenant_name: String,
    pub tenant_hash_name: String,
    pub username: String,
    pub hashed_password: String,
    pub role: String,
}

impl CreateTenantWithAdminParams {
    pub fn builder() -> CreateTenantWithAdminParamsBuilder {
        CreateTenantWithAdminParamsBuilder::default()
    }
}

impl CreateTenantWithAdminParamsBuilder {
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn tenant_type(mut self, tenant_type: impl Into<String>) -> Self {
        self.tenant_type = Some(tenant_type.into());
        self
    }

    pub fn tenant_name(mut self, tenant_name: impl Into<String>) -> Self {
        self.tenant_name = Some(tenant_name.into());
        self
    }

    pub fn tenant_hash_name(mut self, tenant_hash_name: impl Into<String>) -> Self {
        self.tenant_hash_name = Some(tenant_hash_name.into());
        self
    }

    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    pub fn hashed_password(mut self, hashed_password: impl Into<String>) -> Self {
        self.hashed_password = Some(hashed_password.into());
        self
    }

    pub fn role(mut self, role: impl Into<String>) -> Self {
        self.role = Some(role.into());
        self
    }

    pub fn build(self) -> Result<CreateTenantWithAdminParams, &'static str> {
        Ok(CreateTenantWithAdminParams {
            name: self.name.ok_or("name is required")?,
            tenant_type: self.tenant_type.ok_or("tenant_type is required")?,
            tenant_name: self.tenant_name.ok_or("tenant_name is required")?,
            tenant_hash_name: self
                .tenant_hash_name
                .ok_or("tenant_hash_name is required")?,
            username: self.username.ok_or("username is required")?,
            hashed_password: self.hashed_password.ok_or("hashed_password is required")?,
            role: self.role.ok_or("role is required")?,
        })
    }
}
