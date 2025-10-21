# 数据库迁移指南

本文档提供了为现有项目添加和管理数据库迁移的详细步骤和最佳实践。

## 1. 安装 SQLx CLI

首先，安装 SQLx 命令行工具：

```bash
cargo install sqlx-cli
```

## 2. 配置数据库连接

创建一个 `.env` 文件，设置数据库连接信息：

```
DATABASE_URL=mysql://username:password@localhost:3306/tenant_saas
```

## 3. 初始化迁移系统

在项目根目录下初始化 SQLx 迁移系统：

```bash
sqlx migrate add initial
```

这将创建一个 `migrations` 目录和第一个迁移文件，如 `migrations/20231101000000_initial.sql`。

## 4. 检查和创建数据库

迁移脚本会自动检查数据库是否存在，如果不存在则创建：

```bash
# 检查数据库是否存在
DB_EXISTS=$(mysql -h"$DB_HOST" -P"$DB_PORT" -u"$DB_USER" -p"$DB_PASS" -e "SHOW DATABASES LIKE '$DB_NAME';" 2>/dev/null | grep -o "$DB_NAME")

# 如果不存在则创建
if [ -z "$DB_EXISTS" ]; then
    mysql -h"$DB_HOST" -P"$DB_PORT" -u"$DB_USER" -p"$DB_PASS" -e "CREATE DATABASE IF NOT EXISTS $DB_NAME CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;" 2>/dev/null
fi
```

在初始迁移文件中，也可以添加创建数据库的语句：

```sql
-- 创建数据库（如果不存在）
CREATE DATABASE IF NOT EXISTS tenant_saas CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

-- 使用数据库
USE tenant_saas;
```

## 5. 捕获现有数据库结构

有两种方法可以捕获现有数据库结构：

### 方法 1：使用 mysqldump 导出结构（推荐）

```bash
# 导出数据库结构（不包含数据）
mysqldump -u username -p --no-data tenant_saas > migrations/20231101000000_initial.sql

# 编辑文件，添加 SQLx 迁移注释
echo "-- Add migration script here" > migrations/20231101000000_initial.sql.temp
cat migrations/20231101000000_initial.sql >> migrations/20231101000000_initial.sql.temp
mv migrations/20231101000000_initial.sql.temp migrations/20231101000000_initial.sql
```

### 方法 2：手动创建迁移文件

如果您的数据库结构较简单，或者想要更精确地控制迁移内容，可以手动编辑迁移文件：

```sql
-- migrations/20231101000000_initial.sql
-- Add migration script here

-- 用户表
CREATE TABLE IF NOT EXISTS users (
    id INT NOT NULL AUTO_INCREMENT,
    username VARCHAR(50) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    name VARCHAR(100) NOT NULL,
    is_super_admin BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    UNIQUE KEY (username)
);

-- 其他表...
```

## 6. 标记迁移为已应用

由于数据库已经存在，需要告诉 SQLx 初始迁移已经应用：

```bash
# 创建迁移表（如果不存在）
sqlx migrate info

# 标记初始迁移为已应用
sqlx migrate run
```

## 7. 为后续更改创建迁移文件

每当需要更改数据库结构时，创建新的迁移文件：

```bash
sqlx migrate add add_email_to_users
```

然后编辑新创建的迁移文件，添加所需的 SQL 语句：

```sql
-- migrations/20231102000000_add_email_to_users.sql
-- Add migration script here
ALTER TABLE users ADD COLUMN email VARCHAR(255) AFTER username;
```

## 8. 创建回滚迁移（可选但推荐）

为每个迁移创建一个对应的回滚文件，以便在需要时撤销更改：

```bash
# 创建回滚目录
mkdir -p migrations/revert

# 创建回滚文件
echo "-- Revert migration script
ALTER TABLE users DROP COLUMN email;" > migrations/revert/20231102000000_add_email_to_users.sql
```

## 9. 集成到项目中

### 更新 Cargo.toml

确保项目依赖包含 SQLx 的迁移功能：

