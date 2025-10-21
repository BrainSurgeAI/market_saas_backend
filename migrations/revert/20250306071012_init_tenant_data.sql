-- Revert migration script
-- 删除初始化的数据

SET FOREIGN_KEY_CHECKS = 0;

-- 删除 tenants 表数据
DELETE FROM tenants WHERE id IN (
1
);

SET FOREIGN_KEY_CHECKS = 1;
