use axum::{Extension, Json};
use tracing::{error, info, warn};

use crate::common::{ApiResponse, AppError};
use crate::dto::{auth::{ResetPasswordDto, SuperAdminLoginRequest}, ValidatedJSON};
use crate::middleware::{auth::create_jwt, context::RequestContext};
use crate::models::claims::Claims;
use crate::repositories::{superadmin_traits::SuperAdminRepository, user_traits::UserRepository};

/// # Request Verify Code
///
/// ## Fields
/// * `phone_number` - Phone number of the user
///
/// ## Returns
/// * `code` - Code of the user  Current mock code is 123456
// pub async fn request_verify_code(_phone_number: String) -> Result<(), AppError> {
//     //let code = "123456";
//     // TODO: send message to third party service
//     //send_message_to_user(phone_number, message).await?;
//     Ok(())
// }

pub(crate) async fn super_admin_login<T, U>(
    Extension(super_admin_repo): Extension<T>,
    Extension(user_repo): Extension<U>,
    Extension(context): Extension<RequestContext>,
    ValidatedJSON(payload): ValidatedJSON<SuperAdminLoginRequest>,
) -> Result<Json<ApiResponse<String>>, AppError>
where
    T: SuperAdminRepository + Send + Sync,
    U: UserRepository + Send + Sync,
{
    // TODO: authenticate verify code

    super_admin_repo
        .authenticate(&payload.username, &payload.password)
        .await?;

    let user_auth = user_repo
        .get_user_permissions(&payload.username)
        .await?
        .ok_or_else(|| {
            warn!(
                "Cannot find user by username: {} or has no permission",
                payload.username
            );
            AppError::Auth("Invalid username or password".to_string())
        })?;

    let claims = Claims {
        tenant_type: "SUPER_ADMIN".to_string(),
        tenant_name: "SUPER_ADMIN".to_string(),
        tenant_hash: "SUPER_ADMIN".to_string(),
        username: payload.username,
        real_name: user_auth.real_name,
        roles: vec![user_auth.roles.to_string()],

        is_super_admin: user_auth.is_super_admin,
        exp: chrono::Utc::now()
            .checked_add_signed(chrono::Duration::days(1))
            .expect("Invalid timestamp")
            .timestamp() as usize,
    };

    let token = create_jwt(&claims).map_err(|e| {
        error!("Error creating JWT: {:?}", e);
        AppError::Internal("Error creating JWT".to_string())
    })?;

    info!("Super Admin {} login success", claims.username);
    Ok(Json(ApiResponse::new(Some(token), &context)))
}

pub(crate) async fn reset_password<T>(
    Extension(super_admin_repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    ValidatedJSON(payload): ValidatedJSON<ResetPasswordDto>,
) -> Result<Json<ApiResponse<String>>, AppError>
where
    T: SuperAdminRepository + Send + Sync,
{
    if !claims.is_super_admin {
        warn!(
            "User {} has no permissions to reset Super Admin's password",
            claims.username
        );
        return Err(AppError::Forbidden("No permission".to_string()));
    }

    super_admin_repo
        .reset_password(&payload.current_password, &payload.new_password)
        .await?;

    Ok(Json(ApiResponse::new(
        Some("Password reset successfully".to_string()),
        &context,
    )))
}