```toml
[dependencies]
sqlx = { version = "0.6", features = ["runtime-tokio-rustls", "mysql", "migrate"] }
```

### 在应用启动时运行迁移

在应用启动时自动应用待处理的迁移：

```rust
use sqlx::mysql::MySqlPoolOptions;
use sqlx::migrate::Migrator;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载环境变量
    dotenv::dotenv().ok();
    
    // 获取数据库连接字符串
    let database_url = std::env::var("DATABASE_URL")?;
    
    // 创建数据库连接池
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    
    // 运行迁移
    let migrator = Migrator::new(Path::new("./migrations")).await?;
    migrator.run(&pool).await?;
    
    // 启动应用...
    
    Ok(())
}
```

## 10. 创建迁移管理脚本

创建一个脚本简化迁移管理：

```bash
#!/bin/bash
# migrate.sh

# 颜色输出
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# 显示帮助
show_help() {
    echo "用法: $0 [命令] [参数]"
    echo "命令:"
    echo "  create [name]    创建新迁移"
    echo "  run              应用所有待处理的迁移"
    echo "  revert           回滚最近的迁移"
    echo "  info             显示迁移状态"
}

# 检查环境变量
if [ -z "$DATABASE_URL" ]; then
    if [ -f .env ]; then
        export $(grep -v '^#' .env | xargs)
    else
        echo -e "${RED}错误: 未设置 DATABASE_URL 环境变量${NC}"
        exit 1
    fi
fi

# 解析命令
case "$1" in
    create)
        if [ -z "$2" ]; then
            echo -e "${RED}错误: 请提供迁移名称${NC}"
            exit 1
        fi
        echo -e "${GREEN}创建新迁移: $2${NC}"
        sqlx migrate add "$2"
        echo -e "${GREEN}创建回滚文件${NC}"
        mkdir -p migrations/revert
        LATEST=$(ls -1 migrations/*.sql | sort | tail -1)
        FILENAME=$(basename "$LATEST")
        echo "-- Revert migration script" > "migrations/revert/$FILENAME"
        echo -e "${GREEN}请编辑迁移文件: $LATEST${NC}"
        echo -e "${GREEN}请编辑回滚文件: migrations/revert/$FILENAME${NC}"
        ;;
    run)
        echo -e "${GREEN}应用待处理的迁移...${NC}"
        sqlx migrate run
        ;;
    revert)
        echo -e "${GREEN}回滚最近的迁移...${NC}"
        LATEST=$(ls -1 migrations/*.sql | sort | tail -1)
        FILENAME=$(basename "$LATEST")
        if [ -f "migrations/revert/$FILENAME" ]; then
            mysql -u$(grep -oP '(?<=mysql:\/\/)[^:]+' <<< "$DATABASE_URL") \
                 -p$(grep -oP '(?<=:)[^@]+' <<< "$DATABASE_URL") \
                 -h$(grep -oP '(?<=@)[^:]+' <<< "$DATABASE_URL") \
                 $(grep -oP '(?<=\/)[^?]+' <<< "$DATABASE_URL") \
                 < "migrations/revert/$FILENAME"
            sqlx migrate revert
            echo -e "${GREEN}迁移已回滚${NC}"
        else
            echo -e "${RED}错误: 回滚文件不存在: migrations/revert/$FILENAME${NC}"
            exit 1
        fi
        ;;
    info)
        echo -e "${GREEN}迁移状态:${NC}"
        sqlx migrate info
        ;;
    *)
        show_help
        exit 1
        ;;
esac

exit 0
```

使迁移脚本可执行：

```bash
chmod +x migrate.sh
```

## 11. 迁移最佳实践

1. **原子性**：每个迁移应该是原子的，要么完全应用，要么完全不应用。

2. **幂等性**：迁移应该是幂等的，可以多次运行而不产生错误。使用 `IF NOT EXISTS` 和 `IF EXISTS` 子句。

3. **向前兼容**：确保迁移不会破坏现有功能。

4. **测试**：在应用到生产环境前，在测试环境中测试迁移。

5. **备份**：在应用迁移前备份数据库。

