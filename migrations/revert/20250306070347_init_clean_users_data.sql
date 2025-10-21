-- Revert migration script
-- 删除初始化的 users 表数据

DELETE FROM users WHERE id IN (
1,
2,
3,
4,
5,
6,
7,
8
);
