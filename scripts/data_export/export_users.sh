#!/bin/bash
# export_users.sh - 导出users表数据

# 颜色输出
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# 数据库连接信息
DB_USER="root"
DB_PASS="Nihaoccj123"
DB_HOST="127.0.0.1"
DB_PORT="3307"
DB_NAME="tenant_saas"

# 导出目录
EXPORT_DIR="scripts/data_export"
mkdir -p "$EXPORT_DIR"

# 导出users表数据
echo -e "${GREEN}正在导出users表数据...${NC}"
mysqldump -h"$DB_HOST" -P"$DB_PORT" -u"$DB_USER" -p"$DB_PASS" \
    --no-create-info \
    --complete-insert \
    --skip-extended-insert \
    --skip-comments \
    "$DB_NAME" users > "$EXPORT_DIR/users_data.sql"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}users表数据已成功导出到 $EXPORT_DIR/users_data.sql${NC}"
else
    echo -e "${RED}导出users表数据失败${NC}"
    exit 1
fi

# 创建迁移文件
echo -e "${GREEN}正在创建迁移文件...${NC}"
./scripts/migrate.sh create init_users_data

# 获取最新的迁移文件
LATEST_MIGRATION=$(ls -1 migrations/*.sql | sort | tail -1)
LATEST_REVERT=$(ls -1 migrations/revert/*.sql | sort | tail -1)

# 添加导出的数据到迁移文件
echo -e "${GREEN}正在将导出的数据添加到迁移文件 $LATEST_MIGRATION${NC}"
echo "-- Add migration script here" > "$LATEST_MIGRATION"
echo "-- 初始化users表数据" >> "$LATEST_MIGRATION"
echo "" >> "$LATEST_MIGRATION"
cat "$EXPORT_DIR/users_data.sql" >> "$LATEST_MIGRATION"

# 创建回滚文件
echo -e "${GREEN}正在创建回滚文件 $LATEST_REVERT${NC}"
echo "-- Revert migration script" > "$LATEST_REVERT"
echo "-- 删除初始化的users表数据" >> "$LATEST_REVERT"
echo "" >> "$LATEST_REVERT"
echo "DELETE FROM users WHERE id IN (" >> "$LATEST_REVERT"

# 提取用户ID并添加到回滚文件
grep -o "VALUES ([0-9]*," "$EXPORT_DIR/users_data.sql" | sed 's/VALUES (//' | sed 's/,//' | sed '$!s/$/,/' >> "$LATEST_REVERT"
echo ");" >> "$LATEST_REVERT"

echo -e "${GREEN}迁移文件和回滚文件已创建完成${NC}"
echo -e "${GREEN}迁移文件: $LATEST_MIGRATION${NC}"
echo -e "${GREEN}回滚文件: $LATEST_REVERT${NC}"
echo -e "${YELLOW}请检查迁移文件和回滚文件的内容，确保正确性${NC}"
echo -e "${YELLOW}然后运行 './scripts/migrate.sh run' 应用迁移${NC}"

exit 0 