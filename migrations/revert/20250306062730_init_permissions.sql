-- Revert migration script
-- 删除角色权限关联
DELETE FROM role_permissions;

-- 删除所有角色
DELETE FROM roles;

-- 删除所有权限
DELETE FROM permissions;