6. **文档**：为每个迁移添加注释，说明其目的和影响。

## 12. 数据导出和迁移

为了将现有表中的数据导出并创建迁移文件，我们提供了以下脚本：

### 导出单个表数据

使用 `export_table_data.sh` 脚本导出单个表的数据：

```bash
./scripts/data_export/export_table_data.sh users
```

这将导出 `users` 表的数据，并创建一个名为 `init_users_data` 的迁移文件。

您也可以指定迁移文件的名称：

```bash
./scripts/data_export/export_table_data.sh users init_admin_users
```

### 导出多个表数据

使用 `export_all_data.sh` 脚本导出多个表的数据：

```bash
# 导出指定表的数据
./scripts/data_export/export_all_data.sh -t users,roles,permissions -n init_user_system_data

# 导出所有表的数据
./scripts/data_export/export_all_data.sh -a
```

### 导出数据的迁移文件示例

```sql
-- migrations/20250307000000_init_users_data.sql
-- Add migration script here
-- 初始化 users 表数据

INSERT INTO `users` VALUES (1,'admin','$2a$10$X7SIL/CHr5rE.K4PXTcR8eXRlqFcuNKt1wGYo0XTY0qMGH9TCwTK.','系统管理员','admin@example.com','13800138000',NULL,1,NULL,'ACTIVE','2025-03-06 06:29:07','2025-03-06 06:29:07');
INSERT INTO `users` VALUES (2,'market_admin','$2a$10$X7SIL/CHr5rE.K4PXTcR8eXRlqFcuNKt1wGYo0XTY0qMGH9TCwTK.','市场管理员','market@example.com','13800138001',NULL,0,1,'ACTIVE','2025-03-06 06:29:07','2025-03-06 06:29:07');
```

对应的回滚文件：

```sql
-- migrations/revert/20250307000000_init_users_data.sql
-- Revert migration script
-- 删除初始化的 users 表数据

DELETE FROM users WHERE id IN (
1,
2
);
```

### 批量导出的迁移文件示例

```sql
-- migrations/20250307000001_init_all_data.sql
-- Add migration script here
-- 初始化数据

-- users 表数据
INSERT INTO `users` VALUES (1,'admin','$2a$10$X7SIL/CHr5rE.K4PXTcR8eXRlqFcuNKt1wGYo0XTY0qMGH9TCwTK.','系统管理员','admin@example.com','13800138000',NULL,1,NULL,'ACTIVE','2025-03-06 06:29:07','2025-03-06 06:29:07');
INSERT INTO `users` VALUES (2,'market_admin','$2a$10$X7SIL/CHr5rE.K4PXTcR8eXRlqFcuNKt1wGYo0XTY0qMGH9TCwTK.','市场管理员','market@example.com','13800138001',NULL,0,1,'ACTIVE','2025-03-06 06:29:07','2025-03-06 06:29:07');

-- roles 表数据
INSERT INTO `roles` VALUES (1,'SYSTEM_ADMIN',NULL,'系统管理员','2025-03-06 06:29:07','2025-03-06 06:29:07');
INSERT INTO `roles` VALUES (2,'MARKET_ADMIN','MARKET','市场管理员','2025-03-06 06:29:07','2025-03-06 06:29:07');
```

对应的回滚文件：

```sql
-- migrations/revert/20250307000001_init_all_data.sql
-- Revert migration script
-- 删除初始化的数据

SET FOREIGN_KEY_CHECKS = 0;

-- 删除 users 表数据
DELETE FROM users WHERE id IN (
1,
2
);

-- 删除 roles 表数据
DELETE FROM roles WHERE id IN (
1,
2
);

SET FOREIGN_KEY_CHECKS = 1;
```

## 13. 示例迁移文件

### 初始化数据库结构

