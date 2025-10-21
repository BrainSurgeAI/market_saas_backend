use crate::{common::AppError, map_db_err};
use anyhow::Result;
use async_trait::async_trait;

use tracing::{error, warn};

use super::my_sql_repository::MySqlRepository;

#[async_trait]
pub trait SuperAdminRepository: Send + Sync {
    async fn authenticate(&self, username: &str, password: &str) -> Result<(), AppError>;
    async fn reset_password(
        &self,
        current_password: &str,
        new_password: &str,
    ) -> Result<(), AppError>;
}

#[async_trait]
impl SuperAdminRepository for MySqlRepository {
    async fn authenticate(&self, username: &str, password: &str) -> Result<(), AppError> {
        let record = sqlx::query!(
            "SELECT password_hash FROM users WHERE username = ? AND is_super_admin = 1",
            username
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!("Failed to authenticate super admin"))?;

        if record.is_none() {
            return Err(AppError::Auth("Invalid username or password".to_string()));
        }

        // verify password
        let password_hash = record.unwrap().password_hash;
        if !bcrypt::verify(password, &password_hash).map_err(|e| {
            error!("bcrypt verify error: {}", e);
            AppError::Internal("Password verification failed".to_string())
        })? {
            warn!("Invalid username {} or password {}", username, password);
            return Err(AppError::Auth("Invalid username or password".to_string()));
        }

        Ok(())
    }

    async fn reset_password(
        &self,
        current_password: &str,
        new_password: &str,
    ) -> Result<(), AppError> {
        let record =
            sqlx::query!("SELECT password_hash FROM users WHERE id=1 AND is_super_admin = 1")
                .fetch_optional(&self.pool)
                .await
                .map_err(map_db_err!("Failed to authenticate super admin"))?;

        if record.is_none() {
            return Err(AppError::Auth("Invalid username or password".to_string()));
        }

        let password_hash = record.unwrap().password_hash;
        if !bcrypt::verify(current_password, &password_hash).map_err(|e| {
            error!("bcrypt verify error: {}", e);
            AppError::Internal("Password verification failed".to_string())
        })? {
            return Err(AppError::Auth("Invalid current password".to_string()));
        }

        let new_password_hash = bcrypt::hash(new_password, bcrypt::DEFAULT_COST).map_err(|e| {
            error!("bcrypt hash error: {}", e);
            AppError::Internal("Password hashing failed".to_string())
        })?;

        let result =
            sqlx::query("UPDATE users SET password_hash = ? WHERE id = 1 AND is_super_admin = 1")
                .bind(new_password_hash)
                .execute(&self.pool)
                .await
                .map_err(map_db_err!("Failed to reset super admin password"));

        if result.is_err() {
            return Err(AppError::Auth(
                "Failed to reset super admin password".to_string(),
            ));
        }

        Ok(())
    }
}
