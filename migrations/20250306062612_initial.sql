-- Add migration script here
-- 初始数据库结构

-- 用户表
CREATE TABLE IF NOT EXISTS users (
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
CREATE TABLE IF NOT EXISTS tenants (
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
    KEY idx_deleted_at (deleted_at),
    KEY idx_name_hash_type (name_hash, tenant_type, deleted_at)
);

-- 角色表
CREATE TABLE IF NOT EXISTS roles (
    id INT NOT NULL AUTO_INCREMENT,
    name VARCHAR(32) NOT NULL,
    tenant_type VARCHAR(16),
    alias_name VARCHAR(32),
    PRIMARY KEY (id),
    UNIQUE KEY uk_name (name)
);

-- 权限表
CREATE TABLE IF NOT EXISTS permissions (
    id INT NOT NULL AUTO_INCREMENT,
    name VARCHAR(32) NOT NULL,
    PRIMARY KEY (id),
    UNIQUE KEY uk_name (name)
);

-- 角色权限关联表
CREATE TABLE IF NOT EXISTS role_permissions (
    role_id INT NOT NULL,
    permission_id INT NOT NULL,
    PRIMARY KEY (role_id, permission_id),
    KEY fk_role_permissions_permission (permission_id),
    CONSTRAINT fk_role_permissions_permission FOREIGN KEY (permission_id) REFERENCES permissions (id),
    CONSTRAINT fk_role_permissions_role FOREIGN KEY (role_id) REFERENCES roles (id)
);

-- 用户角色关联表
CREATE TABLE IF NOT EXISTS user_roles (
    user_id INT NOT NULL,
    role_id INT NOT NULL,
    alias_name VARCHAR(32),
    PRIMARY KEY (user_id, role_id),
    KEY fk_user_roles_role (role_id),
    CONSTRAINT fk_user_roles_role FOREIGN KEY (role_id) REFERENCES roles (id),
    CONSTRAINT fk_user_roles_user FOREIGN KEY (user_id) REFERENCES users (id)
);

-- 消息表
CREATE TABLE IF NOT EXISTS messages (
    id INT NOT NULL AUTO_INCREMENT,
    user_id INT NOT NULL,
    content TEXT NOT NULL,
    is_read TINYINT(1) DEFAULT 0,
    created_at TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    KEY idx_user_id (user_id),
    CONSTRAINT fk_messages_user FOREIGN KEY (user_id) REFERENCES users (id)
);

-- 分类表
CREATE TABLE IF NOT EXISTS categories (
    id INT NOT NULL AUTO_INCREMENT,
    name VARCHAR(50) NOT NULL COMMENT '分类名称',
    level TINYINT NOT NULL COMMENT '分类层级：1-一级分类(肉类), 2-二级分类(牛肉类), 3-三级分类(半肥半瘦牛肉馅)',
    parent_id INT DEFAULT NULL COMMENT '父级分类ID',
    sort_order SMALLINT DEFAULT 0 COMMENT '排序号',
    PRIMARY KEY (id),
    KEY idx_parent_id (parent_id),
    KEY idx_level_sort (level, sort_order)
) COMMENT='产品分类表';

-- 产品表
CREATE TABLE IF NOT EXISTS products (
    id INT NOT NULL AUTO_INCREMENT COMMENT '主键ID',
    category_id INT NOT NULL COMMENT '分类ID',
    product_code VARCHAR(10) COMMENT '产品唯一编号',
    name VARCHAR(100) NOT NULL COMMENT '产品名称',
    min_order_quantity DECIMAL(7,2) DEFAULT 1 COMMENT '最小起订量',
    brand VARCHAR(16) DEFAULT '其他' COMMENT '品牌',
    unit VARCHAR(4) DEFAULT NULL COMMENT '单位',
    spec VARCHAR(64) DEFAULT NULL COMMENT '规格',
    pricing_method VARCHAR(32) DEFAULT NULL COMMENT '计价方式',
    product_description VARCHAR(96) DEFAULT NULL COMMENT '产品描述',
    special_notes VARCHAR(255) DEFAULT NULL COMMENT '特殊说明',
    tips VARCHAR(96) DEFAULT NULL COMMENT '小贴士',
    storage_conditions VARCHAR(16) DEFAULT NULL COMMENT '存储条件',
    shelf_life VARCHAR(8) DEFAULT NULL COMMENT '保质期',
    tax_rate DECIMAL(4,2) DEFAULT 0 COMMENT '税率',
    is_disabled BOOLEAN DEFAULT FALSE COMMENT '状态：false-禁用，true-启用',
    sort_order SMALLINT DEFAULT 0 COMMENT '排序号',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    PRIMARY KEY (id),
    UNIQUE KEY idx_product_code (product_code),
    KEY idx_category_status (category_id, is_disabled)
) COMMENT='产品表';

-- 产品价格表
CREATE TABLE IF NOT EXISTS product_prices (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT COMMENT '主键ID',
    product_id INT NOT NULL COMMENT '产品ID',
    price_date DATE DEFAULT (CURRENT_DATE),
    min_price DECIMAL(6,2) NOT NULL COMMENT '最低价',
    min_price_change DECIMAL(6,2) NOT NULL COMMENT '最低价变化',
    max_price DECIMAL(6,2) NOT NULL COMMENT '最高价',
    max_price_change DECIMAL(6,2) NOT NULL COMMENT '最高价变化',
    avg_price DECIMAL(6,2) NOT NULL COMMENT '平均价',
    avg_price_change DECIMAL(6,2) NOT NULL COMMENT '平均价变化',
    market_id INT NOT NULL COMMENT '所属市场ID',
    status VARCHAR(16) NOT NULL DEFAULT 'PENDING' COMMENT '状态：PENDING-待审核, APPROVED-已审核, PUBLISHED-已发布, REJECTED-已拒绝',
    remark VARCHAR(32) DEFAULT NULL COMMENT '备注信息',
    source VARCHAR(32) NOT NULL DEFAULT 'SYSTEM' COMMENT '价格来源：SYSTEM-系统录入, API-接口导入, MARKET-市场上报',
    valid_hours TINYINT DEFAULT 24 COMMENT '价格有效时长(小时)',
    created_by VARCHAR(16) NOT NULL COMMENT '创建人',
    approved_by VARCHAR(16) DEFAULT NULL COMMENT '审核人',
    approved_at TIMESTAMP NULL DEFAULT NULL COMMENT '审核时间',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    PRIMARY KEY (id),
    UNIQUE KEY uk_product_date_market (product_id, price_date, market_id),
    KEY idx_price_date (price_date),
    KEY idx_market_status (market_id, status),
    KEY idx_product_status (product_id, status),
    CONSTRAINT fk_prices_market FOREIGN KEY (market_id) REFERENCES tenants (id),
    CONSTRAINT fk_prices_product FOREIGN KEY (product_id) REFERENCES products (id)
) COMMENT='产品价格表';

-- 产品加工费用表
CREATE TABLE IF NOT EXISTS product_processing_fees (
    id INT NOT NULL AUTO_INCREMENT COMMENT '主键ID',
    processing_type VARCHAR(32) NOT NULL COMMENT '加工类型：SKINNING-去皮, DEBONING-剔骨等',
    fee_type VARCHAR(16) NOT NULL DEFAULT 'FIXED' COMMENT '费用类型：FIXED-固定金额, PERCENTAGE-百分比',
    fee_value DECIMAL(4,2) NOT NULL DEFAULT 0 COMMENT '费用值：固定金额时表示具体金额，百分比时表示百分比值',
    description VARCHAR(255) DEFAULT NULL COMMENT '说明',
    is_checkbox BOOLEAN DEFAULT FALSE COMMENT '是否显示为checkbox',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    PRIMARY KEY (id),
    UNIQUE KEY uk_product_processing (processing_type)
) COMMENT='产品加工费用表';

-- 产品加工费用关联表
CREATE TABLE IF NOT EXISTS product_processing_fee_relations (
    id INT NOT NULL AUTO_INCREMENT COMMENT '主键ID',
    product_id INT NOT NULL COMMENT '产品ID',
    processing_fee_id INT NOT NULL COMMENT '加工费用ID',
    is_default BOOLEAN DEFAULT FALSE COMMENT '是否为默认加工选项',
    sort_order SMALLINT DEFAULT 0 COMMENT '排序号',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    PRIMARY KEY (id),
    UNIQUE KEY uk_product_processing (product_id, processing_fee_id),
    KEY idx_product_id (product_id),
    KEY idx_processing_fee_id (processing_fee_id),
    CONSTRAINT fk_product_processing_product FOREIGN KEY (product_id) REFERENCES products (id) ON DELETE CASCADE,
    CONSTRAINT fk_product_processing_fee FOREIGN KEY (processing_fee_id) REFERENCES product_processing_fees (id) ON DELETE CASCADE
) COMMENT='产品加工费用关联表';

-- 供应商财务信息表
CREATE TABLE IF NOT EXISTS provider_financial_profiles (
    id INT NOT NULL AUTO_INCREMENT,
    tenant_id INT NOT NULL,
    credit_score DECIMAL(10,2) DEFAULT NULL,
    credit_limit DECIMAL(15,2) DEFAULT NULL,
    deposit_amount DECIMAL(15,2) DEFAULT NULL,
    payment_period_days INT DEFAULT NULL,
    created_at TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP NULL DEFAULT NULL,
    PRIMARY KEY (id),
    UNIQUE KEY uk_tenant_id (tenant_id),
    KEY idx_deleted_at (deleted_at),
    CONSTRAINT fk_supplier_financial_tenant FOREIGN KEY (tenant_id) REFERENCES tenants (id)
);

-- 临时图片URL表
CREATE TABLE IF NOT EXISTS temp_image_urls (
    id INT NOT NULL AUTO_INCREMENT,
    object_key VARCHAR(512) NOT NULL,
    product_code VARCHAR(10) DEFAULT NULL,
    temp_url TEXT NOT NULL,
    expires_at DATETIME NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    UNIQUE KEY object_key (object_key),
    UNIQUE KEY product_code (product_code)
);

-- 租户关系表
CREATE TABLE IF NOT EXISTS tenant_relationships (
    id INT NOT NULL AUTO_INCREMENT,
    market_id INT NOT NULL,
    provider_id INT NOT NULL,
    status VARCHAR(16) NOT NULL DEFAULT 'PENDING',
    created_at TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    KEY idx_market_id (market_id),
    KEY idx_provider_id (provider_id),
    CONSTRAINT fk_market_providers_market FOREIGN KEY (market_id) REFERENCES tenants (id),
    CONSTRAINT fk_market_providers_provider FOREIGN KEY (provider_id) REFERENCES tenants (id)
);

CREATE TABLE delivery_staff (
  id INT NOT NULL AUTO_INCREMENT COMMENT '主键ID',
  provider_id INT NOT NULL COMMENT '租户ID，关联tenants表',
  name VARCHAR(32) NOT NULL COMMENT '姓名',
  phone CHAR(11) NOT NULL COMMENT '手机号',
  id_card CHAR(18) NOT NULL COMMENT '身份证号',
  created_by VARCHAR(32) NOT NULL COMMENT '创建人',
  status TINYINT NOT NULL DEFAULT 1 COMMENT '状态：0=禁用，1=启用',
   created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
   updated_at TIMESTAMP  DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  remark VARCHAR(255) NULL COMMENT '备注',
  PRIMARY KEY (id),
  UNIQUE KEY uk_tenant_phone (provider_id, phone),
  KEY idx_provider_id (provider_id),
  CONSTRAINT fk_delivery_staff_provider
    FOREIGN KEY (provider_id)
    REFERENCES tenants (id)
    ON DELETE CASCADE
    ON UPDATE CASCADE
) ENGINE=InnoDB
  DEFAULT CHARSET=utf8mb4
  COLLATE=utf8mb4_unicode_ci
  COMMENT='配送员信息表';


-- 用户分类关联表
CREATE TABLE IF NOT EXISTS user_category_assignments (
    id INT NOT NULL AUTO_INCREMENT,
    user_id INT NOT NULL,
    category_id INT NOT NULL,
    created_at TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    UNIQUE KEY user_id (user_id, category_id),
    KEY category_id (category_id),
    CONSTRAINT user_category_assignments_ibfk_1 FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    CONSTRAINT user_category_assignments_ibfk_2 FOREIGN KEY (category_id) REFERENCES categories (id) ON DELETE CASCADE
);

-- 客户分类折扣表
CREATE TABLE IF NOT EXISTS customer_category_discounts (
    id INT NOT NULL AUTO_INCREMENT COMMENT '主键ID',
    tenant_id INT NOT NULL COMMENT '客户租户ID',
    category_id INT NOT NULL COMMENT '一级分类ID',
    discount_rate DECIMAL(4,2) NOT NULL COMMENT '折扣率：0.8表示8折',
    start_date DATE NOT NULL COMMENT '生效开始日期',
    end_date DATE NULL COMMENT '生效结束日期，NULL表示永久有效',
    status BOOLEAN NOT NULL DEFAULT TRUE COMMENT '状态：true-生效中, false-已失效',
    remark VARCHAR(255) DEFAULT NULL COMMENT '备注说明',
    created_by VARCHAR(32) NOT NULL COMMENT '创建人',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    
    PRIMARY KEY (id),
    UNIQUE KEY uk_tenant_category (tenant_id, category_id, start_date),
    KEY idx_category_status (category_id, status),
    KEY idx_tenant_status (tenant_id, status),
    KEY idx_date_status (start_date, end_date, status),
    
    CONSTRAINT fk_customer_discounts_tenant FOREIGN KEY (tenant_id) REFERENCES tenants (id),
    CONSTRAINT fk_customer_discounts_category FOREIGN KEY (category_id) REFERENCES categories (id),
    CONSTRAINT chk_discount_rate CHECK (discount_rate > 0 AND discount_rate <= 1)
) COMMENT='客户分类折扣表';

-- 折扣历史记录表
CREATE TABLE IF NOT EXISTS customer_category_discount_history (
    id INT NOT NULL AUTO_INCREMENT COMMENT '主键ID',
    tenant_id INT NOT NULL COMMENT '客户租户ID',
    category_id INT NOT NULL COMMENT '一级分类ID',
    old_discount_rate DECIMAL(4,3) COMMENT '原折扣率',
    new_discount_rate DECIMAL(4,3) NOT NULL COMMENT '新折扣率',
    change_type VARCHAR(16) NOT NULL COMMENT '变更类型：CREATE-新建, UPDATE-修改, EXPIRE-失效',
    change_reason VARCHAR(255) DEFAULT NULL COMMENT '变更原因',
    changed_by VARCHAR(32) NOT NULL COMMENT '操作人',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    
    PRIMARY KEY (id),
    KEY idx_tenant_category (tenant_id, category_id),
    KEY idx_created_at (created_at)
) COMMENT='客户分类折扣历史表';

-- 订单主表
CREATE TABLE IF NOT EXISTS orders (
    id INT NOT NULL AUTO_INCREMENT COMMENT '主键ID',
    order_code VARCHAR(32) NOT NULL COMMENT '订单编号：DD-时间戳-4位随机数',
    customer_id INT NOT NULL COMMENT '客户租户ID',
    market_id INT NOT NULL COMMENT '市场租户ID',
    order_status VARCHAR(16) NOT NULL DEFAULT 'PENDING' COMMENT '订单状态：PENDING-待确认, CONFIRMED-已确认, PROCESSING-处理中, STOCKED-备货完毕, COMPLETED-已完成, CANCELLED-已取消, REJECTED-已拒绝',
    total_amount DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '订单总金额',
    discount_amount DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '折扣总金额',
    actual_amount DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '实际支付金额',
    delivery_date DATE NOT NULL COMMENT '期望送货日期',
    delivery_address VARCHAR(255) NOT NULL COMMENT '送货地址',
    contact_name VARCHAR(32) NOT NULL COMMENT '联系人姓名',
    contact_phone VARCHAR(16) NOT NULL COMMENT '联系人电话',
    remark VARCHAR(255) DEFAULT NULL COMMENT '订单备注',
    created_by VARCHAR(32) NOT NULL COMMENT '下单人',
    confirmed_by VARCHAR(32) DEFAULT NULL COMMENT '确认人',
    confirmed_at TIMESTAMP NULL DEFAULT NULL COMMENT '确认时间',
    stocked_by VARCHAR(32) DEFAULT NULL COMMENT '备货人',
    stocked_at TIMESTAMP NULL DEFAULT NULL COMMENT '备货时间',
    processed_by VARCHAR(32) DEFAULT NULL COMMENT '处理人',
    processed_at TIMESTAMP NULL DEFAULT NULL COMMENT '处理时间',
    cancelled_by VARCHAR(32) DEFAULT NULL COMMENT '取消人',
    cancelled_at TIMESTAMP NULL DEFAULT NULL COMMENT '取消时间',
    cancel_reason VARCHAR(255) DEFAULT NULL COMMENT '取消原因',
    after_sale_at TIMESTAMP NULL,
    rejected_by VARCHAR(32) DEFAULT NULL COMMENT '拒绝人',
    rejected_at TIMESTAMP NULL DEFAULT NULL COMMENT '拒绝时间',
    reject_reason VARCHAR(255) DEFAULT NULL COMMENT '拒绝原因',
    completed_by VARCHAR(32) DEFAULT NULL COMMENT '完成人',
    completed_at TIMESTAMP NULL DEFAULT NULL COMMENT '完成时间',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    deleted_at TIMESTAMP NULL DEFAULT NULL COMMENT '删除时间',

    PRIMARY KEY (id),
    UNIQUE KEY uk_order_code (order_code),
    KEY idx_customer_status (customer_id, order_status),
    KEY idx_market_status (market_id, order_status),
    KEY idx_delivery_date (delivery_date),
    KEY idx_created_at (created_at),
    KEY idx_deleted_at (deleted_at),
    KEY idx_status_deleted (order_status, deleted_at),
    
    CONSTRAINT fk_orders_customer FOREIGN KEY (customer_id) REFERENCES tenants (id),
    CONSTRAINT fk_orders_market FOREIGN KEY (market_id) REFERENCES tenants (id)
) COMMENT='订单主表';

-- 订单明细表
CREATE TABLE IF NOT EXISTS order_details (
    id INT NOT NULL AUTO_INCREMENT COMMENT '主键ID',
    order_id INT NOT NULL COMMENT '订单ID',
    product_code VARCHAR(10) NOT NULL COMMENT '产品编号',
    product_name VARCHAR(100) NOT NULL COMMENT '产品名称（下单时）',
    category_id INT NOT NULL COMMENT '分类ID',
    category_name VARCHAR(50) NOT NULL COMMENT '分类名称（下单时）',
    unit VARCHAR(4) NOT NULL COMMENT '单位',
    quantity DECIMAL(8,2) NOT NULL COMMENT '下单数量',
    actual_quantity DECIMAL(8,2) DEFAULT NULL COMMENT '实际数量，供应商称重后更新',
    original_price DECIMAL(10,2) NOT NULL COMMENT '原始单价',
    discount_rate DECIMAL(4,2) NOT NULL DEFAULT 1 COMMENT '折扣率',
    actual_price DECIMAL(10,2) NOT NULL DEFAULT 0 COMMENT '实际单价（折扣后）',
    total_amount DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '总金额（数量*实际单价）',
    actual_amount DECIMAL(12,2) DEFAULT NULL COMMENT '实际总金额（实际数量*实际单价）',
    processing_requirements TEXT DEFAULT NULL COMMENT '加工要求',
    remark VARCHAR(255) DEFAULT NULL COMMENT '备注',
    status VARCHAR(16) DEFAULT 'PENDING' COMMENT '签收状态: PENDING-待签收, RECEIVED-已签收, RETURNED-退货, EXCHANGED-换货',
    receipt_quantity DECIMAL(8,2) NULL COMMENT '实际签收数量',
    receipt_date TIMESTAMP NULL COMMENT '签收时间',
    receipt_notes VARCHAR(255) NULL COMMENT '签收备注',
    receipt_evidence VARCHAR(255) NULL COMMENT '签收凭证(图片URL)',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',

    PRIMARY KEY (id),
    KEY idx_order_id (order_id),
    KEY idx_product_code (product_code),
    
    CONSTRAINT fk_order_details_order FOREIGN KEY (order_id) REFERENCES orders (id) ON DELETE CASCADE,
    CONSTRAINT fk_order_details_category FOREIGN KEY (category_id) REFERENCES categories (id)
) COMMENT='订单明细表';

-- 订单状态变更历史表
CREATE TABLE IF NOT EXISTS order_status_history (
    id INT NOT NULL AUTO_INCREMENT COMMENT '主键ID',
    order_id INT NOT NULL COMMENT '订单ID',
    from_status VARCHAR(16) NOT NULL COMMENT '原状态',
    to_status VARCHAR(16) NOT NULL COMMENT '新状态',
    changed_by VARCHAR(32) NOT NULL COMMENT '操作人',
    change_reason VARCHAR(255) DEFAULT NULL COMMENT '变更原因',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    
    PRIMARY KEY (id),
    KEY idx_order_id (order_id),
    KEY idx_order_status_time (order_id, to_status, created_at),
    CONSTRAINT fk_order_history_order FOREIGN KEY (order_id) REFERENCES orders (id) ON DELETE CASCADE
) COMMENT='订单状态变更历史表';

-- 退换货记录表
CREATE TABLE IF NOT EXISTS return_exchange_records (
    id INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    order_detail_id INT NOT NULL COMMENT '关联的订单明细ID',
    operation_type VARCHAR(16) NOT NULL COMMENT '类型: RETURN-退货, EXCHANGE-换货',
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
) COMMENT='退换货记录表';

-- 换货商品表
CREATE TABLE IF NOT EXISTS exchange_items (
    id INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    return_exchange_id INT NOT NULL COMMENT '关联的退换货记录ID',
    product_code VARCHAR(10) NOT NULL COMMENT '换货产品编号',
    product_name VARCHAR(100) NOT NULL COMMENT '换货产品名称',
    quantity DECIMAL(10,2) NOT NULL COMMENT '换货数量',
    price DECIMAL(10,2) NOT NULL COMMENT '换货产品单价',
    total_amount DECIMAL(12,2) NOT NULL COMMENT '换货总金额',
    status VARCHAR(16) NOT NULL DEFAULT 'PENDING' COMMENT '状态: PENDING-待发货, SHIPPED-已发货, RECEIVED-已收货',
    shipped_at TIMESTAMP NULL COMMENT '发货时间',
    received_at TIMESTAMP NULL COMMENT '收货时间',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    
    CONSTRAINT fk_exchange_items_return_exchange FOREIGN KEY (return_exchange_id) 
    REFERENCES return_exchange_records(id) ON DELETE CASCADE
) COMMENT='换货商品表';

-- 退款记录表
CREATE TABLE IF NOT EXISTS refund_records (
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
) COMMENT='退款记录表';

-- 供应商订单分配表
CREATE TABLE IF NOT EXISTS provider_orders_assignments (
    id INT UNSIGNED AUTO_INCREMENT NOT NULL,
    provider_id INT NOT NULL,
    order_id INT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NULL,
    PRIMARY KEY (id),
    KEY idx_provider_order (provider_id, order_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


CREATE TABLE IF NOT EXISTS reconciliation_statements (
    id INT AUTO_INCREMENT PRIMARY KEY,

    statement_code VARCHAR(100) NOT NULL COMMENT '对账单编号',

    customer_id INT NOT NULL COMMENT '客户ID',
    market_id INT NOT NULL COMMENT '市场ID',
    provider_id INT NOT NULL COMMENT '供应商ID',

    supplier_name VARCHAR(255) NULL COMMENT '供应商名称',
    customer_name VARCHAR(255) NULL COMMENT '客户名称',

    start_date DATE NOT NULL COMMENT '开始日期',
    end_date DATE NOT NULL COMMENT '结束日期',

    total_amount DECIMAL(18,2) NOT NULL COMMENT '总金额',
    discount_amount DECIMAL(18,2) NOT NULL COMMENT '折扣金额',
    actual_amount DECIMAL(18,2) NOT NULL COMMENT '实际金额',

    status VARCHAR(50) NOT NULL COMMENT '状态',
    remark TEXT NULL COMMENT '备注',

    created_by VARCHAR(100) NOT NULL COMMENT '创建人',
    confirmed_by VARCHAR(100) NULL COMMENT '确认人',
    confirmed_at TIMESTAMP NULL DEFAULT NULL COMMENT '确认时间',
    completed_by VARCHAR(100) NULL COMMENT '完成处理人',
    completed_at TIMESTAMP NULL DEFAULT NULL COMMENT '完成时间',

    created_at TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    deleted_at TIMESTAMP NULL DEFAULT NULL COMMENT '删除时间',

    CONSTRAINT fk_customer FOREIGN KEY (customer_id)
        REFERENCES tenants(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,

    CONSTRAINT fk_market FOREIGN KEY (market_id)
        REFERENCES tenants(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,

    CONSTRAINT fk_provider FOREIGN KEY (provider_id)
        REFERENCES tenants(id)
        ON UPDATE CASCADE ON DELETE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='对账单表';

CREATE TABLE reconciliation_statement_orders (
    id INT AUTO_INCREMENT PRIMARY KEY,

    statement_id INT NOT NULL COMMENT '对账单ID',
    order_id INT NOT NULL COMMENT '订单ID',
    order_code VARCHAR(100) NOT NULL COMMENT '订单编号',
    order_date DATE NOT NULL COMMENT '订单日期',

    total_amount DECIMAL(18,2) NOT NULL COMMENT '订单总金额',
    actual_amount DECIMAL(18,2) NOT NULL COMMENT '实际金额',

    created_at TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',

    KEY idx_statement_id (statement_id),
    KEY idx_order_id (order_id),

    CONSTRAINT fk_statement_order FOREIGN KEY (statement_id)
        REFERENCES reconciliation_statements(id)
        ON UPDATE CASCADE ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='对账单订单表';

CREATE TABLE reconciliation_statement_details (
    id INT AUTO_INCREMENT PRIMARY KEY,
    statement_id INT NOT NULL COMMENT '对账单ID',
    order_detail_id INT NOT NULL COMMENT '订单明细ID',
    product_code VARCHAR(100) NOT NULL COMMENT '产品编号',
    product_name VARCHAR(255) NOT NULL COMMENT '产品名称',
    category_name VARCHAR(255) NOT NULL COMMENT '分类名称',
    unit VARCHAR(50) NOT NULL COMMENT '单位',
    original_quantity DECIMAL(18,2) NOT NULL COMMENT '原始数量',
    actual_quantity DECIMAL(18,2) NOT NULL COMMENT '实际数量',
    receipt_quantity DECIMAL(18,2) NOT NULL COMMENT '收货数量',
    returned_quantity DECIMAL(18,2) NOT NULL COMMENT '退货数量',
    price DECIMAL(18,2) NOT NULL COMMENT '单价',
    original_amount DECIMAL(18,2) NOT NULL COMMENT '原始金额',
    actual_amount DECIMAL(18,2) NOT NULL COMMENT '实际金额',
    order_date DATE NOT NULL COMMENT '订单日期',
    created_at TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    KEY idx_statement_id (statement_id),
    KEY idx_order_detail_id (order_detail_id),
    CONSTRAINT fk_statement_detail FOREIGN KEY (statement_id) REFERENCES reconciliation_statements(id) ON UPDATE CASCADE ON DELETE CASCADE,
    CONSTRAINT fk_order_detail FOREIGN KEY (order_detail_id) REFERENCES order_details(id) ON UPDATE CASCADE ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='对账单明细表';

ALTER TABLE orders 
ADD COLUMN delivery_staff_id INT NULL COMMENT '配送员ID';

ALTER TABLE orders 
ADD CONSTRAINT fk_orders_delivery_staff 
FOREIGN KEY (delivery_staff_id) REFERENCES delivery_staff(id) 
ON DELETE SET NULL 
ON UPDATE CASCADE;

ALTER TABLE orders 
ADD INDEX idx_orders_delivery_staff (delivery_staff_id);

