/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `_sqlx_test_databases`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `_sqlx_test_databases` (
  `db_name` text NOT NULL,
  `test_path` text NOT NULL,
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`db_name`(63))
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `categories`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `categories` (
  `id` int NOT NULL AUTO_INCREMENT,
  `name` varchar(50) NOT NULL COMMENT '分类名称',
  `level` tinyint NOT NULL COMMENT '分类层级：1-一级分类(肉类), 2-二级分类(牛肉类), 3-三级分类(半肥半瘦牛肉馅)',
  `parent_id` int DEFAULT NULL COMMENT '父级分类ID',
  `sort_order` smallint DEFAULT '0' COMMENT '排序号',
  PRIMARY KEY (`id`),
  KEY `idx_parent_id` (`parent_id`),
  KEY `idx_level_sort` (`level`,`sort_order`)
) ENGINE=InnoDB AUTO_INCREMENT=966 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='产品分类表';
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `customer_category_discount_history`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `customer_category_discount_history` (
  `id` int NOT NULL AUTO_INCREMENT COMMENT '主键ID',
  `tenant_id` int NOT NULL COMMENT '客户租户ID',
  `category_id` int NOT NULL COMMENT '一级分类ID',
  `old_discount_rate` decimal(4,3) DEFAULT NULL COMMENT '原折扣率',
  `new_discount_rate` decimal(4,3) NOT NULL COMMENT '新折扣率',
  `change_type` varchar(16) NOT NULL COMMENT '变更类型：CREATE-新建, UPDATE-修改, EXPIRE-失效',
  `change_reason` varchar(255) DEFAULT NULL COMMENT '变更原因',
  `changed_by` varchar(32) NOT NULL COMMENT '操作人',
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  PRIMARY KEY (`id`),
  KEY `idx_tenant_category` (`tenant_id`,`category_id`),
  KEY `idx_created_at` (`created_at`)
) ENGINE=InnoDB AUTO_INCREMENT=7 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='客户分类折扣历史表';
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `customer_category_discounts`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `customer_category_discounts` (
  `id` int NOT NULL AUTO_INCREMENT COMMENT '主键ID',
  `tenant_id` int NOT NULL COMMENT '客户租户ID',
  `category_id` int NOT NULL COMMENT '一级分类ID',
  `discount_rate` decimal(4,2) NOT NULL COMMENT '折扣率：0.8表示8折',
  `start_date` date NOT NULL COMMENT '生效开始日期',
  `end_date` date DEFAULT NULL COMMENT '生效结束日期，NULL表示永久有效',
  `status` tinyint(1) NOT NULL DEFAULT '1' COMMENT '状态：true-生效中, false-已失效',
  `remark` varchar(255) DEFAULT NULL COMMENT '备注说明',
  `created_by` varchar(32) NOT NULL COMMENT '创建人',
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  `updated_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_tenant_category` (`tenant_id`,`category_id`,`start_date`),
  KEY `idx_category_status` (`category_id`,`status`),
  KEY `idx_tenant_status` (`tenant_id`,`status`),
  KEY `idx_date_status` (`start_date`,`end_date`,`status`),
  CONSTRAINT `fk_customer_discounts_category` FOREIGN KEY (`category_id`) REFERENCES `categories` (`id`),
  CONSTRAINT `fk_customer_discounts_tenant` FOREIGN KEY (`tenant_id`) REFERENCES `tenants` (`id`),
  CONSTRAINT `chk_discount_rate` CHECK (((`discount_rate` > 0) and (`discount_rate` <= 1)))
) ENGINE=InnoDB AUTO_INCREMENT=6 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='客户分类折扣表';
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `delivery_staff`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `delivery_staff` (
  `id` int NOT NULL AUTO_INCREMENT COMMENT '主键ID',
  `provider_id` int NOT NULL COMMENT '租户ID，关联tenants表',
  `name` varchar(32) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '姓名',
  `phone` char(11) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '手机号',
  `id_card` char(18) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '身份证号',
  `created_by` varchar(32) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '创建人',
  `status` tinyint NOT NULL DEFAULT '1' COMMENT '状态：0=禁用，1=启用',
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  `remark` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '备注',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_tenant_phone` (`provider_id`,`phone`),
  KEY `idx_provider_id` (`provider_id`),
  CONSTRAINT `fk_delivery_staff_provider` FOREIGN KEY (`provider_id`) REFERENCES `tenants` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB AUTO_INCREMENT=5 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='配送员信息表';
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `exchange_items`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `exchange_items` (
  `id` int NOT NULL AUTO_INCREMENT,
  `return_exchange_id` int NOT NULL COMMENT '关联的退换货记录ID',
  `product_code` varchar(10) NOT NULL COMMENT '换货产品编号',
  `product_name` varchar(100) NOT NULL COMMENT '换货产品名称',
  `quantity` decimal(10,2) NOT NULL COMMENT '换货数量',
  `price` decimal(10,2) NOT NULL COMMENT '换货产品单价',
  `total_amount` decimal(12,2) NOT NULL COMMENT '换货总金额',
  `status` varchar(16) NOT NULL DEFAULT 'PENDING' COMMENT '状态: PENDING-待发货, SHIPPED-已发货, RECEIVED-已收货',
  `shipped_at` timestamp NULL DEFAULT NULL COMMENT '发货时间',
  `received_at` timestamp NULL DEFAULT NULL COMMENT '收货时间',
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  `updated_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
  PRIMARY KEY (`id`),
  KEY `fk_exchange_items_return_exchange` (`return_exchange_id`),
  CONSTRAINT `fk_exchange_items_return_exchange` FOREIGN KEY (`return_exchange_id`) REFERENCES `return_exchange_records` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='换货商品表';
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `menu_config`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `menu_config` (
  `id` int NOT NULL AUTO_INCREMENT COMMENT '菜单ID',
  `title` varchar(100) NOT NULL COMMENT '菜单标题 (e.g., "系统管理")',
  `url` varchar(255) NOT NULL COMMENT '菜单URL模板 (e.g., "/workspace")',
  `icon` varchar(50) DEFAULT NULL COMMENT '图标名 (e.g., "Settings")',
  `parent_id` int DEFAULT NULL COMMENT '父菜单ID (NULL=根级)',
  `sort_order` int DEFAULT '0' COMMENT '排序 (小在前)',
  `section` varchar(20) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL DEFAULT 'navMain' COMMENT '所属section (navMain/projects/teams)',
  `is_active` tinyint(1) DEFAULT '1' COMMENT '是否启用',
  `created_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  KEY `idx_menu_parent_order` (`parent_id`,`sort_order`),
  KEY `idx_menu_section` (`section`),
  CONSTRAINT `menu_config_ibfk_1` FOREIGN KEY (`parent_id`) REFERENCES `menu_config` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB AUTO_INCREMENT=29 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='菜单配置表';
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `messages`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `messages` (
  `id` int NOT NULL AUTO_INCREMENT,
  `user_id` int NOT NULL,
  `content` text NOT NULL,
  `is_read` tinyint(1) DEFAULT '0',
  `created_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  KEY `idx_user_id` (`user_id`),
  CONSTRAINT `fk_messages_user` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `order_details`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `order_details` (
  `id` int NOT NULL AUTO_INCREMENT COMMENT '主键ID',
  `order_id` int NOT NULL COMMENT '订单ID',
  `product_code` varchar(10) NOT NULL COMMENT '产品编号',
  `product_name` varchar(100) NOT NULL COMMENT '产品名称（下单时）',
  `category_id` int NOT NULL COMMENT '分类ID',
  `category_name` varchar(50) NOT NULL COMMENT '分类名称（下单时）',
  `unit` varchar(4) NOT NULL COMMENT '单位',
  `ordered_qty` decimal(8,2) NOT NULL COMMENT '下单数量',
  `unit_price` decimal(10,2) NOT NULL DEFAULT '0.00' COMMENT '原始单价',
  `discount_rate` decimal(4,2) NOT NULL DEFAULT '1.00' COMMENT '折扣率',
  `discounted_unit_price` decimal(10,2) NOT NULL DEFAULT '0.00' COMMENT '实际单价（折扣后）',
  `ordered_amount` decimal(12,2) NOT NULL DEFAULT '0.00' COMMENT '总金额（数量*实际单价）',
  `net_amount` decimal(12,2) DEFAULT '0.00' COMMENT '实际总金额（实际数量*实际单价）',
  `processing_requirements` text COMMENT '加工要求',
  `remark` varchar(255) DEFAULT NULL COMMENT '备注',
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  `updated_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
  PRIMARY KEY (`id`),
  KEY `idx_order_id` (`order_id`),
  KEY `idx_product_code` (`product_code`),
  KEY `fk_order_details_category` (`category_id`),
  CONSTRAINT `fk_order_details_category` FOREIGN KEY (`category_id`) REFERENCES `categories` (`id`),
  CONSTRAINT `fk_order_details_order` FOREIGN KEY (`order_id`) REFERENCES `orders` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB AUTO_INCREMENT=4 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='订单明细表';
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `order_inspection_items`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `order_inspection_items` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `inspection_id` int unsigned NOT NULL,
  `order_detail_id` int NOT NULL,
  `inspected_qty` decimal(10,2) NOT NULL DEFAULT '0.00',
  `accepted` tinyint(1) DEFAULT '0',
  `remarks` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `result` enum('PENDING','SIGN','EXCHANGE','RETURN') CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci NOT NULL DEFAULT 'PENDING' COMMENT '验收结果：SIGN-签收，EXCHANGE-换货，RETURN-退货',
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `need_to_inspection` decimal(10,2) NOT NULL,
  PRIMARY KEY (`id`),
  KEY `fk_inspection_items` (`inspection_id`),
  KEY `fk_inspection_items_order_detail` (`order_detail_id`),
  CONSTRAINT `fk_inspection_items` FOREIGN KEY (`inspection_id`) REFERENCES `order_inspections` (`id`) ON DELETE CASCADE,
  CONSTRAINT `fk_inspection_items_order_detail` FOREIGN KEY (`order_detail_id`) REFERENCES `order_details` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB AUTO_INCREMENT=4 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `order_inspections`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `order_inspections` (
  `id` int unsigned NOT NULL AUTO_INCREMENT,
  `order_id` int NOT NULL,
  `inspected_by_type` enum('MARKET','CUSTOMER') CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci NOT NULL,
  `inspected_by_id` int NOT NULL,
  `inspection_result` enum('PENDING','PASS','PARTIAL','REJECTED','EXCHANGE') CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci NOT NULL DEFAULT 'PENDING',
  `parent_id` int unsigned DEFAULT NULL,
  `inspection_round` int NOT NULL DEFAULT '1',
  `inspected_at` datetime DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  KEY `fk_order_inspections_orders` (`order_id`),
  KEY `fk_order_inspections_users` (`inspected_by_id`),
  KEY `fk_order_inspections_parent` (`parent_id`),
  CONSTRAINT `fk_order_inspections_orders` FOREIGN KEY (`order_id`) REFERENCES `orders` (`id`) ON DELETE CASCADE,
  CONSTRAINT `fk_order_inspections_parent` FOREIGN KEY (`parent_id`) REFERENCES `order_inspections` (`id`) ON DELETE SET NULL,
  CONSTRAINT `fk_order_inspections_users` FOREIGN KEY (`inspected_by_id`) REFERENCES `users` (`id`) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `order_status_history`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `order_status_history` (
  `id` int NOT NULL AUTO_INCREMENT COMMENT '主键ID',
  `order_id` int NOT NULL COMMENT '订单ID',
  `from_status` varchar(32) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL COMMENT '原状态',
  `to_status` varchar(32) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL COMMENT '新状态',
  `changed_by` varchar(32) NOT NULL COMMENT '操作人',
  `change_reason` varchar(255) DEFAULT NULL COMMENT '变更原因',
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  PRIMARY KEY (`id`),
  KEY `idx_order_id` (`order_id`),
  KEY `idx_order_status_time` (`order_id`,`to_status`,`created_at`),
  CONSTRAINT `fk_order_history_order` FOREIGN KEY (`order_id`) REFERENCES `orders` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB AUTO_INCREMENT=7 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='订单状态变更历史表';
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `orders`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `orders` (
  `id` int NOT NULL AUTO_INCREMENT COMMENT '主键ID',
  `order_code` varchar(32) NOT NULL COMMENT '订单编号：DD-时间戳-4位随机数',
  `customer_id` int NOT NULL COMMENT '客户租户ID',
  `market_id` int NOT NULL COMMENT '市场租户ID',
  `order_status` varchar(32) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL DEFAULT 'PENDING' COMMENT '订单状态：PENDING-待确认, CONFIRMED-已确认, PROCESSING-处理中, STOCKED-备货完毕, COMPLETED-已完成, CANCELLED-已取消, REJECTED-已拒绝',
  `ordered_amount` decimal(12,2) NOT NULL DEFAULT '0.00' COMMENT '订单总金额',
  `discount_amount` decimal(12,2) NOT NULL DEFAULT '0.00' COMMENT '折扣总金额',
  `net_amount` decimal(12,2) NOT NULL DEFAULT '0.00' COMMENT '实际支付金额',
  `delivery_date` date NOT NULL COMMENT '期望送货日期',
  `delivery_address` varchar(255) NOT NULL COMMENT '送货地址',
  `contact_name` varchar(32) NOT NULL COMMENT '联系人姓名',
  `contact_phone` varchar(16) NOT NULL COMMENT '联系人电话',
  `remark` varchar(255) DEFAULT NULL COMMENT '订单备注',
  `created_by` varchar(32) NOT NULL COMMENT '下单人',
  `confirmed_by` varchar(32) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci DEFAULT NULL COMMENT '供应商订单确认人NORMAL流程',
  `confirmed_at` timestamp NULL DEFAULT NULL COMMENT '确认时间',
  `stocked_by` varchar(32) DEFAULT NULL COMMENT '备货人',
  `stocked_at` timestamp NULL DEFAULT NULL COMMENT '备货时间',
  `processed_by` varchar(32) DEFAULT NULL COMMENT '处理人',
  `processed_at` timestamp NULL DEFAULT NULL COMMENT '处理时间',
  `cancelled_by` varchar(32) DEFAULT NULL COMMENT '取消人',
  `cancelled_at` timestamp NULL DEFAULT NULL COMMENT '取消时间',
  `cancel_reason` varchar(255) DEFAULT NULL COMMENT '取消原因',
  `rejected_by` varchar(32) DEFAULT NULL COMMENT '拒绝人',
  `rejected_at` timestamp NULL DEFAULT NULL COMMENT '拒绝时间',
  `reject_reason` varchar(255) DEFAULT NULL COMMENT '拒绝原因',
  `completed_by` varchar(32) DEFAULT NULL COMMENT '完成人',
  `completed_at` timestamp NULL DEFAULT NULL COMMENT '完成时间',
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  `updated_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
  `deleted_at` timestamp NULL DEFAULT NULL COMMENT '删除时间',
  `delivery_staff_id` int DEFAULT NULL COMMENT '配送员ID',
  `after_sale_at` timestamp NULL DEFAULT NULL,
  `market_contact_number` varchar(11) DEFAULT NULL,
  `assigned_by` varchar(32) DEFAULT NULL,
  `assigned_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_order_code` (`order_code`),
  KEY `idx_customer_status` (`customer_id`,`order_status`),
  KEY `idx_market_status` (`market_id`,`order_status`),
  KEY `idx_delivery_date` (`delivery_date`),
  KEY `idx_created_at` (`created_at`),
  KEY `idx_deleted_at` (`deleted_at`),
  KEY `idx_status_deleted` (`order_status`,`deleted_at`),
  KEY `idx_orders_delivery_staff` (`delivery_staff_id`),
  CONSTRAINT `fk_orders_customer` FOREIGN KEY (`customer_id`) REFERENCES `tenants` (`id`),
  CONSTRAINT `fk_orders_delivery_staff` FOREIGN KEY (`delivery_staff_id`) REFERENCES `delivery_staff` (`id`) ON DELETE SET NULL ON UPDATE CASCADE,
  CONSTRAINT `fk_orders_market` FOREIGN KEY (`market_id`) REFERENCES `tenants` (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='订单主表';
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `permissions`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `permissions` (
  `id` int NOT NULL AUTO_INCREMENT,
  `name` varchar(32) NOT NULL,
  `cname` varchar(16) DEFAULT NULL,
  `description` varchar(32) DEFAULT NULL,
  `http_method` varchar(8) DEFAULT NULL,
  `path_pattern` varchar(64) DEFAULT NULL,
  `self_only` tinyint DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `idx_name` (`name`)
) ENGINE=InnoDB AUTO_INCREMENT=77 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `product_prices`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `product_prices` (
  `id` int unsigned NOT NULL AUTO_INCREMENT COMMENT '主键ID',
  `product_id` int NOT NULL COMMENT '产品ID',
  `price_date` date DEFAULT (curdate()),
  `min_price` decimal(6,2) NOT NULL COMMENT '最低价',
  `min_price_change` decimal(6,2) NOT NULL COMMENT '最低价变化',
  `max_price` decimal(6,2) NOT NULL COMMENT '最高价',
  `max_price_change` decimal(6,2) NOT NULL COMMENT '最高价变化',
  `avg_price` decimal(6,2) NOT NULL COMMENT '平均价',
  `avg_price_change` decimal(6,2) NOT NULL COMMENT '平均价变化',
  `market_id` int NOT NULL COMMENT '所属市场ID',
  `status` varchar(16) NOT NULL DEFAULT 'PENDING' COMMENT '状态：PENDING-待审核, APPROVED-已审核, PUBLISHED-已发布, REJECTED-已拒绝',
  `remark` varchar(32) DEFAULT NULL COMMENT '备注信息',
  `source` varchar(32) NOT NULL DEFAULT 'SYSTEM' COMMENT '价格来源：SYSTEM-系统录入, API-接口导入, MARKET-市场上报',
  `valid_hours` tinyint DEFAULT '24' COMMENT '价格有效时长(小时)',
  `created_by` varchar(16) NOT NULL COMMENT '创建人',
  `approved_by` varchar(16) DEFAULT NULL COMMENT '审核人',
  `approved_at` timestamp NULL DEFAULT NULL COMMENT '审核时间',
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  `updated_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_product_date_market` (`product_id`,`price_date`,`market_id`),
  KEY `idx_price_date` (`price_date`),
  KEY `idx_market_status` (`market_id`,`status`),
  KEY `idx_product_status` (`product_id`,`status`),
  CONSTRAINT `fk_prices_market` FOREIGN KEY (`market_id`) REFERENCES `tenants` (`id`),
  CONSTRAINT `fk_prices_product` FOREIGN KEY (`product_id`) REFERENCES `products` (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=2077 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='产品价格表';
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `product_processing_fee_relations`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `product_processing_fee_relations` (
  `id` int NOT NULL AUTO_INCREMENT COMMENT '主键ID',
  `product_id` int NOT NULL COMMENT '产品ID',
  `processing_fee_id` int NOT NULL COMMENT '加工费用ID',
  `is_default` tinyint(1) DEFAULT '0' COMMENT '是否为默认加工选项',
  `sort_order` smallint DEFAULT '0' COMMENT '排序号',
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  `updated_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_product_processing` (`product_id`,`processing_fee_id`),
  KEY `idx_product_id` (`product_id`),
  KEY `idx_processing_fee_id` (`processing_fee_id`),
  CONSTRAINT `fk_product_processing_fee` FOREIGN KEY (`processing_fee_id`) REFERENCES `product_processing_fees` (`id`) ON DELETE CASCADE,
  CONSTRAINT `fk_product_processing_product` FOREIGN KEY (`product_id`) REFERENCES `products` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='产品加工费用关联表';
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `product_processing_fees`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `product_processing_fees` (
  `id` int NOT NULL AUTO_INCREMENT COMMENT '主键ID',
  `processing_type` varchar(32) NOT NULL COMMENT '加工类型：SKINNING-去皮, DEBONING-剔骨等',
  `fee_type` varchar(16) NOT NULL DEFAULT 'FIXED' COMMENT '费用类型：FIXED-固定金额, PERCENTAGE-百分比',
  `fee_value` decimal(4,2) NOT NULL DEFAULT '0.00' COMMENT '费用值：固定金额时表示具体金额，百分比时表示百分比值',
  `description` varchar(255) DEFAULT NULL COMMENT '说明',
  `is_checkbox` tinyint(1) DEFAULT '0' COMMENT '是否显示为checkbox',
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  `updated_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_product_processing` (`processing_type`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='产品加工费用表';
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `products`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `products` (
  `id` int NOT NULL AUTO_INCREMENT COMMENT '主键ID',
  `category_id` int NOT NULL COMMENT '分类ID',
  `product_code` varchar(10) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL COMMENT '产品唯一编号',
  `name` varchar(100) NOT NULL COMMENT '产品名称',
  `min_order_quantity` decimal(7,2) DEFAULT '1.00' COMMENT '最小起订量',
  `brand` varchar(16) DEFAULT '其他' COMMENT '品牌',
  `unit` varchar(4) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL COMMENT '单位',
  `spec` varchar(64) DEFAULT NULL COMMENT '规格',
  `pricing_method` varchar(32) DEFAULT NULL COMMENT '计价方式',
  `product_description` varchar(96) DEFAULT NULL COMMENT '产品描述',
  `special_notes` varchar(255) DEFAULT NULL COMMENT '特殊说明',
  `tips` varchar(96) DEFAULT NULL COMMENT '小贴士',
  `storage_conditions` varchar(16) DEFAULT NULL COMMENT '存储条件',
  `shelf_life` varchar(8) DEFAULT NULL COMMENT '保质期',
  `tax_rate` decimal(4,2) DEFAULT '0.00' COMMENT '税率',
  `is_disabled` tinyint(1) DEFAULT '0' COMMENT '状态：false-禁用，true-启用',
  `sort_order` smallint DEFAULT '0' COMMENT '排序号',
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  `updated_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
  PRIMARY KEY (`id`),
  UNIQUE KEY `idx_product_code` (`product_code`),
  KEY `idx_category_status` (`category_id`,`is_disabled`)
) ENGINE=InnoDB AUTO_INCREMENT=1393 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='产品表';
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `provider_deliveries`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `provider_deliveries` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT '主键ID',
  `assignment_id` int unsigned NOT NULL COMMENT '关联 provider_orders_assignments.id',
  `parent_id` bigint unsigned DEFAULT NULL,
  `delivery_round` int NOT NULL DEFAULT '1',
  `delivery_type` enum('NORMAL','EXCHANGE','RETURN','OTHER') CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci NOT NULL DEFAULT 'NORMAL',
  `delivered_at` datetime DEFAULT NULL COMMENT '发货时间',
  `delivered_by` varchar(16) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '操作人',
  `delivery_status` enum('PREPARING','DELIVERED','CANCELLED') CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci NOT NULL DEFAULT 'PREPARING' COMMENT '发货状态',
  `remark` varchar(128) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '备注',
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
  `delivery_contact_number` varchar(11) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci NOT NULL,
  PRIMARY KEY (`id`),
  KEY `idx_assignment_id` (`assignment_id`),
  KEY `fk_provider_deliveries_parent` (`parent_id`),
  CONSTRAINT `fk_deliveries_assignment` FOREIGN KEY (`assignment_id`) REFERENCES `provider_orders_assignments` (`id`) ON DELETE CASCADE ON UPDATE CASCADE,
  CONSTRAINT `fk_provider_deliveries_parent` FOREIGN KEY (`parent_id`) REFERENCES `provider_deliveries` (`id`) ON DELETE SET NULL
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='供应商发货记录表（关联订单分配表）';
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `provider_delivery_items`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `provider_delivery_items` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT '主键ID',
  `delivery_id` bigint unsigned NOT NULL COMMENT '对应 provider_deliveries.id',
  `order_detail_id` bigint unsigned NOT NULL COMMENT '订单明细ID',
  `product_code` varchar(10) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL,
  `actual_qty` decimal(10,2) NOT NULL DEFAULT '0.00' COMMENT '实际发货数量',
  `unit_price` decimal(10,2) NOT NULL COMMENT '单价',
  `subtotal` decimal(12,2) GENERATED ALWAYS AS ((`actual_qty` * `unit_price`)) STORED COMMENT '小计',
  `weight_unit` varchar(8) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci DEFAULT 'kg' COMMENT '计量单位',
  `remark` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '备注',
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
  PRIMARY KEY (`id`),
  KEY `idx_delivery_id` (`delivery_id`),
  KEY `fk_delivery_items_product` (`product_code`),
  CONSTRAINT `fk_delivery_items_delivery` FOREIGN KEY (`delivery_id`) REFERENCES `provider_deliveries` (`id`) ON DELETE CASCADE ON UPDATE CASCADE,
  CONSTRAINT `fk_delivery_items_product` FOREIGN KEY (`product_code`) REFERENCES `products` (`product_code`) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB AUTO_INCREMENT=4 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='供应商发货明细表（商品维度）';
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `provider_financial_profiles`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `provider_financial_profiles` (
  `id` int NOT NULL AUTO_INCREMENT,
  `tenant_id` int NOT NULL,
  `credit_score` decimal(10,2) DEFAULT NULL,
  `credit_limit` decimal(15,2) DEFAULT NULL,
  `deposit_amount` decimal(15,2) DEFAULT NULL,
  `payment_period_days` int DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  `deleted_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_tenant_id` (`tenant_id`),
  KEY `idx_deleted_at` (`deleted_at`),
  CONSTRAINT `fk_supplier_financial_tenant` FOREIGN KEY (`tenant_id`) REFERENCES `tenants` (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `provider_orders_assignments`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `provider_orders_assignments` (
  `id` int unsigned NOT NULL AUTO_INCREMENT,
  `provider_id` int NOT NULL,
  `order_id` int NOT NULL,
  `created_at` datetime DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  KEY `idx_provider_order` (`provider_id`,`order_id`)
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `reconciliation_statement_details`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `reconciliation_statement_details` (
  `id` int NOT NULL AUTO_INCREMENT,
  `statement_id` int NOT NULL COMMENT '对账单ID',
  `order_detail_id` int NOT NULL COMMENT '订单明细ID',
  `product_code` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '产品编号',
  `product_name` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '产品名称',
  `category_name` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '分类名称',
  `unit` varchar(50) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '单位',
  `original_quantity` decimal(18,2) NOT NULL COMMENT '原始数量',
  `actual_quantity` decimal(18,2) NOT NULL COMMENT '实际数量',
  `receipt_quantity` decimal(18,2) NOT NULL COMMENT '收货数量',
  `returned_quantity` decimal(18,2) NOT NULL COMMENT '退货数量',
  `price` decimal(18,2) NOT NULL COMMENT '单价',
  `original_amount` decimal(18,2) NOT NULL COMMENT '原始金额',
  `actual_amount` decimal(18,2) NOT NULL COMMENT '实际金额',
  `order_date` date NOT NULL COMMENT '订单日期',
  `created_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  PRIMARY KEY (`id`),
  KEY `idx_statement_id` (`statement_id`),
  KEY `idx_order_detail_id` (`order_detail_id`),
  CONSTRAINT `fk_order_detail` FOREIGN KEY (`order_detail_id`) REFERENCES `order_details` (`id`) ON DELETE CASCADE ON UPDATE CASCADE,
  CONSTRAINT `fk_statement_detail` FOREIGN KEY (`statement_id`) REFERENCES `reconciliation_statements` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='对账单明细表';
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `reconciliation_statement_orders`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `reconciliation_statement_orders` (
  `id` int NOT NULL AUTO_INCREMENT,
  `statement_id` int NOT NULL COMMENT '对账单ID',
  `order_id` int NOT NULL COMMENT '订单ID',
  `order_code` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '订单编号',
  `order_date` date NOT NULL COMMENT '订单日期',
  `total_amount` decimal(18,2) NOT NULL COMMENT '订单总金额',
  `actual_amount` decimal(18,2) NOT NULL COMMENT '实际金额',
  `created_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  PRIMARY KEY (`id`),
  KEY `idx_statement_id` (`statement_id`),
  KEY `idx_order_id` (`order_id`),
  CONSTRAINT `fk_order` FOREIGN KEY (`order_id`) REFERENCES `orders` (`id`) ON DELETE RESTRICT ON UPDATE CASCADE,
  CONSTRAINT `fk_statement_order` FOREIGN KEY (`statement_id`) REFERENCES `reconciliation_statements` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='对账单订单表';
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `reconciliation_statements`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `reconciliation_statements` (
  `id` int NOT NULL AUTO_INCREMENT,
  `statement_code` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '对账单编号',
  `customer_id` int NOT NULL COMMENT '客户ID',
  `market_id` int NOT NULL COMMENT '市场ID',
  `provider_id` int NOT NULL COMMENT '供应商ID',
  `supplier_name` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '供应商名称',
  `customer_name` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '客户名称',
  `start_date` date NOT NULL COMMENT '开始日期',
  `end_date` date NOT NULL COMMENT '结束日期',
  `total_amount` decimal(18,2) NOT NULL COMMENT '总金额',
  `discount_amount` decimal(18,2) NOT NULL COMMENT '折扣金额',
  `actual_amount` decimal(18,2) NOT NULL COMMENT '实际金额',
  `status` varchar(50) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '状态',
  `remark` text CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci COMMENT '备注',
  `created_by` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '创建人',
  `confirmed_by` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '确认人',
  `confirmed_at` timestamp NULL DEFAULT NULL COMMENT '确认时间',
  `completed_by` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '完成处理人',
  `completed_at` timestamp NULL DEFAULT NULL COMMENT '完成时间',
  `created_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  `updated_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
  `deleted_at` timestamp NULL DEFAULT NULL COMMENT '删除时间',
  PRIMARY KEY (`id`),
  KEY `fk_customer` (`customer_id`),
  KEY `fk_market` (`market_id`),
  KEY `fk_provider` (`provider_id`),
  CONSTRAINT `fk_customer` FOREIGN KEY (`customer_id`) REFERENCES `tenants` (`id`) ON DELETE RESTRICT ON UPDATE CASCADE,
  CONSTRAINT `fk_market` FOREIGN KEY (`market_id`) REFERENCES `tenants` (`id`) ON DELETE RESTRICT ON UPDATE CASCADE,
  CONSTRAINT `fk_provider` FOREIGN KEY (`provider_id`) REFERENCES `tenants` (`id`) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='对账单表';
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `refund_records`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `refund_records` (
  `id` int NOT NULL AUTO_INCREMENT,
  `return_exchange_id` int NOT NULL COMMENT '关联的退换货记录ID',
  `amount` decimal(12,2) NOT NULL COMMENT '退款金额',
  `method` varchar(16) NOT NULL COMMENT '退款方式: ORIGINAL-原路退回, BALANCE-退回余额等',
  `transaction_id` varchar(64) DEFAULT NULL COMMENT '交易ID',
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  `updated_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
  PRIMARY KEY (`id`),
  KEY `fk_refund_return_exchange` (`return_exchange_id`),
  CONSTRAINT `fk_refund_return_exchange` FOREIGN KEY (`return_exchange_id`) REFERENCES `return_exchange_records` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='退款记录表';
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `return_exchange_records`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `return_exchange_records` (
  `id` int NOT NULL AUTO_INCREMENT,
  `order_detail_id` int NOT NULL COMMENT '关联的订单明细ID',
  `inspection_id` int unsigned NOT NULL,
  `operation_type` varchar(16) NOT NULL COMMENT '类型: RETURN-退货, EXCHANGE-换货',
  `quantity` decimal(10,2) NOT NULL COMMENT '退换数量',
  `reason` varchar(32) NOT NULL COMMENT '原因: DAMAGED-损坏, WRONG_ITEM-错误商品, QUALITY_ISSUE-质量问题等',
  `reason_description` text COMMENT '原因详细描述',
  `status` varchar(16) NOT NULL DEFAULT 'PENDING' COMMENT '状态: PENDING-待处理, APPROVED-已批准, REJECTED-已拒绝, COMPLETED-已完成',
  `evidence_images` text COMMENT '证据图片(JSON数组存储URL)',
  `processed_by` varchar(32) DEFAULT NULL COMMENT '处理人',
  `processed_at` timestamp NULL DEFAULT NULL COMMENT '处理时间',
  `created_by` varchar(32) NOT NULL COMMENT '创建人',
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  `updated_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
  `actual_quantity` decimal(10,2) DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `fk_return_exchange_order_detail` (`order_detail_id`),
  KEY `fk_rer_inspection` (`inspection_id`),
  CONSTRAINT `fk_rer_inspection` FOREIGN KEY (`inspection_id`) REFERENCES `order_inspections` (`id`),
  CONSTRAINT `fk_return_exchange_order_detail` FOREIGN KEY (`order_detail_id`) REFERENCES `order_details` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB AUTO_INCREMENT=3 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='退换货记录表';
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `role_menu`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `role_menu` (
  `role_id` int NOT NULL COMMENT '角色ID',
  `menu_id` int NOT NULL COMMENT '菜单ID',
  PRIMARY KEY (`role_id`,`menu_id`),
  KEY `menu_id` (`menu_id`),
  CONSTRAINT `role_menu_ibfk_1` FOREIGN KEY (`role_id`) REFERENCES `roles` (`id`) ON DELETE CASCADE,
  CONSTRAINT `role_menu_ibfk_2` FOREIGN KEY (`menu_id`) REFERENCES `menu_config` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='角色菜单关联表';
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `role_permissions`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `role_permissions` (
  `role_id` int NOT NULL,
  `permission_id` int NOT NULL,
  PRIMARY KEY (`role_id`,`permission_id`),
  KEY `fk_role_permissions_permission` (`permission_id`),
  CONSTRAINT `fk_role_permissions_permission` FOREIGN KEY (`permission_id`) REFERENCES `permissions` (`id`),
  CONSTRAINT `fk_role_permissions_role` FOREIGN KEY (`role_id`) REFERENCES `roles` (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `roles`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `roles` (
  `id` int NOT NULL AUTO_INCREMENT,
  `name` varchar(32) NOT NULL,
  `tenant_type` varchar(16) DEFAULT NULL,
  `alias_name` varchar(32) DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_name` (`name`)
) ENGINE=InnoDB AUTO_INCREMENT=15 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `temp_image_urls`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `temp_image_urls` (
  `id` int NOT NULL AUTO_INCREMENT,
  `object_key` varchar(512) NOT NULL,
  `product_code` varchar(10) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL,
  `temp_url` text NOT NULL,
  `expires_at` datetime NOT NULL,
  `created_at` datetime DEFAULT CURRENT_TIMESTAMP,
  `updated_at` datetime DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  `bucket_name` varchar(100) NOT NULL DEFAULT 'product',
  PRIMARY KEY (`id`),
  UNIQUE KEY `object_key` (`object_key`),
  UNIQUE KEY `product_code` (`product_code`),
  KEY `idx_bucket_name` (`bucket_name`)
) ENGINE=InnoDB AUTO_INCREMENT=44382 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `tenant_relationships`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `tenant_relationships` (
  `id` int NOT NULL AUTO_INCREMENT,
  `market_id` int NOT NULL,
  `provider_id` int NOT NULL,
  `status` varchar(16) NOT NULL DEFAULT 'PENDING',
  `created_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  KEY `idx_market_id` (`market_id`),
  KEY `idx_provider_id` (`provider_id`),
  CONSTRAINT `fk_market_providers_market` FOREIGN KEY (`market_id`) REFERENCES `tenants` (`id`),
  CONSTRAINT `fk_market_providers_provider` FOREIGN KEY (`provider_id`) REFERENCES `tenants` (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=6 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `tenants`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `tenants` (
  `id` int NOT NULL AUTO_INCREMENT,
  `name` varchar(255) NOT NULL,
  `name_hash` varchar(8) NOT NULL,
  `address` varchar(32) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL,
  `license_image` varchar(255) DEFAULT NULL,
  `status` varchar(12) NOT NULL DEFAULT 'PENDING',
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  `tenant_type` varchar(16) NOT NULL,
  `encrypted_fields` json DEFAULT NULL,
  `business_scope` varchar(32) DEFAULT NULL,
  `is_special_tenant` tinyint(1) DEFAULT '0',
  `verified_at` timestamp NULL DEFAULT NULL,
  `deleted_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `name_hash` (`name_hash`),
  KEY `idx_status` (`status`),
  KEY `idx_deleted_at` (`deleted_at`),
  KEY `idx_name_hash_type` (`name_hash`,`tenant_type`,`deleted_at`)
) ENGINE=InnoDB AUTO_INCREMENT=7 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `user_category_assignments`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `user_category_assignments` (
  `id` int NOT NULL AUTO_INCREMENT,
  `user_id` int NOT NULL,
  `category_id` int NOT NULL,
  `created_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `user_id` (`user_id`,`category_id`),
  KEY `category_id` (`category_id`),
  CONSTRAINT `user_category_assignments_ibfk_1` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE,
  CONSTRAINT `user_category_assignments_ibfk_2` FOREIGN KEY (`category_id`) REFERENCES `categories` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB AUTO_INCREMENT=16 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `user_roles`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `user_roles` (
  `user_id` int NOT NULL,
  `role_id` int NOT NULL,
  `alias_name` varchar(32) DEFAULT NULL,
  PRIMARY KEY (`user_id`,`role_id`),
  KEY `fk_user_roles_role` (`role_id`),
  CONSTRAINT `fk_user_roles_role` FOREIGN KEY (`role_id`) REFERENCES `roles` (`id`),
  CONSTRAINT `fk_user_roles_user` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `users`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `users` (
  `id` int NOT NULL AUTO_INCREMENT,
  `name` varchar(16) NOT NULL,
  `username` varchar(16) NOT NULL,
  `password_hash` varchar(255) NOT NULL,
  `email` varchar(128) DEFAULT NULL,
  `phone` varchar(16) DEFAULT NULL,
  `tenant_id` int DEFAULT NULL,
  `is_super_admin` tinyint(1) NOT NULL DEFAULT '0',
  `deleted_at` timestamp NULL DEFAULT NULL,
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_username` (`username`),
  KEY `idx_tenant_id` (`tenant_id`),
  KEY `idx_deleted_at` (`deleted_at`)
) ENGINE=InnoDB AUTO_INCREMENT=29 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