```sql
-- migrations/20250306062612_initial.sql
-- 创建用户表
CREATE TABLE IF NOT EXISTS users (
    id INT NOT NULL AUTO_INCREMENT,
    username VARCHAR(50) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255),
    phone VARCHAR(20),
    avatar_url VARCHAR(255),
    is_super_admin BOOLEAN DEFAULT FALSE,
    tenant_id INT,
    status ENUM('ACTIVE', 'INACTIVE', 'LOCKED') DEFAULT 'ACTIVE',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    UNIQUE KEY (username),
    KEY (tenant_id),
    KEY (status)
);

-- 创建租户表
CREATE TABLE IF NOT EXISTS tenants (
    id INT NOT NULL AUTO_INCREMENT,
    name VARCHAR(100) NOT NULL,
    code VARCHAR(50) NOT NULL,
    type ENUM('MARKET', 'PROVIDER', 'CUSTOMER') NOT NULL,
    status ENUM('ACTIVE', 'INACTIVE', 'PENDING', 'REJECTED') DEFAULT 'PENDING',
    contact_name VARCHAR(50),
    contact_phone VARCHAR(20),
    contact_email VARCHAR(100),
    address TEXT,
    logo_url VARCHAR(255),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    UNIQUE KEY (code),
    KEY (type),
    KEY (status)
);

-- 创建角色表
CREATE TABLE IF NOT EXISTS roles (
    id INT NOT NULL AUTO_INCREMENT,
    name VARCHAR(50) NOT NULL,
    tenant_type ENUM('MARKET', 'PROVIDER', 'CUSTOMER'),
    alias_name VARCHAR(50) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    UNIQUE KEY (name)
);

-- 创建权限表
CREATE TABLE IF NOT EXISTS permissions (
    id INT NOT NULL AUTO_INCREMENT,
    name VARCHAR(50) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    UNIQUE KEY (name)
);

-- 创建角色权限关联表
CREATE TABLE IF NOT EXISTS role_permissions (
    role_id INT NOT NULL,
    permission_id INT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (role_id, permission_id),
    CONSTRAINT fk_role_permissions_role FOREIGN KEY (role_id) REFERENCES roles (id) ON DELETE CASCADE,
    CONSTRAINT fk_role_permissions_permission FOREIGN KEY (permission_id) REFERENCES permissions (id) ON DELETE CASCADE
);

-- 创建用户角色关联表
CREATE TABLE IF NOT EXISTS user_roles (
    user_id INT NOT NULL,
    role_id INT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, role_id),
    CONSTRAINT fk_user_roles_user FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    CONSTRAINT fk_user_roles_role FOREIGN KEY (role_id) REFERENCES roles (id) ON DELETE CASCADE
);

-- 创建消息表
CREATE TABLE IF NOT EXISTS messages (
    id INT NOT NULL AUTO_INCREMENT,
    sender_id INT NOT NULL,
    receiver_id INT NOT NULL,
    content TEXT NOT NULL,
    is_read BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    KEY (sender_id),
    KEY (receiver_id),
    KEY (is_read),
    CONSTRAINT fk_messages_sender FOREIGN KEY (sender_id) REFERENCES users (id) ON DELETE CASCADE,
    CONSTRAINT fk_messages_receiver FOREIGN KEY (receiver_id) REFERENCES users (id) ON DELETE CASCADE
);
```

### 初始化权限和角色数据

