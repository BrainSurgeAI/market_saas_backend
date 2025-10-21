-- Add migration script here
-- 初始化权限数据
INSERT IGNORE INTO permissions (name) VALUES
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
('audit:export')        -- 导出审计日志
ON DUPLICATE KEY UPDATE name = VALUES(name);

-- 初始化角色数据
INSERT IGNORE INTO roles (name, tenant_type, alias_name) VALUES
('SYSTEM_ADMIN', null, '系统管理员'),            -- 系统管理员，拥有所有权限
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
('SYSTEM_OPERATOR', null, '系统操作员')          -- 系统操作员，负责日常系统运维工作
ON DUPLICATE KEY UPDATE 
    tenant_type = VALUES(tenant_type),
    alias_name = VALUES(alias_name);

-- 为系统管理员分配所有权限
-- SYSTEM_ADMIN (超级管理员，拥有所有权限)
INSERT IGNORE INTO role_permissions (role_id, permission_id)
SELECT 
    (SELECT id FROM roles WHERE name = 'SYSTEM_ADMIN'),
    id
FROM permissions
ON DUPLICATE KEY UPDATE role_id = VALUES(role_id), permission_id = VALUES(permission_id);

-- MARKET_ADMIN (市场管理员)
INSERT IGNORE INTO role_permissions (role_id, permission_id)
SELECT 
    (SELECT id FROM roles WHERE name = 'MARKET_ADMIN'),
    id
FROM permissions 
WHERE name IN (
    'tenant:create', 'tenant:read', 'tenant:update',
    'user:create', 'user:read', 'user:update', 'user:list', 'user:role', 'user:disable',
    'report:view', 'report:export', 'message:read', 'message:update',
    'audit:view'
)
ON DUPLICATE KEY UPDATE role_id = VALUES(role_id), permission_id = VALUES(permission_id);

-- PROVIDER_ADMIN (供应商管理员)
INSERT IGNORE INTO role_permissions (role_id, permission_id)
SELECT 
    (SELECT id FROM roles WHERE name = 'PROVIDER_ADMIN'),
    id
FROM permissions 
WHERE name IN (
    'tenant:create', 'tenant:read', 'tenant:update',
    'user:create', 'user:read', 'user:update', 'user:list', 'user:role', 'user:disable',
    'report:view', 'report:export', 'message:read', 'message:update',
    'audit:view'
)
ON DUPLICATE KEY UPDATE role_id = VALUES(role_id), permission_id = VALUES(permission_id);

-- CUSTOMER_ADMIN (客户管理员)
INSERT IGNORE INTO role_permissions (role_id, permission_id)
SELECT 
    (SELECT id FROM roles WHERE name = 'CUSTOMER_ADMIN'),
    id
FROM permissions 
WHERE name IN (
    'tenant:create', 'tenant:read', 'tenant:update',
    'user:create', 'user:read', 'user:update', 'user:list', 'user:role', 'user:disable',
    'report:view', 'report:export', 'message:read', 'message:update',
    'audit:view'
)
ON DUPLICATE KEY UPDATE role_id = VALUES(role_id), permission_id = VALUES(permission_id);

-- PRICER (定价员)
INSERT IGNORE INTO role_permissions (role_id, permission_id)
SELECT 
    (SELECT id FROM roles WHERE name = 'PRICER'),
    id
FROM permissions 
WHERE name IN (
    'price:create', 'price:read', 'price:update', 'price:list', 'price:history',
    'message:read', 'message:update', 'tenant:read', 'user:read'
)
ON DUPLICATE KEY UPDATE role_id = VALUES(role_id), permission_id = VALUES(permission_id);

-- AUDITOR (审计员)
INSERT IGNORE INTO role_permissions (role_id, permission_id)
SELECT 
    (SELECT id FROM roles WHERE name = 'AUDITOR'),
    id
FROM permissions 
WHERE name IN (
    'price:read', 'price:list', 'price:history', 'price:publish', 'price:approve', 'tenant:read', 'user:read'
)
ON DUPLICATE KEY UPDATE role_id = VALUES(role_id), permission_id = VALUES(permission_id);

-- ORDER_CREATOR (采购员)
INSERT IGNORE INTO role_permissions (role_id, permission_id)
SELECT 
    (SELECT id FROM roles WHERE name = 'ORDER_CREATOR'),
    id
FROM permissions 
WHERE name IN (
    'order:create', 'order:read', 'order:update', 'order:list', 'order:track', 'order:cancel', 'tenant:read', 'user:read',
    'price:read', 'price:list'
)
ON DUPLICATE KEY UPDATE role_id = VALUES(role_id), permission_id = VALUES(permission_id);

-- ORDER_PUBLISHER (订单发布者-市场根据订单进行拆分成子订单发布)
INSERT IGNORE INTO role_permissions (role_id, permission_id)
SELECT 
    (SELECT id FROM roles WHERE name = 'ORDER_PUBLISHER'),
    id
FROM permissions 
WHERE name IN (
    'order:read', 'order:publish', 'order:list', 'order:track', 'order:reject',
    'price:read', 'price:list'
)
ON DUPLICATE KEY UPDATE role_id = VALUES(role_id), permission_id = VALUES(permission_id);

-- ORDER_VIEWER (订单查看者)
INSERT IGNORE INTO role_permissions (role_id, permission_id)
SELECT 
    (SELECT id FROM roles WHERE name = 'ORDER_VIEWER'),
    id
FROM permissions 
WHERE name IN (
    'order:read', 'order:list',
    'price:read', 'price:list'
)
ON DUPLICATE KEY UPDATE role_id = VALUES(role_id), permission_id = VALUES(permission_id);

-- FINANCE_ADMIN (财务管理员)
INSERT IGNORE INTO role_permissions (role_id, permission_id)
SELECT 
    (SELECT id FROM roles WHERE name = 'FINANCE_ADMIN'),
    id
FROM permissions 
WHERE name IN (
    'tenant:financial', 'tenant:read', 'tenant:list',
    'order:read', 'order:list', 'order:export',
    'report:view', 'report:export',
    'price:read', 'price:list'
)
ON DUPLICATE KEY UPDATE role_id = VALUES(role_id), permission_id = VALUES(permission_id);

-- REPORT_VIEWER (报表查看者)
INSERT IGNORE INTO role_permissions (role_id, permission_id)
SELECT 
    (SELECT id FROM roles WHERE name = 'REPORT_VIEWER'),
    id
FROM permissions 
WHERE name IN (
    'report:view', 'report:export'
)
ON DUPLICATE KEY UPDATE role_id = VALUES(role_id), permission_id = VALUES(permission_id);

-- CUSTOMER_SERVICE (客服人员)
INSERT IGNORE INTO role_permissions (role_id, permission_id)
SELECT 
    (SELECT id FROM roles WHERE name = 'CUSTOMER_SERVICE'),
    id
FROM permissions 
WHERE name IN (
    'order:read', 'order:list', 'order:track', 'order:comment',
    'message:send', 'message:read',
    'tenant:read', 'tenant:list',
    'price:read', 'price:list'
)
ON DUPLICATE KEY UPDATE role_id = VALUES(role_id), permission_id = VALUES(permission_id);
