#!/bin/bash
# export_all_data.sh - 批量导出多个表数据并创建迁移文件

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

# 显示帮助
show_help() {
    echo "用法: $0 [选项]"
    echo "选项:"
    echo "  -h, --help     显示帮助信息"
    echo "  -a, --all      导出所有表数据"
    echo "  -t, --tables   指定要导出的表，用逗号分隔"
    echo "  -n, --name     指定迁移文件的名称，默认为 init_all_data"
    echo "示例:"
    echo "  $0 -a"
    echo "  $0 -t users,roles,permissions -n init_user_system_data"
}

# 默认值
EXPORT_ALL=false
TABLES=""
MIGRATION_NAME="init_all_data"

# 解析参数
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -a|--all)
            EXPORT_ALL=true
            shift
            ;;
        -t|--tables)
            TABLES="$2"
            shift 2
            ;;
        -n|--name)
            MIGRATION_NAME="$2"
            shift 2
            ;;
        *)
            echo -e "${RED}错误: 未知选项 $1${NC}"
            show_help
            exit 1
            ;;
    esac
done

# 检查参数
if [ "$EXPORT_ALL" = false ] && [ -z "$TABLES" ]; then
    echo -e "${RED}错误: 请指定要导出的表或使用 -a 导出所有表${NC}"
    show_help
    exit 1
fi

# 获取所有表
if [ "$EXPORT_ALL" = true ]; then
    echo -e "${YELLOW}获取所有表...${NC}"
    ALL_TABLES=$(mysql -h"$DB_HOST" -P"$DB_PORT" -u"$DB_USER" -p"$DB_PASS" -N -e "USE $DB_NAME; SHOW TABLES;" 2>/dev/null)
    TABLES=$(echo "$ALL_TABLES" | tr '\n' ',' | sed 's/,$//')
    echo -e "${GREEN}找到以下表: $TABLES${NC}"
fi

# 创建迁移文件
echo -e "${GREEN}正在创建迁移文件...${NC}"
./scripts/migrate.sh create "$MIGRATION_NAME"

# 获取最新的迁移文件
LATEST_MIGRATION=$(ls -1 migrations/*.sql | sort | tail -1)
LATEST_REVERT=$(ls -1 migrations/revert/*.sql | sort | tail -1)

# 初始化迁移文件
echo "-- Add migration script here" > "$LATEST_MIGRATION"
echo "-- 初始化数据" >> "$LATEST_MIGRATION"
echo "" >> "$LATEST_MIGRATION"

# 初始化回滚文件
echo "-- Revert migration script" > "$LATEST_REVERT"
echo "-- 删除初始化的数据" >> "$LATEST_REVERT"
echo "" >> "$LATEST_REVERT"
echo "SET FOREIGN_KEY_CHECKS = 0;" >> "$LATEST_REVERT"
echo "" >> "$LATEST_REVERT"

# 处理每个表
IFS=',' read -ra TABLE_ARRAY <<< "$TABLES"
for TABLE in "${TABLE_ARRAY[@]}"; do
    TABLE=$(echo "$TABLE" | tr -d ' ')
    
    echo -e "${YELLOW}处理表 $TABLE...${NC}"
    
    # 检查表是否存在
    TABLE_EXISTS=$(mysql -h"$DB_HOST" -P"$DB_PORT" -u"$DB_USER" -p"$DB_PASS" -e "USE $DB_NAME; SHOW TABLES LIKE '$TABLE';" 2>/dev/null | grep -o "$TABLE")
    
    if [ -z "$TABLE_EXISTS" ]; then
        echo -e "${RED}警告: 表 $TABLE 不存在，跳过${NC}"
        continue
    fi
    
    # 导出表数据
    echo -e "${GREEN}正在导出 $TABLE 表数据...${NC}"
    mysqldump -h"$DB_HOST" -P"$DB_PORT" -u"$DB_USER" -p"$DB_PASS" \
        --no-create-info \
        --complete-insert \
        --skip-extended-insert \
        --skip-comments \
        --skip-set-charset \
        --skip-add-locks \
        --skip-disable-keys \
        --quote-names \
        "$DB_NAME" "$TABLE" > "$EXPORT_DIR/${TABLE}_data.sql.temp"
    
    # 清理导出的SQL文件，去除MySQL特有的注释和锁表语句
    grep -v "^\/\*\!" "$EXPORT_DIR/${TABLE}_data.sql.temp" | grep -v "^LOCK TABLES" | grep -v "^UNLOCK TABLES" > "$EXPORT_DIR/${TABLE}_data.sql"
    rm "$EXPORT_DIR/${TABLE}_data.sql.temp"
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}$TABLE 表数据已成功导出到 $EXPORT_DIR/${TABLE}_data.sql${NC}"
        
        # 检查是否有数据
        if [ -s "$EXPORT_DIR/${TABLE}_data.sql" ]; then
            # 添加表数据到迁移文件
            echo "-- $TABLE 表数据" >> "$LATEST_MIGRATION"
            cat "$EXPORT_DIR/${TABLE}_data.sql" >> "$LATEST_MIGRATION"
            echo "" >> "$LATEST_MIGRATION"
            
            # 获取表的主键列名
            PRIMARY_KEY=$(mysql -h"$DB_HOST" -P"$DB_PORT" -u"$DB_USER" -p"$DB_PASS" -N -e "
                SELECT COLUMN_NAME 
                FROM INFORMATION_SCHEMA.KEY_COLUMN_USAGE 
                WHERE TABLE_SCHEMA = '$DB_NAME' 
                AND TABLE_NAME = '$TABLE' 
                AND CONSTRAINT_NAME = 'PRIMARY';" 2>/dev/null)
            
            # 添加回滚语句
            echo "-- 删除 $TABLE 表数据" >> "$LATEST_REVERT"
            if [ -z "$PRIMARY_KEY" ]; then
                echo "TRUNCATE TABLE $TABLE;" >> "$LATEST_REVERT"
            else
                echo "DELETE FROM $TABLE WHERE $PRIMARY_KEY IN (" >> "$LATEST_REVERT"
                grep -o "VALUES ([^,]*," "$EXPORT_DIR/${TABLE}_data.sql" | sed 's/VALUES (//' | sed 's/,//' | sed '$!s/$/,/' >> "$LATEST_REVERT"
                echo ");" >> "$LATEST_REVERT"
            fi
            echo "" >> "$LATEST_REVERT"
        else
            echo -e "${YELLOW}表 $TABLE 没有数据，跳过${NC}"
        fi
    else
        echo -e "${RED}导出 $TABLE 表数据失败，跳过${NC}"
    fi
done

# 完成回滚文件
echo "SET FOREIGN_KEY_CHECKS = 1;" >> "$LATEST_REVERT"

echo -e "${GREEN}迁移文件和回滚文件已创建完成${NC}"
echo -e "${GREEN}迁移文件: $LATEST_MIGRATION${NC}"
echo -e "${GREEN}回滚文件: $LATEST_REVERT${NC}"
echo -e "${YELLOW}请检查迁移文件和回滚文件的内容，确保正确性${NC}"
echo -e "${YELLOW}然后运行 './scripts/migrate.sh run' 应用迁移${NC}"

exit 0 