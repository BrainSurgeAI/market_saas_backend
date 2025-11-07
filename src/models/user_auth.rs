#[derive(Debug, sqlx::FromRow)]
pub(crate) struct UserPermission {
    pub(crate) id: i32,
    pub(crate) real_name: String,
    pub(crate) username: String,
    pub(crate) password_hash: String,
    pub(crate) is_super_admin: bool,
    pub(crate) roles: String,
    pub(crate) permissions: String,
}
