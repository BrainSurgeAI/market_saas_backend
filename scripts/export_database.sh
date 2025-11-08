#!/bin/bash
# export_database.sh - 导出 MySQL 数据库的表结构和数据
# 支持 Docker 容器和直接连接两种方式

set -e

# 颜色输出
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 默认配置（可以从环境变量或命令行参数覆盖）
DB_USER="${MYSQL_USER:-root}"
DB_PASS="${MYSQL_PASS:-Nihaoccj123}"
DB_HOST="${MYSQL_HOST:-127.0.0.1}"
DB_PORT="${MYSQL_PORT:-3307}"
DB_NAME="${MYSQL_DB:-tenant_saas}"
DOCKER_CONTAINER="${MYSQL_CONTAINER:-}"
OUTPUT_DIR="${OUTPUT_DIR:-./database_backup}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# 显示帮助信息
show_help() {
    cat << EOF
用法: $0 [选项]

导出 MySQL 数据库的表结构和数据

选项:
    -h, --help              显示帮助信息
    -u, --user USER        数据库用户名 (默认: root)
    -p, --password PASS     数据库密码 (默认: Nihaoccj123)
    -H, --host HOST         数据库主机 (默认: 127.0.0.1)
    -P, --port PORT         数据库端口 (默认: 3307)
    -d, --database DB       数据库名称 (默认: tenant_saas)
    -c, --container NAME    Docker 容器名称（如果使用 Docker）
    -o, --output DIR        输出目录 (默认: ./database_backup)
    -t, --tables TABLES     指定要导出的表，用逗号分隔（默认: 所有表）
    --structure-only        只导出表结构，不导出数据
    --data-only             只导出数据，不导出表结构
    --compress              压缩导出的文件（使用 gzip）

环境变量:
    MYSQL_USER              数据库用户名
    MYSQL_PASS              数据库密码
    MYSQL_HOST              数据库主机
    MYSQL_PORT              数据库端口
    MYSQL_DB                数据库名称
    MYSQL_CONTAINER         Docker 容器名称
    OUTPUT_DIR              输出目录

示例:
    # 使用默认配置导出整个数据库
    $0

    # 导出指定表
    $0 -t users,roles,permissions

    # 使用 Docker 容器
    $0 -c mysql_container

    # 只导出表结构
    $0 --structure-only

    # 只导出数据
    $0 --data-only

    # 压缩导出
    $0 --compress

    # 使用环境变量
    export MYSQL_USER=root
    export MYSQL_PASS=password
    export MYSQL_DB=mydb
    $0
EOF
}

# 解析命令行参数
EXPORT_STRUCTURE=true
EXPORT_DATA=true
COMPRESS=false
TABLES=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -u|--user)
            DB_USER="$2"
            shift 2
            ;;
        -p|--password)
            DB_PASS="$2"
            shift 2
            ;;
        -H|--host)
            DB_HOST="$2"
            shift 2
            ;;
        -P|--port)
            DB_PORT="$2"
            shift 2
            ;;
        -d|--database)
            DB_NAME="$2"
            shift 2
            ;;
        -c|--container)
            DOCKER_CONTAINER="$2"
            shift 2
            ;;
        -o|--output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        -t|--tables)
            TABLES="$2"
            shift 2
            ;;
        --structure-only)
            EXPORT_STRUCTURE=true
            EXPORT_DATA=false
            shift
            ;;
        --data-only)
            EXPORT_STRUCTURE=false
            EXPORT_DATA=true
            shift
            ;;
        --compress)
            COMPRESS=true
            shift
            ;;
        *)
            echo -e "${RED}错误: 未知选项 $1${NC}"
            show_help
            exit 1
            ;;
    esac
done