```sql
-- migrations/20250306062730_init_permissions.sql
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
('audit:export')        -- 导出审计日志
ON DUPLICATE KEY UPDATE name = VALUES(name);

-- 初始化角色数据
INSERT INTO roles (name, tenant_type, alias_name) VALUES
('SYSTEM_ADMIN', null, '系统管理员'),            -- 系统管理员，拥有所有权限
('MARKET_ADMIN', 'MARKET', '市场管理员'),        -- 市场管理员，负责管理市场运营
('PROVIDER_ADMIN', 'PROVIDER', '供应商管理员'),  -- 供应商管理员，管理供应商相关业务
('CUSTOMER_ADMIN', 'CUSTOMER', '客户管理员'),    -- 客户管理员，管理客户相关业务
('PRICER', 'MARKET', '询价员'),                 -- 询价员，负责商品定价管理
('AUDITOR', 'MARKET', '价格审计员'),             -- 价格审计员，负责商品价格公示审计工作
('ORDER_CREATOR', 'CUSTOMER', '采购员'),        -- 采购员，可以创建和管理订单
('ORDER_PUBLISHER', 'MARKET', '订单发布者'),     -- 订单发布者，负责订单的发布和跟踪
('ORDER_VIEWER', null, '订单查看者'),            -- 订单查看者，只能查看订单信息
('FINANCE_ADMIN', null, '财务管理员'),           -- 财务管理员，负责财务相关操作
('REPORT_VIEWER', null, '报表查看者'),           -- 报表查看者，可以查看和导出报表
('CUSTOMER_SERVICE', 'MARKET', '客服人员'),      -- 客服人员，处理客户服务相关工作
('SYSTEM_OPERATOR', null, '系统操作员')          -- 系统操作员，负责日常系统运维工作
ON DUPLICATE KEY UPDATE 
    tenant_type = VALUES(tenant_type),
    alias_name = VALUES(alias_name);

-- 为系统管理员分配所有权限
INSERT INTO role_permissions (role_id, permission_id)
SELECT 
    (SELECT id FROM roles WHERE name = 'SYSTEM_ADMIN'),
    id
FROM permissions
ON DUPLICATE KEY UPDATE role_id = VALUES(role_id), permission_id = VALUES(permission_id);

-- 为其他角色分配相应权限
-- 此处省略其他角色权限分配代码...
```

### 添加新表

```sql
-- migrations/20231103000000_create_notifications.sql
-- Add migration script here
CREATE TABLE IF NOT EXISTS notifications (
    id INT NOT NULL AUTO_INCREMENT,
    user_id INT NOT NULL,
    message TEXT NOT NULL,
    is_read BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    KEY (user_id),
    CONSTRAINT fk_notifications_user FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);
```

### 修改表结构

```sql
-- migrations/20231104000000_add_status_to_products.sql
-- Add migration script here
ALTER TABLE products 
ADD COLUMN status ENUM('ACTIVE', 'INACTIVE', 'DISCONTINUED') NOT NULL DEFAULT 'ACTIVE' AFTER name,
ADD INDEX idx_product_status (status);
```

### 添加数据

```sql
-- migrations/20231105000000_add_default_categories.sql
-- Add migration script here
INSERT INTO categories (name, level, parent_id) VALUES 
('蔬菜', 1, NULL),
('水果', 1, NULL),
('肉类', 1, NULL)
ON DUPLICATE KEY UPDATE name = VALUES(name);
```

## 13. 完整的项目结构

```
project/
├── Cargo.toml
├── .env                              # 数据库连接配置
├── src/
│   ├── main.rs                       # 应用入口点
│   ├── middleware/
│   │   └── auth.rs
│   ├── repositories/
│   │   └── product_traits.rs
│   └── ...
├── migrations/                       # 迁移文件
│   ├── 20231101000000_initial.sql    # 初始数据库结构
│   ├── 20231102000000_add_email_to_users.sql
│   └── revert/                       # 回滚脚本
│       ├── 20231101000000_initial.sql
│       └── 20231102000000_add_email_to_users.sql
├── scripts/
│   ├── database.sql                  # 原始数据库脚本
│   ├── create_user.rs                # 用户创建脚本
│   ├── batch_create_users.rs         # 批量用户创建脚本
│   ├── export_data.rs                # 数据导出脚本
│   ├── import_data.rs                # 数据导入脚本
│   ├── test_prices.sql               # 价格测试数据
│   └── migrate.sh                    # 迁移管理脚本
└── README.md                         # 项目文档
```

## 总结

通过添加迁移系统，您将获得以下好处：

1. **版本控制**：数据库更改被版本控制，可以追踪每个更改。
2. **自动化**：迁移可以自动应用，减少手动操作。
3. **一致性**：确保所有环境（开发、测试、生产）使用相同的数据库结构。
4. **回滚能力**：在出现问题时可以回滚更改。
5. **协作**：团队成员可以共享和应用数据库更改。

这些步骤将帮助您为现有项目添加强大的数据库迁移功能，简化数据库管理并提高开发效率。 