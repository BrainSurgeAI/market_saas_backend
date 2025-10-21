#!/bin/bash

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 打印彩色信息
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_step() {
    echo -e "${BLUE}🔧 $1${NC}"
}

# 容器化MySQL配置
DEFAULT_DB_HOST="localhost"
DEFAULT_DB_PORT="3307"
DEFAULT_DB_USER="root"
DEFAULT_DB_PASSWORD="Nihaoccj123"
DEFAULT_DB_NAME="test_db"

# 从环境变量获取配置，如果不存在则使用默认值
DB_HOST=${TEST_DB_HOST:-$DEFAULT_DB_HOST}
DB_PORT=${TEST_DB_PORT:-$DEFAULT_DB_PORT}
DB_USER=${TEST_DB_USER:-$DEFAULT_DB_USER}
DB_PASSWORD=${TEST_DB_PASSWORD:-$DEFAULT_DB_PASSWORD}
DB_NAME=${TEST_DB_NAME:-$DEFAULT_DB_NAME}

echo "🚀 测试数据库设置脚本 (容器化MySQL)"
echo "=========================================="

print_info "数据库配置:"
echo "  主机: $DB_HOST"
echo "  端口: $DB_PORT"
echo "  用户: $DB_USER"
echo "  数据库: $DB_NAME"
echo

# 检查MySQL容器连接
print_step "检查MySQL容器连接..."
if mysql --protocol=TCP -h"$DB_HOST" -P"$DB_PORT" -u"$DB_USER" -p"$DB_PASSWORD" -e "SELECT 1" > /dev/null 2>&1; then
    print_success "MySQL容器连接成功"
else
    print_error "MySQL容器连接失败"
    print_info "请确保："
    echo "  1. MySQL容器正在运行"
    echo "  2. 端口3307可访问"
    echo "  3. 用户名和密码正确"
    echo "  4. 可以尝试: docker ps | grep mysql"
    exit 1
fi

# 显示MySQL版本信息
print_step "获取MySQL版本信息..."
MYSQL_VERSION=$(mysql --protocol=TCP -h"$DB_HOST" -P"$DB_PORT" -u"$DB_USER" -p"$DB_PASSWORD" -e "SELECT VERSION();" 2>/dev/null | tail -n 1)
print_success "MySQL版本: $MYSQL_VERSION"

# 创建测试数据库
print_step "创建测试数据库 '$DB_NAME'..."
mysql --protocol=TCP -h"$DB_HOST" -P"$DB_PORT" -u"$DB_USER" -p"$DB_PASSWORD" -e "CREATE DATABASE IF NOT EXISTS \`$DB_NAME\` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;" 2>/dev/null
if [ $? -eq 0 ]; then
    print_success "测试数据库创建成功"
else
    print_error "测试数据库创建失败"
    exit 1
fi

# 设置环境变量
export DATABASE_URL="mysql://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME"
export TEST_DATABASE_URL="$DATABASE_URL"

print_step "运行数据库迁移..."

# 执行初始迁移
if [ -f "migrations/20250306062612_initial.sql" ]; then
    print_info "执行初始迁移..."
    mysql --protocol=TCP -h"$DB_HOST" -P"$DB_PORT" -u"$DB_USER" -p"$DB_PASSWORD" "$DB_NAME" < migrations/20250306062612_initial.sql
    if [ $? -eq 0 ]; then
        print_success "初始迁移执行完成"
    else
        print_error "初始迁移执行失败"
        exit 1
    fi
else
    print_warning "未找到初始迁移文件: migrations/20250306062612_initial.sql"
fi

# 如果有其他SQL脚本，也可以执行
if [ -f "scripts/database.sql" ]; then
    print_info "执行额外的数据库脚本..."
    mysql --protocol=TCP -h"$DB_HOST" -P"$DB_PORT" -u"$DB_USER" -p"$DB_PASSWORD" "$DB_NAME" < scripts/database.sql
    if [ $? -eq 0 ]; then
        print_success "数据库脚本执行完成"
    else
        print_warning "数据库脚本执行失败，但不影响继续"
    fi
fi

# 验证表是否创建成功
print_step "验证数据库表..."
TABLE_COUNT=$(mysql --protocol=TCP -h"$DB_HOST" -P"$DB_PORT" -u"$DB_USER" -p"$DB_PASSWORD" "$DB_NAME" -e "SHOW TABLES;" 2>/dev/null | wc -l)
if [ "$TABLE_COUNT" -gt 1 ]; then
    print_success "数据库表创建成功 (共 $((TABLE_COUNT-1)) 个表)"
    print_info "主要表列表:"
    mysql --protocol=TCP -h"$DB_HOST" -P"$DB_PORT" -u"$DB_USER" -p"$DB_PASSWORD" "$DB_NAME" -e "SHOW TABLES;" 2>/dev/null | tail -n +2 | head -10
else
    print_warning "数据库表较少，可能需要检查迁移文件"
fi

# 测试关键表是否存在
print_step "检查关键表..."
REQUIRED_TABLES=("tenants" "orders" "order_details" "users" "roles")
for table in "${REQUIRED_TABLES[@]}"; do
    if mysql --protocol=TCP -h"$DB_HOST" -P"$DB_PORT" -u"$DB_USER" -p"$DB_PASSWORD" "$DB_NAME" -e "DESCRIBE $table;" > /dev/null 2>&1; then
        print_success "表 '$table' 存在"
    else
        print_warning "表 '$table' 不存在"
    fi
done

print_success "测试数据库设置完成！"
echo
print_info "环境变量设置："
echo "export TEST_DATABASE_URL=\"mysql://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME\""
echo "export DATABASE_URL=\"mysql://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME\""
echo
print_info "现在可以运行集成测试了："
echo "  ./scripts/dev.sh integration"
echo "  或者: cargo test --test order_repository_integration_tests"
echo
print_info "如果需要重置数据库，可以运行："
echo "  mysql -h$DB_HOST -P$DB_PORT -u$DB_USER -p$DB_PASSWORD -e \"DROP DATABASE IF EXISTS $DB_NAME;\"" 