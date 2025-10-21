-- Revert migration script
-- 删除初始化的数据

SET FOREIGN_KEY_CHECKS = 0;

-- 删除 user_roles 表数据
DELETE FROM user_roles WHERE (user_id, role_id) IN (
    (2, 2),
    (3, 5),
    (4, 5),
    (5, 5),
    (6, 5),
    (7, 5),
    (8, 6)
);

SET FOREIGN_KEY_CHECKS = 1;
