#!/bin/bash
set -e

# 颜色输出
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}=== 应用启动 ===${NC}"

# 检查环境变量
if [ -z "$DATABASE_URL" ]; then
    if [ -f .env ]; then
        export $(grep -v '^#' .env | xargs)
    else
        echo -e "${RED}错误: 未设置 DATABASE_URL 环境变量${NC}"
        exit 1
    fi
fi

# 环境变量控制是否执行迁移
# 默认为false，可以通过环境变量PERFORM_MIGRATION=true启用
PERFORM_MIGRATION=${PERFORM_MIGRATION:-false}

if [ "$PERFORM_MIGRATION" = "true" ]; then
    echo -e "${YELLOW}执行数据库迁移...${NC}"
    
    # 执行迁移
    ./scripts/migrate.sh run
    
    echo -e "${GREEN}迁移完成${NC}"
else
    echo -e "${YELLOW}迁移已禁用，跳过迁移步骤${NC}"
fi

# 启动应用程序
echo -e "${GREEN}启动应用程序${NC}"
exec ./market-saas-backend 