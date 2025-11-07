-- 如果存在则删除数据库
DROP DATABASE IF EXISTS tenant_saas;

-- 创建数据库
CREATE DATABASE IF NOT EXISTS tenant_saas
DEFAULT CHARACTER SET utf8mb4
DEFAULT COLLATE utf8mb4_unicode_ci;

-- 使用数据库
USE tenant_saas;

-- 用户表
CREATE TABLE users (
    id INT NOT NULL AUTO_INCREMENT,
    name VARCHAR(16) NOT NULL,
    username VARCHAR(16) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    email VARCHAR(128),
    phone VARCHAR(16),
    tenant_id INT,
    is_super_admin TINYINT(1) DEFAULT 0,
    deleted_at TIMESTAMP NULL DEFAULT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    UNIQUE KEY uk_username (username),
    KEY idx_tenant_id (tenant_id),
    KEY idx_deleted_at (deleted_at)
);

-- 租户表
CREATE TABLE tenants (
    id INT NOT NULL AUTO_INCREMENT,
    name VARCHAR(255) NOT NULL,
    name_hash VARCHAR(8) NOT NULL UNIQUE,
    address VARCHAR(32),
    license_image VARCHAR(255),
    status VARCHAR(12) NOT NULL DEFAULT 'PENDING', -- PENDING, APPROVED, ACTIVE, REJECTED
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    tenant_type VARCHAR(16) NOT NULL, -- MARKET, PROVIDER, CUSTOMER
    encrypted_fields JSON,
    business_scope VARCHAR(32),
    is_special_tenant TINYINT(1) DEFAULT 0, -- 0表示普通租户，1表示特殊租户（政府机关） 为了采购时不显示产品价格
    verified_at TIMESTAMP NULL,
    deleted_at TIMESTAMP NULL DEFAULT NULL,
    PRIMARY KEY (id),
    KEY idx_status (status),
    KEY idx_deleted_at (deleted_at)
);

-- 角色表
CREATE TABLE roles (
    id INT NOT NULL AUTO_INCREMENT,
    name VARCHAR(32) NOT NULL,
    tenant_type VARCHAR(16), -- MARKET, PROVIDER, CUSTOMER, null表示通用角色
    alias_name VARCHAR(32),
    PRIMARY KEY (id),
    UNIQUE KEY uk_name (name)
);

-- 权限表
CREATE TABLE permissions (
    id INT NOT NULL AUTO_INCREMENT,
    name VARCHAR(32) NOT NULL,
    PRIMARY KEY (id),
    UNIQUE KEY uk_name (name)
);

-- 用户角色关联表
CREATE TABLE user_roles (
    user_id INT NOT NULL,
    role_id INT NOT NULL,
    alias_name VARCHAR(32),
    PRIMARY KEY (user_id, role_id),
    CONSTRAINT fk_user_roles_user FOREIGN KEY (user_id) REFERENCES users(id),
    CONSTRAINT fk_user_roles_role FOREIGN KEY (role_id) REFERENCES roles(id)
);

-- 角色权限关联表
CREATE TABLE role_permissions (
    role_id INT NOT NULL,
    permission_id INT NOT NULL,
    PRIMARY KEY (role_id, permission_id),
    CONSTRAINT fk_role_permissions_role FOREIGN KEY (role_id) REFERENCES roles(id),
    CONSTRAINT fk_role_permissions_permission FOREIGN KEY (permission_id) REFERENCES permissions(id)
);

-- 租户关系表
CREATE TABLE tenant_relationships (
    id INT NOT NULL AUTO_INCREMENT,
    market_id INT NOT NULL,
    provider_id INT NOT NULL,
    status VARCHAR(16) NOT NULL DEFAULT 'PENDING',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    KEY idx_market_id (market_id),
    KEY idx_provider_id (provider_id),
    CONSTRAINT fk_market_providers_market FOREIGN KEY (market_id) REFERENCES tenants(id),
    CONSTRAINT fk_market_providers_provider FOREIGN KEY (provider_id) REFERENCES tenants(id)
);

-- 供应商财务档案表
CREATE TABLE provider_financial_profiles (
    id INT NOT NULL AUTO_INCREMENT,
    tenant_id INT NOT NULL,
    credit_score DECIMAL(10,2),
    credit_limit DECIMAL(15,2),
    deposit_amount DECIMAL(15,2),
    payment_period_days INT,
    created_at TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP NULL DEFAULT NULL,
    PRIMARY KEY (id),
    UNIQUE KEY uk_tenant_id (tenant_id),
    KEY idx_deleted_at (deleted_at),
    CONSTRAINT fk_supplier_financial_tenant FOREIGN KEY (tenant_id) REFERENCES tenants(id)
);

-- 消息表
CREATE TABLE messages (
    id INT NOT NULL AUTO_INCREMENT,
    user_id INT NOT NULL,
    content TEXT NOT NULL,
    is_read TINYINT(1) DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    KEY idx_user_id (user_id),
    CONSTRAINT fk_messages_user FOREIGN KEY (user_id) REFERENCES users(id)
);

