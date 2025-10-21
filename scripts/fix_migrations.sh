#!/bin/bash

# 批量修复迁移文件中的 INSERT INTO 语句
# 将 INSERT INTO 改为 INSERT IGNORE INTO 以避免主键冲突

set -e

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_section() {
    echo -e "\n${BLUE}=== $1 ===${NC}\n"
}

# 检查是否在项目根目录
if [ ! -f "Cargo.toml" ]; then
    print_error "请在项目根目录运行此脚本"
    exit 1
fi

print_section "修复迁移文件中的 INSERT INTO 语句"

# 查找所有迁移文件
migration_files=$(find migrations -name "*.sql" -type f)

if [ -z "$migration_files" ]; then
    print_warning "未找到迁移文件"
    exit 0
fi

fixed_count=0
total_count=0

for file in $migration_files; do
    total_count=$((total_count + 1))
    
    # 检查文件是否包含 INSERT INTO（但不是 INSERT IGNORE INTO）
    if grep -q "INSERT INTO" "$file" && ! grep -q "INSERT IGNORE INTO" "$file"; then
        print_info "修复文件: $file"
        
        # 创建备份
        cp "$file" "$file.backup"
        
        # 替换 INSERT INTO 为 INSERT IGNORE INTO
        sed -i 's/INSERT INTO/INSERT IGNORE INTO/g' "$file"
        
        # 验证修改
        if grep -q "INSERT IGNORE INTO" "$file"; then
            print_success "已修复: $file"
            fixed_count=$((fixed_count + 1))
            
            # 删除备份文件
            rm "$file.backup"
        else
            print_error "修复失败: $file"
            # 恢复备份
            mv "$file.backup" "$file"
        fi
    else
        print_info "跳过文件: $file (无需修复或已修复)"
    fi
done

print_section "修复完成"
print_info "总文件数: $total_count"
print_info "已修复文件数: $fixed_count"

if [ $fixed_count -gt 0 ]; then
    print_success "所有迁移文件已修复完成！"
    print_info "现在可以重新运行测试了"
else
    print_info "没有需要修复的文件"
fi 