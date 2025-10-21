#!/bin/bash
# migrate.sh - 数据库迁移管理脚本

# 颜色输出
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# 显示帮助
show_help() {
    echo "用法: $0 [命令] [参数]"
    echo "命令:"
    echo "  create [name]    创建新迁移"
    echo "  run              应用所有待处理的迁移"
    echo "  revert           回滚最近的迁移"
    echo "  info             显示迁移状态"
    echo "  init             从现有数据库初始化迁移"
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

# 从环境变量中提取数据库连接信息
DB_URL=${DATABASE_URL#*://}
DB_USER=${DB_URL%%:*}
DB_URL=${DB_URL#*:}
DB_PASS=${DB_URL%%@*}
DB_URL=${DB_URL#*@}
DB_HOST=${DB_URL%%:*}
DB_URL=${DB_URL#*:}
DB_PORT=${DB_URL%%/*}
DB_URL=${DB_URL#*/}
DB_NAME=${DB_URL%%\?*}

echo "数据库连接信息："
echo "用户: $DB_USER"
echo "主机: $DB_HOST"
echo "端口: $DB_PORT"
echo "数据库: $DB_NAME"

# 检查数据库是否存在，如果不存在则创建
check_and_create_database() {
    echo -e "${YELLOW}检查数据库 $DB_NAME 是否存在...${NC}"
    
    # 检查数据库是否存在
    DB_EXISTS=$(mysql -h"$DB_HOST" -P"$DB_PORT" -u"$DB_USER" -p"$DB_PASS" -e "SHOW DATABASES LIKE '$DB_NAME';" 2>/dev/null | grep -o "$DB_NAME")
    
    if [ -z "$DB_EXISTS" ]; then
        echo -e "${YELLOW}数据库 $DB_NAME 不存在，正在创建...${NC}"
        mysql -h"$DB_HOST" -P"$DB_PORT" -u"$DB_USER" -p"$DB_PASS" -e "CREATE DATABASE IF NOT EXISTS $DB_NAME CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;" 2>/dev/null
        if [ $? -eq 0 ]; then
            echo -e "${GREEN}数据库 $DB_NAME 创建成功${NC}"
        else
            echo -e "${RED}创建数据库 $DB_NAME 失败，请检查连接信息和权限${NC}"
        fi
    else
        echo -e "${GREEN}数据库 $DB_NAME 已存在${NC}"
    fi
}

# 解析命令
case "$1" in
    create)
        if [ -z "$2" ]; then
            echo -e "${RED}错误: 请提供迁移名称${NC}"
            exit 1
        fi
        # 检查并创建数据库
        check_and_create_database
        
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
        # 检查并创建数据库
        check_and_create_database
        
        echo -e "${GREEN}应用待处理的迁移...${NC}"
        sqlx migrate run
        ;;
    revert)
        # 检查并创建数据库
        check_and_create_database
        
        echo -e "${GREEN}回滚最近的迁移...${NC}"
        LATEST=$(ls -1 migrations/*.sql | sort | tail -1)
        FILENAME=$(basename "$LATEST")
        if [ -f "migrations/revert/$FILENAME" ]; then
            mysql -h"$DB_HOST" -P"$DB_PORT" -u"$DB_USER" -p"$DB_PASS" "$DB_NAME" < "migrations/revert/$FILENAME"
            sqlx migrate revert
            echo -e "${GREEN}迁移已回滚${NC}"
        else
            echo -e "${RED}错误: 回滚文件不存在: migrations/revert/$FILENAME${NC}"
            exit 1
        fi
        ;;
    info)
        # 检查并创建数据库
        check_and_create_database
        
        echo -e "${GREEN}迁移状态:${NC}"
        sqlx migrate info
        ;;
    init)
        # 检查并创建数据库
        check_and_create_database
        
        echo -e "${GREEN}从现有数据库初始化迁移...${NC}"
        
        # 检查是否已存在迁移
        if [ -d "migrations" ] && [ "$(ls -A migrations 2>/dev/null)" ]; then
            echo -e "${RED}错误: migrations 目录已存在且不为空${NC}"
            echo -e "${RED}请备份并清空该目录后重试${NC}"
            exit 1
        fi
        
        # 创建迁移目录
        mkdir -p migrations
        
        # 创建初始迁移
        sqlx migrate add initial
        
        # 获取初始迁移文件路径
        INITIAL_MIGRATION=$(ls -1 migrations/*.sql | sort | head -1)
        
        echo -e "${GREEN}导出数据库结构到 $INITIAL_MIGRATION${NC}"
        
        # 导出数据库结构（不包含数据）
        mysqldump -h"$DB_HOST" -P"$DB_PORT" -u"$DB_USER" -p"$DB_PASS" --no-data --skip-comments \
            --skip-set-charset --skip-add-locks --skip-add-drop-table \
            "$DB_NAME" > "$INITIAL_MIGRATION.temp"
        
        # 添加 SQLx 迁移注释
        echo "-- Add migration script here" > "$INITIAL_MIGRATION"
        cat "$INITIAL_MIGRATION.temp" >> "$INITIAL_MIGRATION"
        rm "$INITIAL_MIGRATION.temp"
        
        # 创建回滚文件
        mkdir -p migrations/revert
        FILENAME=$(basename "$INITIAL_MIGRATION")
        echo "-- Revert migration script
-- 警告: 这将删除所有表！谨慎使用！
-- SET FOREIGN_KEY_CHECKS = 0;
-- 在此添加 DROP TABLE 语句
-- SET FOREIGN_KEY_CHECKS = 1;" > "migrations/revert/$FILENAME"
        
        echo -e "${GREEN}初始迁移文件已创建: $INITIAL_MIGRATION${NC}"
        echo -e "${GREEN}请检查并编辑迁移文件以确保其正确性${NC}"
        echo -e "${GREEN}然后运行 'sqlx migrate run' 标记初始迁移为已应用${NC}"
        ;;
    *)
        show_help
        exit 1
        ;;
esac

exit 0 