-- 初始化权限数据
INSERT INTO permissions (name) VALUES
('tenant:create'),      -- 创建租户
    ('tenant:read'),        -- 查看租户详情
    ('tenant:disable'),     -- 禁用租户
    ('tenant:update'),      -- 更新租户信息
    ('tenant:list'),        -- 查看租户列表
    ('tenant:verify'),      -- 审核租户资质
    ('tenant:financial'),   -- 管理租户财务信息
    ('user:create'),        -- 创建用户
    ('user:read'),          -- 查看用户详情
    ('user:disable'),       -- 禁用用户
    ('user:update'),        -- 更新用户信息
    ('user:list'),          -- 查看用户列表
    ('user:role'),          -- 管理用户角色
    ('price:create'),       -- 创建价格
    ('price:read'),         -- 查看价格详情
    ('price:approve'),      -- 审批价格
    ('price:update'),       -- 更新价格
    ('price:list'),         -- 查看价格列表
    ('price:history'),      -- 查看价格历史
    ('price:publish'),      -- 发布价格
    ('order:create'),       -- 创建订单
    ('order:read'),         -- 查看订单详情
    ('order:update'),       -- 更新订单
    ('order:cancel'),       -- 取消订单
    ('order:approve'),      -- 审批订单
    ('order:reject'),       -- 拒绝订单
    ('order:publish'),      -- 发布订单
    ('order:list'),         -- 查看订单列表
    ('order:export'),       -- 导出订单
    ('order:comment'),      -- 订单评论/备注
    ('order:track'),        -- 订单跟踪
    ('report:view'),        -- 查看报表
    ('report:export'),      -- 导出报表
    ('report:create'),      -- 创建报表
    ('message:send'),       -- 发送消息
    ('message:read'),       -- 读取消息
    ('message:update'),     -- 更新消息
    ('message:manage'),     -- 管理消息
    ('system:config'),      -- 系统配置
    ('system:log'),         -- 系统日志
    ('system:backup'),      -- 系统备份
    ('audit:view'),         -- 查看审计日志
    ('audit:export');       -- 导出审计日志

-- 初始化角色数据
INSERT INTO roles (name, tenant_type, alias_name) VALUES
    ('SUPER_ADMIN', null, '超级管理员'),            -- 系统管理员，拥有所有权限
    ('MARKET_ADMIN', 'MARKET', '市场管理员'),        -- 市场管理员，负责管理市场运营
    ('PROVIDER_ADMIN', 'PROVIDER', '供应商管理员'),  -- 供应商管理员，管理供应商相关业务
    ('CUSTOMER_ADMIN', 'CUSTOMER', '客户管理员'),    -- 客户管理员，管理客户相关业务
    ('PRICER', 'MARKET', '询价员'),                 -- 询价员，负责商品定价管理
    ('AUDITOR', 'MARKET', '价格审计员'),             -- 价格审计员，负责商品价格公示审计工作
    ('ORDER_CREATOR', 'CUSTOMER', '采购员'),        -- 采购员，可以创建和管理订单
    ('ORDER_PUBLISHER', 'MARKET', '订单发布者'),     -- 订单发布者，负责订单的发布和跟踪（市场根据订单进行拆分成子订单发布）
    ('ORDER_VIEWER', null, '订单查看者'),            -- 订单查看者，只能查看订单信息
    ('FINANCE_ADMIN', null, '财务管理员'),           -- 财务管理员，负责财务相关操作
    ('REPORT_VIEWER', null, '报表查看者'),           -- 报表查看者，可以查看和导出报表
    ('CUSTOMER_SERVICE', 'MARKET', '客服人员'),      -- 客服人员，处理客户服务相关工作
    ('SYSTEM_OPERATOR', null, '系统操作员');          -- 系统操作员，负责日常系统运维工作


-- 为系统管理员分配所有权限
-- SYSTEM_ADMIN (超级管理员，拥有所有权限)
INSERT INTO role_permissions (role_id, permission_id)
SELECT 
    (SELECT id FROM roles WHERE name = 'SUPER_ADMIN'),
    id
FROM permissions;



-- MARKET_ADMIN (市场管理员)
INSERT INTO role_permissions (role_id, permission_id)
SELECT 
    (SELECT id FROM roles WHERE name = 'MARKET_ADMIN'),
    id
FROM permissions 
WHERE name IN (
    'tenant:create', 'tenant:read', 'tenant:update',
    'user:create', 'user:read', 'user:update', 'user:list', 'user:role', 'user:disable',
    'report:view', 'report:export', 'message:read', 'message:update',
    'audit:view'
);

-- PROVIDER_ADMIN (供应商管理员)
INSERT INTO role_permissions (role_id, permission_id)
SELECT 
    (SELECT id FROM roles WHERE name = 'PROVIDER_ADMIN'),
    id
FROM permissions 
WHERE name IN (
    'tenant:create', 'tenant:read', 'tenant:update',
    'user:create', 'user:read', 'user:update', 'user:list', 'user:role', 'user:disable',
    'report:view', 'report:export', 'message:read', 'message:update',
    'audit:view'
);

