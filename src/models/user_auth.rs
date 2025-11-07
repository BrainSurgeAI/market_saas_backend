#[derive(Debug, sqlx::FromRow)]
pub(crate) struct UserPermission {
    pub(crate) real_name: String,
    pub(crate) password_hash: String,
    pub(crate) is_super_admin: bool,
    pub(crate) roles: Option<String>
}