# 检查必要的工具
check_dependencies() {
    local missing_tools=()
    
    if [ -n "$DOCKER_CONTAINER" ]; then
        if ! command -v docker &> /dev/null; then
            missing_tools+=("docker")
        fi
    fi
    
    if ! command -v mysqldump &> /dev/null; then
        if [ -z "$DOCKER_CONTAINER" ]; then
            missing_tools+=("mysqldump")
        fi
    fi
    
    if [ "$COMPRESS" = true ] && ! command -v gzip &> /dev/null; then
        missing_tools+=("gzip")
    fi
    
    if [ ${#missing_tools[@]} -gt 0 ]; then
        echo -e "${RED}错误: 缺少必要的工具: ${missing_tools[*]}${NC}"
        exit 1
    fi
}

# 执行 MySQL 命令（支持 Docker 和直接连接）
mysql_exec() {
    local sql="$1"
    if [ -n "$DOCKER_CONTAINER" ]; then
        docker exec -i "$DOCKER_CONTAINER" mysql -u"$DB_USER" -p"$DB_PASS" -e "$sql" 2>/dev/null
    else
        mysql -h"$DB_HOST" -P"$DB_PORT" -u"$DB_USER" -p"$DB_PASS" -e "$sql" 2>/dev/null
    fi
}

# 执行 mysqldump（支持 Docker 和直接连接）
mysqldump_exec() {
    local args="$@"
    if [ -n "$DOCKER_CONTAINER" ]; then
        docker exec "$DOCKER_CONTAINER" mysqldump -u"$DB_USER" -p"$DB_PASS" $args 2>/dev/null
    else
        mysqldump -h"$DB_HOST" -P"$DB_PORT" -u"$DB_USER" -p"$DB_PASS" $args 2>/dev/null
    fi
}

# 检查数据库连接
check_connection() {
    echo -e "${BLUE}检查数据库连接...${NC}"
    
    if [ -n "$DOCKER_CONTAINER" ]; then
        if ! docker ps --format "{{.Names}}" | grep -q "^${DOCKER_CONTAINER}$"; then
            echo -e "${RED}错误: Docker 容器 '$DOCKER_CONTAINER' 未运行${NC}"
            exit 1
        fi
    fi
    
    if ! mysql_exec "USE $DB_NAME;" > /dev/null 2>&1; then
        echo -e "${RED}错误: 无法连接到数据库 '$DB_NAME'${NC}"
        echo -e "${YELLOW}请检查连接参数:${NC}"
        echo -e "  主机: $DB_HOST"
        echo -e "  端口: $DB_PORT"
        echo -e "  用户: $DB_USER"
        echo -e "  数据库: $DB_NAME"
        if [ -n "$DOCKER_CONTAINER" ]; then
            echo -e "  容器: $DOCKER_CONTAINER"
        fi
        exit 1
    fi
    
    echo -e "${GREEN}数据库连接成功${NC}"
}

# 获取所有表名
get_tables() {
    if [ -n "$TABLES" ]; then
        echo "$TABLES" | tr ',' ' '
    else
        mysql_exec "USE $DB_NAME; SHOW TABLES;" | tail -n +2 | tr '\n' ' '
    fi
}

# 主函数
main() {
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}MySQL 数据库导出工具${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
    
    check_dependencies
    check_connection
    
    # 创建输出目录
    mkdir -p "$OUTPUT_DIR"
    
    # 获取表列表
    local table_list=$(get_tables)
    if [ -z "$table_list" ]; then
        echo -e "${YELLOW}警告: 数据库 '$DB_NAME' 中没有找到表${NC}"
        exit 0
    fi
    
    echo -e "${GREEN}找到以下表: $(echo $table_list | tr ' ' ',')${NC}"
    echo ""
    
    # 构建输出文件名
    local suffix=""
    if [ "$EXPORT_STRUCTURE" = true ] && [ "$EXPORT_DATA" = true ]; then
        suffix="full"
    elif [ "$EXPORT_STRUCTURE" = true ]; then
        suffix="structure"
    else
        suffix="data"
    fi
    
    local output_file="${OUTPUT_DIR}/${DB_NAME}_${suffix}_${TIMESTAMP}.sql"
    
    # 构建 mysqldump 参数
    local dump_args=""
    
    if [ "$EXPORT_STRUCTURE" = true ] && [ "$EXPORT_DATA" = false ]; then
        dump_args="--no-data"
    elif [ "$EXPORT_STRUCTURE" = false ] && [ "$EXPORT_DATA" = true ]; then
        dump_args="--no-create-info"
    fi
    
    dump_args="$dump_args --single-transaction --routines --triggers --events"
    dump_args="$dump_args --add-drop-table --complete-insert --extended-insert"
    dump_args="$dump_args --skip-comments --skip-set-charset"
    
    # 导出数据库
    echo -e "${BLUE}正在导出数据库...${NC}"
    
    if [ -n "$TABLES" ]; then
        # 导出指定表
        local table_array=($(echo "$TABLES" | tr ',' ' '))
        mysqldump_exec $dump_args "$DB_NAME" "${table_array[@]}" > "$output_file"
    else
        # 导出整个数据库
        mysqldump_exec $dump_args "$DB_NAME" > "$output_file"
    fi
    
    if [ $? -eq 0 ]; then
        local file_size=$(du -h "$output_file" | cut -f1)
        echo -e "${GREEN}导出成功！${NC}"
        echo -e "${GREEN}文件: $output_file${NC}"
        echo -e "${GREEN}大小: $file_size${NC}"
        
        # 压缩文件（如果启用）
        if [ "$COMPRESS" = true ]; then
            echo -e "${BLUE}正在压缩文件...${NC}"
            gzip "$output_file"
            local compressed_file="${output_file}.gz"
            local compressed_size=$(du -h "$compressed_file" | cut -f1)
            echo -e "${GREEN}压缩完成！${NC}"
            echo -e "${GREEN}压缩文件: $compressed_file${NC}"
            echo -e "${GREEN}压缩后大小: $compressed_size${NC}"
        fi
        
        # 创建信息文件
        local info_file="${OUTPUT_DIR}/${DB_NAME}_${suffix}_${TIMESTAMP}.info"
        cat > "$info_file" << EOF
数据库导出信息
================
导出时间: $(date '+%Y-%m-%d %H:%M:%S')
数据库名称: $DB_NAME
数据库主机: $DB_HOST
数据库端口: $DB_PORT
数据库用户: $DB_USER
导出类型: $suffix
导出表: $(echo $table_list | tr ' ' ',')
EOF
        if [ -n "$DOCKER_CONTAINER" ]; then
            echo "Docker 容器: $DOCKER_CONTAINER" >> "$info_file"
        fi
        echo -e "${GREEN}信息文件: $info_file${NC}"
        
    else
        echo -e "${RED}导出失败！${NC}"
        exit 1
    fi
    
    echo ""
    echo -e "${BLUE}========================================${NC}"
    echo -e "${GREEN}导出完成！${NC}"
    echo -e "${BLUE}========================================${NC}"
}

# 运行主函数
main

