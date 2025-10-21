#[derive(Debug, sqlx::FromRow)]
pub struct UserPermission {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub is_super_admin: bool,
    pub roles: String,
    pub permissions: String,
}