-- CUSTOMER_ADMIN (客户管理员)
INSERT INTO role_permissions (role_id, permission_id)
SELECT 
    (SELECT id FROM roles WHERE name = 'CUSTOMER_ADMIN'),
    id
FROM permissions 
WHERE name IN (
    'tenant:create', 'tenant:read', 'tenant:update',
    'user:create', 'user:read', 'user:update', 'user:list', 'user:role', 'user:disable',
    'report:view', 'report:export', 'message:read', 'message:update',
    'audit:view'
);

-- PRICER (定价员)
INSERT INTO role_permissions (role_id, permission_id)
SELECT 
    (SELECT id FROM roles WHERE name = 'PRICER'),
    id
FROM permissions 
WHERE name IN (
    'price:create', 'price:read', 'price:update', 'price:list', 'price:history',
    'message:read', 'message:update','message:list','tenant:read','user:read'
);

-- AUDITOR (审计员)
INSERT INTO role_permissions (role_id, permission_id)
SELECT 
    (SELECT id FROM roles WHERE name = 'AUDITOR'),
    id
FROM permissions 
WHERE name IN (
    'price:read', 'price:list', 'price:history', 'price:publish', 'price:approve', 'tenant:read','user:read'
);

-- ORDER_CREATOR (采购员)
INSERT INTO role_permissions (role_id, permission_id)
SELECT 
    (SELECT id FROM roles WHERE name = 'ORDER_CREATOR'),
    id
FROM permissions 
WHERE name IN (
    'order:create', 'order:read', 'order:update', 'order:list', 'order:track', 'order:cancel','tenant:read','user:read'
    'price:read', 'price:list'
);

-- ORDER_PUBLISHER (订单发布者-市场根据订单进行拆分成子订单发布)
INSERT INTO role_permissions (role_id, permission_id)
SELECT 
    (SELECT id FROM roles WHERE name = 'ORDER_PUBLISHER'),
    id
FROM permissions 
WHERE name IN (
    'order:read', 'order:publish', 'order:list', 'order:track', 'order:reject',
    'price:read', 'price:list'
);

-- ORDER_VIEWER (订单查看者)
INSERT INTO role_permissions (role_id, permission_id)
SELECT 
    (SELECT id FROM roles WHERE name = 'ORDER_VIEWER'),
    id
FROM permissions 
WHERE name IN (
    'order:read', 'order:list',
    'price:read', 'price:list'
);

-- FINANCE_ADMIN (财务管理员)
INSERT INTO role_permissions (role_id, permission_id)
SELECT 
    (SELECT id FROM roles WHERE name = 'FINANCE_ADMIN'),
    id
FROM permissions 
WHERE name IN (
    'tenant:financial', 'tenant:read', 'tenant:list',
    'order:read', 'order:list', 'order:export',
    'report:view', 'report:export',
    'price:read', 'price:list'
);

-- REPORT_VIEWER (报表查看者)
INSERT INTO role_permissions (role_id, permission_id)
SELECT 
    (SELECT id FROM roles WHERE name = 'REPORT_VIEWER'),
    id
FROM permissions 
WHERE name IN (
    'report:view', 'report:export'
);

-- CUSTOMER_SERVICE (客服人员)
INSERT INTO role_permissions (role_id, permission_id)
SELECT 
    (SELECT id FROM roles WHERE name = 'CUSTOMER_SERVICE'),
    id
FROM permissions 
WHERE name IN (
    'order:read', 'order:list', 'order:track', 'order:comment',
    'message:send', 'message:read',
    'tenant:read', 'tenant:list',
    'price:read', 'price:list'
);

