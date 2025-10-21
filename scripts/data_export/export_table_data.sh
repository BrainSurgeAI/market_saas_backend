#!/bin/bash
# export_table_data.sh - 导出指定表数据并创建迁移文件

# 颜色输出
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# 显示帮助
show_help() {
    echo "用法: $0 <表名> [迁移名称]"
    echo "参数:"
    echo "  <表名>       要导出数据的表名"
    echo "  [迁移名称]   可选，迁移文件的名称，默认为 init_<表名>_data"
    echo "示例:"
    echo "  $0 users"
    echo "  $0 roles init_roles_and_permissions"
}

# 检查参数
if [ -z "$1" ]; then
    echo -e "${RED}错误: 请提供表名${NC}"
    show_help
    exit 1
fi

TABLE_NAME="$1"
MIGRATION_NAME="${2:-init_${TABLE_NAME}_data}"

# 数据库连接信息
DB_USER="root"
DB_PASS="Nihaoccj123"
DB_HOST="127.0.0.1"
DB_PORT="3307"
DB_NAME="tenant_saas"

# 导出目录
EXPORT_DIR="scripts/data_export"
mkdir -p "$EXPORT_DIR"

# 检查表是否存在
echo -e "${YELLOW}检查表 $TABLE_NAME 是否存在...${NC}"
TABLE_EXISTS=$(mysql -h"$DB_HOST" -P"$DB_PORT" -u"$DB_USER" -p"$DB_PASS" -e "USE $DB_NAME; SHOW TABLES LIKE '$TABLE_NAME';" 2>/dev/null | grep -o "$TABLE_NAME")

if [ -z "$TABLE_EXISTS" ]; then
    echo -e "${RED}错误: 表 $TABLE_NAME 不存在${NC}"
    exit 1
fi

# 导出表数据
echo -e "${GREEN}正在导出 $TABLE_NAME 表数据...${NC}"
mysqldump -h"$DB_HOST" -P"$DB_PORT" -u"$DB_USER" -p"$DB_PASS" \
    --no-create-info \
    --complete-insert \
    --skip-extended-insert \
    --skip-comments \
    --skip-set-charset \
    --skip-add-locks \
    --skip-disable-keys \
    --quote-names \
    "$DB_NAME" "$TABLE_NAME" > "$EXPORT_DIR/${TABLE_NAME}_data.sql.temp"

# 清理导出的SQL文件，去除MySQL特有的注释和锁表语句
grep -v "^\/\*\!" "$EXPORT_DIR/${TABLE_NAME}_data.sql.temp" | grep -v "^LOCK TABLES" | grep -v "^UNLOCK TABLES" > "$EXPORT_DIR/${TABLE_NAME}_data.sql"
rm "$EXPORT_DIR/${TABLE_NAME}_data.sql.temp"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}$TABLE_NAME 表数据已成功导出到 $EXPORT_DIR/${TABLE_NAME}_data.sql${NC}"
else
    echo -e "${RED}导出 $TABLE_NAME 表数据失败${NC}"
    exit 1
fi

# 创建迁移文件
echo -e "${GREEN}正在创建迁移文件...${NC}"
./scripts/migrate.sh create "$MIGRATION_NAME"

# 获取最新的迁移文件
LATEST_MIGRATION=$(ls -1 migrations/*.sql | sort | tail -1)
LATEST_REVERT=$(ls -1 migrations/revert/*.sql | sort | tail -1)

# 添加导出的数据到迁移文件
echo -e "${GREEN}正在将导出的数据添加到迁移文件 $LATEST_MIGRATION${NC}"
echo "-- Add migration script here" > "$LATEST_MIGRATION"
echo "-- 初始化 $TABLE_NAME 表数据" >> "$LATEST_MIGRATION"
echo "" >> "$LATEST_MIGRATION"
cat "$EXPORT_DIR/${TABLE_NAME}_data.sql" >> "$LATEST_MIGRATION"

# 创建回滚文件
echo -e "${GREEN}正在创建回滚文件 $LATEST_REVERT${NC}"
echo "-- Revert migration script" > "$LATEST_REVERT"
echo "-- 删除初始化的 $TABLE_NAME 表数据" >> "$LATEST_REVERT"
echo "" >> "$LATEST_REVERT"

# 获取表的主键列名
PRIMARY_KEY=$(mysql -h"$DB_HOST" -P"$DB_PORT" -u"$DB_USER" -p"$DB_PASS" -N -e "
    SELECT COLUMN_NAME 
    FROM INFORMATION_SCHEMA.KEY_COLUMN_USAGE 
    WHERE TABLE_SCHEMA = '$DB_NAME' 
    AND TABLE_NAME = '$TABLE_NAME' 
    AND CONSTRAINT_NAME = 'PRIMARY';" 2>/dev/null)

if [ -z "$PRIMARY_KEY" ]; then
    echo -e "${YELLOW}警告: 无法获取表 $TABLE_NAME 的主键，使用 TRUNCATE 语句进行回滚${NC}"
    echo "TRUNCATE TABLE $TABLE_NAME;" >> "$LATEST_REVERT"
else
    echo -e "${GREEN}使用主键 $PRIMARY_KEY 创建 DELETE 语句${NC}"
    echo "DELETE FROM $TABLE_NAME WHERE $PRIMARY_KEY IN (" >> "$LATEST_REVERT"
    
    # 提取主键值并添加到回滚文件
    grep -o "VALUES ([^,]*," "$EXPORT_DIR/${TABLE_NAME}_data.sql" | sed 's/VALUES (//' | sed 's/,//' | sed '$!s/$/,/' >> "$LATEST_REVERT"
    echo ");" >> "$LATEST_REVERT"
fi

echo -e "${GREEN}迁移文件和回滚文件已创建完成${NC}"
echo -e "${GREEN}迁移文件: $LATEST_MIGRATION${NC}"
echo -e "${GREEN}回滚文件: $LATEST_REVERT${NC}"
echo -e "${YELLOW}请检查迁移文件和回滚文件的内容，确保正确性${NC}"
echo -e "${YELLOW}然后运行 './scripts/migrate.sh run' 应用迁移${NC}"

exit 0 