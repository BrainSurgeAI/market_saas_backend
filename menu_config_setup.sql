-- 菜单配置表和角色菜单关联表创建及数据插入脚本

-- 创建菜单配置表
CREATE TABLE IF NOT EXISTS menu_config (
    id INT NOT NULL AUTO_INCREMENT,
    title VARCHAR(100) NOT NULL COMMENT '菜单标题',
    url VARCHAR(255) DEFAULT NULL COMMENT '菜单URL',
    icon VARCHAR(100) DEFAULT NULL COMMENT '菜单图标',
    is_active BOOLEAN DEFAULT FALSE COMMENT '是否激活',
    parent_id INT DEFAULT NULL COMMENT '父级菜单ID',
    sort_order SMALLINT DEFAULT 0 COMMENT '排序号',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    KEY idx_parent_id (parent_id),
    KEY idx_sort_order (sort_order),
    CONSTRAINT fk_menu_parent FOREIGN KEY (parent_id) REFERENCES menu_config(id) ON DELETE CASCADE
) COMMENT='菜单配置表';

-- 创建角色菜单关联表
CREATE TABLE IF NOT EXISTS role_menu (
    id INT NOT NULL AUTO_INCREMENT,
    role_name VARCHAR(32) NOT NULL COMMENT '角色名称',
    menu_id INT NOT NULL COMMENT '菜单ID',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    UNIQUE KEY uk_role_menu (role_name, menu_id),
    KEY idx_role_name (role_name),
    KEY idx_menu_id (menu_id),
    CONSTRAINT fk_role_menu_menu FOREIGN KEY (menu_id) REFERENCES menu_config(id) ON DELETE CASCADE
) COMMENT='角色菜单关联表';

-- 插入菜单数据并关联角色

-- 首先插入所有顶级菜单和子菜单
INSERT INTO menu_config (title, url, icon, is_active, parent_id, sort_order) VALUES
-- 系统管理
('系统管理', '/workspace', 'SquareTerminal', TRUE, NULL, 1),
('组织管理', '/workspace/organizations', NULL, FALSE, 1, 1),
('用户管理', '/workspace/users', NULL, FALSE, 1, 2),
('系统设置', '/workspace/settings', NULL, FALSE, 1, 3),
('日志审计', '/workspace/logs', NULL, FALSE, 1, 4),

-- 数据中心
('数据中心', '', 'Database', FALSE, NULL, 2),
('数据概览', '/workspace/data/overview', NULL, FALSE, 6, 1),
('数据分析', '/workspace/data/analysis', NULL, FALSE, 6, 2),

-- 服务监控
('服务监控', '', 'Server', FALSE, NULL, 3),
('服务状态', '/workspace/services/status', NULL, FALSE, 9, 1),
('性能监控', '/workspace/services/performance', NULL, FALSE, 9, 2),

-- 组织管理 (market_admin)
('组织管理', '/workspace', 'Building', TRUE, NULL, 4),
('租户管理', '/workspace/markets/:market_id/tenants', NULL, FALSE, 12, 1),
('成员管理', '/workspace/organization/members', NULL, FALSE, 12, 2),
('部门设置', '/workspace/organization/departments', NULL, FALSE, 12, 3),

-- 数据管理
('数据管理', '', 'Frame', FALSE, NULL, 5),
('结算单', '/workspace/invoiceSettlements', NULL, FALSE, 16, 1),

-- 工作台 (pricer, auditor, order_creator, order_publisher, provider_admin, provider)
('工作台', '/workspace', 'SquareTerminal', TRUE, NULL, 6),
('报价管理', '/workspace/organizations/:org_name/product_prices', NULL, FALSE, 18, 1),
('商品管理', '/workspace/organizations/:org_name/products', NULL, FALSE, 18, 2),

-- 订单管理 (order_creator)
('订单管理', '/workspace/customers/:customer_id/orders', NULL, FALSE, 18, 3),
('去采购', '/workspace', NULL, FALSE, 18, 4),
('结算单', '/workspace/customers/:customer_id/invoiceSettlements', NULL, FALSE, 18, 5),

-- 订单管理 (order_publisher)
('订单管理', '/workspace/markets/:market_id/orders', NULL, FALSE, 18, 6),
('结算单', '/workspace/markets/:market_id/invoiceSettlements', NULL, FALSE, 18, 7),

-- 订单管理 (provider_admin)
('订单管理', '/workspace/providers/:provider_id/orders', NULL, FALSE, 18, 8),
('配送员管理', '/workspace/providers/:provider_id/delivery_staffs', NULL, FALSE, 18, 9),

-- 结算单 (provider)
('结算单', '/workspace/providers/:provider_id/invoiceSettlements', NULL, FALSE, 18, 10);

-- 获取插入的菜单ID，为角色分配菜单
-- super_admin 角色菜单关联 (菜单ID: 1-11)
INSERT INTO role_menu (role_id, menu_id) VALUES
(1, 1),  -- 系统管理
(1, 2),  -- 组织管理
(1, 3),  -- 用户管理
(1, 4),  -- 系统设置
(1, 5),  -- 日志审计
(1, 6),  -- 数据中心
(1, 7),  -- 数据概览
(1, 8),  -- 数据分析
(1, 9),  -- 服务监控
(1, 10), -- 服务状态
(1, 11); -- 性能监控

-- market_admin 角色菜单关联 (菜单ID: 12-17)
INSERT INTO role_menu (role_id, menu_id) VALUES
(2, 12), -- 组织管理
(2, 13), -- 租户管理
(2, 14), -- 成员管理
(2, 15), -- 部门设置
(2, 16), -- 数据管理
(2, 17); -- 结算单

-- pricer 角色菜单关联 (菜单ID: 18-19)
INSERT INTO role_menu (role_id, menu_id) VALUES
(5, 18), -- 工作台
(5, 19); -- 报价管理

-- auditor 角色菜单关联 (菜单ID: 18-20)
INSERT INTO role_menu (role_id, menu_id) VALUES
(6, 18), -- 工作台
(6, 19), -- 报价管理
(6, 20); -- 商品管理

-- order_creator 角色菜单关联 (菜单ID: 18,21-23)
INSERT INTO role_menu (role_id, menu_id) VALUES
(7, 18), -- 工作台
(7, 21), -- 订单管理 (客户)
(7, 22), -- 去采购
(7, 23); -- 结算单 (客户)

-- order_publisher 角色菜单关联 (菜单ID: 18,24-25)
INSERT INTO role_menu (role_id, menu_id) VALUES
(8, 18), -- 工作台
(8, 24), -- 订单管理 (市场)
(8, 25); -- 结算单 (市场)

-- provider_admin 角色菜单关联 (菜单ID: 18,26-27)
INSERT INTO role_menu (role_id, menu_id) VALUES
(3, 18), -- 工作台
(3, 26), -- 订单管理 (供应商)
(3, 27); -- 配送员管理

-- provider 角色菜单关联 (菜单ID: 18,19,28)
-- INSERT INTO role_menu (role_id, menu_id) VALUES
-- ('provider', 18), -- 工作台
-- ('provider', 19), -- 订单管理 (供应商)
-- ('provider', 28); -- 结算单 (供应商)