-- 创建分类表
CREATE TABLE `categories` (
    `id` INT NOT NULL AUTO_INCREMENT,
    `name` varchar(50) NOT NULL COMMENT '分类名称',
    `level` TINYINT NOT NULL COMMENT '分类层级：1-一级分类(肉类), 2-二级分类(牛肉类), 3-三级分类(半肥半瘦牛肉馅)',
    `parent_id` INT DEFAULT NULL COMMENT '父级分类ID',
    `sort_order` SMALLINT DEFAULT 0 COMMENT '排序号',
    PRIMARY KEY (`id`),
    KEY `idx_parent_id` (`parent_id`),
    KEY `idx_level_sort` (`level`, `sort_order`) -- 组合索引
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='产品分类表';


-- 用户分类关联表
CREATE TABLE user_category_assignments (
    id INT AUTO_INCREMENT PRIMARY KEY,
    user_id INT NOT NULL,
    category_id INT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE,
    UNIQUE KEY (user_id, category_id)
);

-- 创建产品表
CREATE TABLE `products` (
    `id` INT NOT NULL AUTO_INCREMENT COMMENT '主键ID',
    `category_id` INT NOT NULL COMMENT '分类ID',
    `product_code` VARCHAR(10) COMMENT '产品唯一编号',
    `name` varchar(100) NOT NULL COMMENT '产品名称',
    `min_order_quantity` DECIMAL(7,2) DEFAULT 1 COMMENT '最小起订量',
    `brand` varchar(16) DEFAULT '其他' COMMENT '品牌',
    `unit` varchar(4) DEFAULT NULL COMMENT '单位',
    `spec` varchar(64) DEFAULT NULL COMMENT '规格',
    `pricing_method` varchar(32) DEFAULT NULL COMMENT '计价方式',
    `product_description` varchar(96) DEFAULT NULL COMMENT '产品描述',
    `special_notes` varchar(255) DEFAULT NULL COMMENT '特殊说明',
    `tips` varchar(96) DEFAULT NULL COMMENT '小贴士',
    `storage_conditions` varchar(16) DEFAULT NULL COMMENT '存储条件',
    `shelf_life` varchar(8) DEFAULT NULL COMMENT '保质期',
    `tax_rate` DECIMAL(4,2) DEFAULT 0 COMMENT '税率',
    `is_disabled` boolean DEFAULT false COMMENT '状态：false-禁用，true-启用',
    `sort_order` SMALLINT DEFAULT 0 COMMENT '排序号',
    `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    `updated_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    PRIMARY KEY (`id`),
    UNIQUE INDEX `idx_product_code` (`product_code`),
    KEY `idx_category_status` (`category_id`, `is_disabled`) -- 组合索引s
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='产品表';



-- 产品价格表
CREATE TABLE `product_prices` (
    `id` INT UNSIGNED NOT NULL AUTO_INCREMENT COMMENT '主键ID',
    `product_id` INT NOT NULL COMMENT '产品ID',
    `price_date` DATE COMMENT '价格日期',
    `min_price` DECIMAL(6,2) NOT NULL COMMENT '最低价',
    `min_price_change` DECIMAL(6,2) NOT NULL COMMENT '最低价变化',
    `max_price` DECIMAL(6,2) NOT NULL COMMENT '最高价',
    `max_price_change` DECIMAL(6,2) NOT NULL COMMENT '最高价变化',
    `avg_price` DECIMAL(6,2) NOT NULL COMMENT '平均价',
    `avg_price_change` DECIMAL(6,2) NOT NULL COMMENT '平均价变化',
    `market_id` INT NOT NULL COMMENT '所属市场ID',
    `status` VARCHAR(16) NOT NULL DEFAULT 'PENDING' COMMENT '状态：PENDING-待审核, APPROVED-已审核, PUBLISHED-已发布, REJECTED-已拒绝',
    `remark` VARCHAR(32) DEFAULT NULL COMMENT '备注信息',
    `source` VARCHAR(32) NOT NULL DEFAULT 'SYSTEM' COMMENT '价格来源：SYSTEM-系统录入, API-接口导入, MARKET-市场上报',
    `valid_hours` TINYINT DEFAULT 24 COMMENT '价格有效时长(小时)',
    `created_by` VARCHAR(16) NOT NULL COMMENT '创建人',
    `approved_by` VARCHAR(16) DEFAULT NULL COMMENT '审核人',
    `approved_at` TIMESTAMP NULL DEFAULT NULL COMMENT '审核时间',
    `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    `updated_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    
    PRIMARY KEY (`id`),
    UNIQUE KEY `uk_product_date_market` (`product_id`, `price_date`, `market_id`),
    KEY `idx_price_date` (`price_date`),
    KEY `idx_market_status` (`market_id`, `status`),
    KEY `idx_product_status` (`product_id`, `status`),
    CONSTRAINT `fk_prices_product` FOREIGN KEY (`product_id`) REFERENCES `products` (`id`),
    CONSTRAINT `fk_prices_market` FOREIGN KEY (`market_id`) REFERENCES `tenants` (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='产品价格表';
ALTER TABLE product_prices MODIFY COLUMN price_date DATE DEFAULT (CURRENT_DATE);

-- 产品加工费用表
CREATE TABLE `product_processing_fees` (
    `id` INT NOT NULL AUTO_INCREMENT COMMENT '主键ID',
    `processing_type` varchar(32) NOT NULL COMMENT '加工类型：SKINNING-去皮, DEBONING-剔骨等',
    `fee_type` varchar(16) NOT NULL DEFAULT 'FIXED' COMMENT '费用类型：FIXED-固定金额, PERCENTAGE-百分比',
    `fee_value` DECIMAL(4,2) NOT NULL DEFAULT 0 COMMENT '费用值：固定金额时表示具体金额，百分比时表示百分比值',
    `description` varchar(255) DEFAULT NULL COMMENT '说明',
    `is_checkbox` boolean DEFAULT false COMMENT '是否显示为checkbox',
    `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    `updated_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    PRIMARY KEY (`id`),
    UNIQUE KEY `uk_product_processing` (`processing_type`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='产品加工费用表';

CREATE TABLE `product_processing_fee_relations` (
    `id` INT NOT NULL AUTO_INCREMENT COMMENT '主键ID',
    `product_id` INT NOT NULL COMMENT '产品ID',
    `processing_fee_id` INT NOT NULL COMMENT '加工费用ID',
    `is_default` BOOLEAN DEFAULT FALSE COMMENT '是否为默认加工选项',
    `sort_order` SMALLINT DEFAULT 0 COMMENT '排序号',
    `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    `updated_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    PRIMARY KEY (`id`),
    UNIQUE KEY `uk_product_processing` (`product_id`, `processing_fee_id`),
    KEY `idx_product_id` (`product_id`),
    KEY `idx_processing_fee_id` (`processing_fee_id`),
    CONSTRAINT `fk_product_processing_product` FOREIGN KEY (`product_id`) REFERENCES `products` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_product_processing_fee` FOREIGN KEY (`processing_fee_id`) REFERENCES `product_processing_fees` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='产品加工费用关联表';

-- 客户分类折扣表
CREATE TABLE `customer_category_discounts` (
    `id` INT NOT NULL AUTO_INCREMENT COMMENT '主键ID',
    `tenant_id` INT NOT NULL COMMENT '客户租户ID',
    `category_id` INT NOT NULL COMMENT '一级分类ID',
    `discount_rate` DECIMAL(4,2) NOT NULL COMMENT '折扣率：0.8表示8折',
    `start_date` DATE NOT NULL COMMENT '生效开始日期',
    `end_date` DATE NULL COMMENT '生效结束日期，NULL表示永久有效',
    `status` BOOLEAN NOT NULL DEFAULT true COMMENT '状态：true-生效中, false-已失效',
    `remark` VARCHAR(255) DEFAULT NULL COMMENT '备注说明',
    `created_by` VARCHAR(32) NOT NULL COMMENT '创建人',
    `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    `updated_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    
    PRIMARY KEY (`id`),
    UNIQUE KEY `uk_tenant_category` (`tenant_id`, `category_id`, `start_date`),
    KEY `idx_category_status` (`category_id`, `status`),
    KEY `idx_tenant_status` (`tenant_id`, `status`),
    KEY `idx_date_status` (`start_date`, `end_date`, `status`),
    
    CONSTRAINT `fk_customer_discounts_tenant` FOREIGN KEY (`tenant_id`) REFERENCES `tenants` (`id`),
    CONSTRAINT `fk_customer_discounts_category` FOREIGN KEY (`category_id`) REFERENCES `categories` (`id`),
    CONSTRAINT `chk_discount_rate` CHECK (`discount_rate` > 0 AND `discount_rate` <= 1)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='客户分类折扣表';

-- 折扣历史记录表（可选，用于记录折扣变更历史）
CREATE TABLE `customer_category_discount_history` (
    `id` INT NOT NULL AUTO_INCREMENT COMMENT '主键ID',
    `tenant_id` INT NOT NULL COMMENT '客户租户ID',
    `category_id` INT NOT NULL COMMENT '一级分类ID',
    `old_discount_rate` DECIMAL(4,3) COMMENT '原折扣率',
    `new_discount_rate` DECIMAL(4,3) NOT NULL COMMENT '新折扣率',
    `change_type` VARCHAR(16) NOT NULL COMMENT '变更类型：CREATE-新建, UPDATE-修改, EXPIRE-失效',
    `change_reason` VARCHAR(255) DEFAULT NULL COMMENT '变更原因',
    `changed_by` VARCHAR(32) NOT NULL COMMENT '操作人',
    `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    
    PRIMARY KEY (`id`),
    KEY `idx_tenant_category` (`tenant_id`, `category_id`),
    KEY `idx_created_at` (`created_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='客户分类折扣历史表';


-- 订单主表
CREATE TABLE `orders` (
    `id` INT NOT NULL AUTO_INCREMENT COMMENT '主键ID',
    `order_code` VARCHAR(32) NOT NULL COMMENT '订单编号：DD-时间戳-4位随机数',
    `customer_id` INT NOT NULL COMMENT '客户租户ID',
    `market_id` INT NOT NULL COMMENT '市场租户ID',
    `order_status` VARCHAR(16) NOT NULL DEFAULT 'PENDING',
    `total_amount` DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '订单总金额',
    `discount_amount` DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '折扣总金额',
    `actual_amount` DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '实际支付金额',
    `delivery_date` DATE NOT NULL COMMENT '期望送货日期',
    `delivery_address` VARCHAR(255) NOT NULL COMMENT '送货地址',
    `contact_name` VARCHAR(32) NOT NULL COMMENT '联系人姓名',
    `contact_phone` VARCHAR(16) NOT NULL COMMENT '联系人电话',
    `remark` VARCHAR(255) DEFAULT NULL COMMENT '订单备注',
    `created_by` VARCHAR(32) NOT NULL COMMENT '下单人',
    `confirmed_by` VARCHAR(32) DEFAULT NULL COMMENT '确认人',
    `confirmed_at` TIMESTAMP NULL DEFAULT NULL COMMENT '确认时间',
    `stocked_by` VARCHAR(32) DEFAULT NULL COMMENT '备货人',
    `stocked_at` TIMESTAMP NULL DEFAULT NULL COMMENT '备货时间',
    `processed_by` VARCHAR(32) DEFAULT NULL COMMENT '处理人',
    `processed_at` TIMESTAMP NULL DEFAULT NULL COMMENT '处理时间',
    `cancelled_by` VARCHAR(32) DEFAULT NULL COMMENT '取消人',
    `cancelled_at` TIMESTAMP NULL DEFAULT NULL COMMENT '取消时间',
    `cancel_reason` VARCHAR(255) DEFAULT NULL COMMENT '取消原因',
    `rejected_by` VARCHAR(32) DEFAULT NULL COMMENT '拒绝人',
    `rejected_at` TIMESTAMP NULL DEFAULT NULL COMMENT '拒绝时间',
    `reject_reason` VARCHAR(255) DEFAULT NULL COMMENT '拒绝原因',
    `completed_by` VARCHAR(32) DEFAULT NULL COMMENT '完成人',
    `completed_at` TIMESTAMP NULL DEFAULT NULL COMMENT '完成时间',
    `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    `updated_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    `deleted_at` TIMESTAMP NULL DEFAULT NULL COMMENT '删除时间',

    PRIMARY KEY (`id`),
    UNIQUE KEY `uk_order_code` (`order_code`),
    KEY `idx_customer_status` (`customer_id`, `order_status`),
    KEY `idx_market_status` (`market_id`, `order_status`),
    KEY `idx_delivery_date` (`delivery_date`),
    KEY `idx_created_at` (`created_at`),
    KEY `idx_deleted_at` (`deleted_at`),
    
    CONSTRAINT `fk_orders_customer` FOREIGN KEY (`customer_id`) REFERENCES `tenants` (`id`),
    CONSTRAINT `fk_orders_market` FOREIGN KEY (`market_id`) REFERENCES `tenants` (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='订单主表';

-- 订单明细表
CREATE TABLE `order_details` (
    `id` INT NOT NULL AUTO_INCREMENT COMMENT '主键ID',
    `order_id` INT NOT NULL COMMENT '订单ID',
    `product_code` VARCHAR(10) NOT NULL COMMENT '产品编号',
    `product_name` VARCHAR(100) NOT NULL COMMENT '产品名称（下单时）',
    `category_id` INT NOT NULL COMMENT '分类ID',
    `category_name` VARCHAR(50) NOT NULL COMMENT '分类名称（下单时）',
    `unit` VARCHAR(4) NOT NULL COMMENT '单位',
    `quantity` DECIMAL(8,2) NOT NULL COMMENT '下单数量',
    `actual_quantity` DECIMAL(8,2) NOT NULL COMMENT '实际数量，供应商称重后更新',
    `original_price` DECIMAL(10,2) NOT NULL COMMENT '原始单价',
    `discount_rate` DECIMAL(4,2) NOT NULL DEFAULT 1 COMMENT '折扣率',
    `actual_price` DECIMAL(10,2) NOT NULL DEFAULT 0 COMMENT '实际单价（折扣后）',
    `total_amount` DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '总金额（数量*实际单价）',
    `actual_amount` DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '实际总金额（实际数量*实际单价）',
    `processing_requirements` TEXT DEFAULT NULL COMMENT '加工要求',
    `remark` VARCHAR(255) DEFAULT NULL COMMENT '备注',
    `status` VARCHAR(16)  DEFAULT 'PENDING' COMMENT '签收状态: PENDING-待签收, RECEIVED-已签收, RETURNED-退货, EXCHANGED-换货',
    `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    `updated_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',

    PRIMARY KEY (`id`),
    KEY `idx_order_id` (`order_id`),
    KEY `idx_product_code` (`product_code`),
    
    CONSTRAINT `fk_order_details_order` FOREIGN KEY (`order_id`) REFERENCES `orders` (`id`) ON DELETE CASCADE,
    CONSTRAINT `fk_order_details_category` FOREIGN KEY (`category_id`) REFERENCES `categories` (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='订单明细表';

ALTER TABLE order_details 
ADD COLUMN receipt_quantity DECIMAL(8,2) NULL COMMENT '实际签收数量',
ADD COLUMN receipt_date TIMESTAMP NULL COMMENT '签收时间',
ADD COLUMN receipt_notes VARCHAR(255) NULL COMMENT '签收备注',
ADD COLUMN receipt_evidence VARCHAR(255) NULL COMMENT '签收凭证(图片URL)';

CREATE TABLE return_exchange_records (
    id INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    order_detail_id INT NOT NULL COMMENT '关联的订单明细ID',
    type VARCHAR(16) NOT NULL COMMENT '类型: RETURN-退货, EXCHANGE-换货',
    quantity DECIMAL(10,2) NOT NULL COMMENT '退换数量',
    reason VARCHAR(32) NOT NULL COMMENT '原因: DAMAGED-损坏, WRONG_ITEM-错误商品, QUALITY_ISSUE-质量问题等',
    reason_description TEXT NULL COMMENT '原因详细描述',
    status VARCHAR(16) NOT NULL DEFAULT 'PENDING' COMMENT '状态: PENDING-待处理, APPROVED-已批准, REJECTED-已拒绝, COMPLETED-已完成',
    evidence_images TEXT NULL COMMENT '证据图片(JSON数组存储URL)',
    processed_by VARCHAR(32) NULL COMMENT '处理人',
    processed_at TIMESTAMP NULL COMMENT '处理时间',
    created_by VARCHAR(32) NOT NULL COMMENT '创建人',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    
    CONSTRAINT fk_return_exchange_order_detail FOREIGN KEY (order_detail_id) 
    REFERENCES order_details(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='退换货记录表';

CREATE TABLE exchange_items (
    id INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    return_exchange_id INT NOT NULL COMMENT '关联的退换货记录ID',
    product_code VARCHAR(10) NOT NULL COMMENT '换货产品编号',
    product_name VARCHAR(100) NOT NULL COMMENT '换货产品名称',
    quantity DECIMAL(10,2) NOT NULL COMMENT '换货数量',
    price DECIMAL(10,2) NOT NULL COMMENT '换货产品单价',
    total_amount DECIMAL(12,2) NOT NULL COMMENT '换货总金额',
    status VARCHAR(16) NOT NULL DEFAULT 'PENDING'
    shipped_at TIMESTAMP NULL COMMENT '发货时间',
    received_at TIMESTAMP NULL COMMENT '收货时间',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    
    CONSTRAINT fk_exchange_items_return_exchange FOREIGN KEY (return_exchange_id) 
    REFERENCES return_exchange_records(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='换货商品表';

CREATE TABLE refund_records (
    id INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    return_exchange_id INT NOT NULL COMMENT '关联的退换货记录ID',
    amount DECIMAL(12,2) NOT NULL COMMENT '退款金额',
    method VARCHAR(16) NOT NULL COMMENT '退款方式: ORIGINAL-原路退回, BALANCE-退回余额等',
    transaction_id VARCHAR(64) NULL COMMENT '交易ID',
    status VARCHAR(16) NOT NULL DEFAULT 'PENDING' COMMENT '状态: PENDING-处理中, COMPLETED-已完成, FAILED-失败',
    completed_at TIMESTAMP NULL COMMENT '完成时间',
    created_by VARCHAR(32) NOT NULL COMMENT '创建人',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    
    CONSTRAINT fk_refund_return_exchange FOREIGN KEY (return_exchange_id) 
    REFERENCES return_exchange_records(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='退款记录表';

-- 订单状态变更历史表
CREATE TABLE `order_status_history` (
    `id` INT NOT NULL AUTO_INCREMENT COMMENT '主键ID',
    `order_id` INT NOT NULL COMMENT '订单ID',
    `from_status` VARCHAR(16) NOT NULL COMMENT '原状态',
    `to_status` VARCHAR(16) NOT NULL COMMENT '新状态',
    `changed_by` VARCHAR(32) NOT NULL COMMENT '操作人',
    `change_reason` VARCHAR(255) DEFAULT NULL COMMENT '变更原因',
    `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    
    PRIMARY KEY (`id`),
    KEY `idx_order_id` (`order_id`),
    CONSTRAINT `fk_order_history_order` FOREIGN KEY (`order_id`) REFERENCES `orders` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='订单状态变更历史表';

CREATE TABLE provider_orders_assignments (
	`id` INT UNSIGNED auto_increment NOT NULL,
	`provider_id` INT NOT NULL,
	`order_id` INT NOT NULL,
	`created_at` DATETIME DEFAULT CURRENT_TIMESTAMP NULL,
	CONSTRAINT `provider_orders_assignments_pk` PRIMARY KEY (id)
)
ENGINE=InnoDB
DEFAULT CHARSET=utf8mb4
COLLATE=utf8mb4_unicode_ci;

ALTER TABLE order_status_history ADD INDEX idx_order_status_time (order_id, to_status, created_at);

ALTER TABLE provider_orders_assignments ADD INDEX idx_provider_order (provider_id, order_id);

   -- 为orders表添加状态索引
   ALTER TABLE orders 
   ADD INDEX idx_status_deleted (order_status, deleted_at);
   
   -- 为tenants表添加name_hash索引（如果还没有）
   ALTER TABLE tenants 
   ADD INDEX idx_name_hash_type (name_hash, tenant_type, deleted_at);

-- 配送人员表
CREATE TABLE delivery_staff (
    id INT NOT NULL AUTO_INCREMENT COMMENT '主键ID',
    provider_id INT NOT NULL COMMENT '供应商ID',
    name VARCHAR(32) NOT NULL COMMENT '配送员姓名',
    phone VARCHAR(16) NOT NULL COMMENT '配送员电话',
    id_card VARCHAR(18) NOT NULL COMMENT '身份证号',
    status BOOLEAN NOT NULL DEFAULT true COMMENT '状态：true-启用, false-禁用',
    remark VARCHAR(255) DEFAULT NULL COMMENT '备注',
    created_by VARCHAR(32) NOT NULL COMMENT '创建人',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    
    PRIMARY KEY (id),
    UNIQUE KEY uk_id_card (id_card),
    KEY idx_provider_id (provider_id),
    KEY idx_status (status),
    
    CONSTRAINT fk_delivery_staff_provider FOREIGN KEY (provider_id) REFERENCES tenants (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='配送人员表';

-- 订单表添加配送员字段
ALTER TABLE orders 
ADD COLUMN delivery_staff_id INT NULL COMMENT '配送员ID' AFTER processed_by,
ADD CONSTRAINT fk_orders_delivery_staff FOREIGN KEY (delivery_staff_id) REFERENCES delivery_staff (id);


-- 对账单主表
CREATE TABLE reconciliation_statements (
    id INT NOT NULL AUTO_INCREMENT COMMENT '主键ID',
    statement_code VARCHAR(32) NOT NULL COMMENT '对账单编号：RS-时间戳-4位随机数',
    customer_id INT NOT NULL COMMENT '客户租户ID',
    market_id INT NOT NULL COMMENT '市场租户ID',
    provider_id INT NOT NULL COMMENT '供应商租户ID',
    start_date DATE NOT NULL COMMENT '对账开始日期',
    end_date DATE NOT NULL COMMENT '对账结束日期',
    total_amount DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '总金额',
    discount_amount DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '折扣总金额',
    actual_amount DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '实际金额',
    status VARCHAR(16) NOT NULL DEFAULT 'PENDING' COMMENT '状态：PENDING-待确认, CONFIRMED-已确认, REJECTED-已拒绝, COMPLETED-已完成',
    remark VARCHAR(255) DEFAULT NULL COMMENT '备注',
    created_by VARCHAR(32) NOT NULL COMMENT '创建人',
    confirmed_by VARCHAR(32) DEFAULT NULL COMMENT '确认人',
    confirmed_at TIMESTAMP NULL DEFAULT NULL COMMENT '确认时间',
    completed_by VARCHAR(32) DEFAULT NULL COMMENT '完成人',
    completed_at TIMESTAMP NULL DEFAULT NULL COMMENT '完成时间',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    deleted_at TIMESTAMP NULL DEFAULT NULL COMMENT '删除时间',
    
    PRIMARY KEY (id),
    UNIQUE KEY uk_statement_code (statement_code),
    KEY idx_customer_status (customer_id, status),
    KEY idx_market_status (market_id, status),
    KEY idx_provider_status (provider_id, status),
    KEY idx_date_range (start_date, end_date),
    KEY idx_created_at (created_at),
    
    CONSTRAINT fk_statements_customer FOREIGN KEY (customer_id) REFERENCES tenants (id),
    CONSTRAINT fk_statements_market FOREIGN KEY (market_id) REFERENCES tenants (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='对账单主表';

-- 对账单订单关联表
CREATE TABLE reconciliation_statement_orders (
    id INT NOT NULL AUTO_INCREMENT COMMENT '主键ID',
    statement_id INT NOT NULL COMMENT '对账单ID',
    order_id INT NOT NULL COMMENT '订单ID',
    order_code VARCHAR(32) NOT NULL COMMENT '订单编号',
    order_date DATE NOT NULL COMMENT '订单日期',
    total_amount DECIMAL(12,2) NOT NULL COMMENT '订单总金额',
    actual_amount DECIMAL(12,2) NOT NULL COMMENT '实际金额（含退换货调整）',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    
    PRIMARY KEY (id),
    KEY idx_statement_id (statement_id),
    KEY idx_order_id (order_id),
    CONSTRAINT fk_statement_orders_statement FOREIGN KEY (statement_id) REFERENCES reconciliation_statements (id) ON DELETE CASCADE,
    CONSTRAINT fk_statement_orders_order FOREIGN KEY (order_id) REFERENCES orders (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='对账单订单关联表';

-- 对账单明细表
CREATE TABLE reconciliation_statement_details (
    id INT NOT NULL AUTO_INCREMENT COMMENT '主键ID',
    statement_id INT NOT NULL COMMENT '对账单ID',
    order_detail_id INT NOT NULL COMMENT '订单明细ID',
    product_code VARCHAR(10) NOT NULL COMMENT '产品编号',
    product_name VARCHAR(100) NOT NULL COMMENT '产品名称',
    category_name VARCHAR(50) NOT NULL COMMENT '分类名称',
    unit VARCHAR(4) NOT NULL COMMENT '单位',
    original_quantity DECIMAL(8,2) NOT NULL COMMENT '原始数量',
    actual_quantity DECIMAL(8,2) NOT NULL COMMENT '实际数量',
    receipt_quantity DECIMAL(8,2) NOT NULL COMMENT '签收数量',
    returned_quantity DECIMAL(8,2) DEFAULT 0 COMMENT '退货数量',
    price DECIMAL(10,2) NOT NULL COMMENT '单价',
    original_amount DECIMAL(12,2) NOT NULL COMMENT '原始金额',
    actual_amount DECIMAL(12,2) NOT NULL COMMENT '实际金额',
    order_date DATE NOT NULL COMMENT '订单日期',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    
    PRIMARY KEY (id),
    KEY idx_statement_id (statement_id),
    KEY idx_order_detail_id (order_detail_id),
    KEY idx_product_code (product_code),
    
    CONSTRAINT fk_statement_details_statement FOREIGN KEY (statement_id) REFERENCES reconciliation_statements (id) ON DELETE CASCADE,
    CONSTRAINT fk_statement_details_order_detail FOREIGN KEY (order_detail_id) REFERENCES order_details (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='对账单明细表';