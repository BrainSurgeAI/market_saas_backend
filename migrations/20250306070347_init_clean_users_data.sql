-- Add migration script here
-- 初始化 users 表数据


INSERT IGNORE INTO `users` (`id`, `name`, `username`, `password_hash`, `email`, `phone`, `tenant_id`, `is_super_admin`, `deleted_at`, `created_at`, `updated_at`) VALUES (1,'超级管理员','SuperAdmin','$2b$12$MU9GAahOXfgFsYjqz5j7fOXkhkDhviXT3o8QUhJ8y1wTkIHYKhIQW',NULL,NULL,NULL,1,NULL,'2025-03-06 06:52:27','2025-03-06 06:52:27');
INSERT IGNORE INTO `users` (`id`, `name`, `username`, `password_hash`, `email`, `phone`, `tenant_id`, `is_super_admin`, `deleted_at`, `created_at`, `updated_at`) VALUES (2,'邦来惠管理员','blhAdmin','$2b$12$Bg0yBHToSuFtuOAI3bBu6uYrp31U6qWiaCrinSvb2Hk4ZIMIiwsEi',NULL,NULL,1,0,NULL,'2025-03-06 06:52:37','2025-03-06 06:52:37');
INSERT IGNORE INTO `users` (`id`, `name`, `username`, `password_hash`, `email`, `phone`, `tenant_id`, `is_super_admin`, `deleted_at`, `created_at`, `updated_at`) VALUES (3,'马龙龙','Malonglong','$2b$12$gsSdVDWZVRcDKzVyRF.cbefEP6YbUIVdH45JTPF6/03W6aGsnyKgG',NULL,NULL,1,0,NULL,'2025-03-06 06:52:57','2025-03-06 06:52:57');
INSERT IGNORE INTO `users` (`id`, `name`, `username`, `password_hash`, `email`, `phone`, `tenant_id`, `is_super_admin`, `deleted_at`, `created_at`, `updated_at`) VALUES (4,'马亚龙','Mayalong','$2b$12$gsSdVDWZVRcDKzVyRF.cbefEP6YbUIVdH45JTPF6/03W6aGsnyKgG',NULL,NULL,1,0,NULL,'2025-03-06 06:52:57','2025-03-06 06:52:57');
INSERT IGNORE INTO `users` (`id`, `name`, `username`, `password_hash`, `email`, `phone`, `tenant_id`, `is_super_admin`, `deleted_at`, `created_at`, `updated_at`) VALUES (5,'艾力','Aili','$2b$12$gsSdVDWZVRcDKzVyRF.cbefEP6YbUIVdH45JTPF6/03W6aGsnyKgG',NULL,NULL,1,0,NULL,'2025-03-06 06:52:57','2025-03-06 06:52:57');
INSERT IGNORE INTO `users` (`id`, `name`, `username`, `password_hash`, `email`, `phone`, `tenant_id`, `is_super_admin`, `deleted_at`, `created_at`, `updated_at`) VALUES (6,'单美茹','Shanmeiru','$2b$12$gsSdVDWZVRcDKzVyRF.cbefEP6YbUIVdH45JTPF6/03W6aGsnyKgG',NULL,NULL,1,0,NULL,'2025-03-06 06:52:57','2025-03-06 06:52:57');
INSERT IGNORE INTO `users` (`id`, `name`, `username`, `password_hash`, `email`, `phone`, `tenant_id`, `is_super_admin`, `deleted_at`, `created_at`, `updated_at`) VALUES (7,'张庆荣','Zhangqingrong','$2b$12$gsSdVDWZVRcDKzVyRF.cbefEP6YbUIVdH45JTPF6/03W6aGsnyKgG',NULL,NULL,1,0,NULL,'2025-03-06 06:52:57','2025-03-06 06:52:57');
INSERT IGNORE INTO `users` (`id`, `name`, `username`, `password_hash`, `email`, `phone`, `tenant_id`, `is_super_admin`, `deleted_at`, `created_at`, `updated_at`) VALUES (8,'邦来惠审核员','blhAuditor','$2b$12$gsSdVDWZVRcDKzVyRF.cbefEP6YbUIVdH45JTPF6/03W6aGsnyKgG',NULL,NULL,1,0,NULL,'2025-03-06 06:52:57','2025-03-06 06:52:57');




