-- Revert migration script
-- 删除初始化的数据

SET FOREIGN_KEY_CHECKS = 0;

-- 删除 user_category_assignments 表数据
DELETE FROM user_category_assignments WHERE id IN (
1,
2,
3,
4,
5,
6,
7,
8,
9,
10,
11,
12,
13,
14,
15
);

SET FOREIGN_KEY_CHECKS = 1;
