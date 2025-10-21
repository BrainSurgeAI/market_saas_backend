#!/bin/bash

# 快速修复常见 Clippy 错误的脚本

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

print_section "快速 Clippy 修复工具"

# 1. 自动修复可修复的问题
print_info "应用自动修复..."
if cargo clippy --fix --allow-dirty --allow-staged --all-targets --all-features; then
    print_success "自动修复完成"
else
    print_warning "自动修复过程中有一些问题，继续检查..."
fi

# 2. 检查剩余问题
print_info "检查剩余的 Clippy 问题..."
clippy_output=$(cargo clippy --all-targets --all-features 2>&1 || true)

# 3. 分析常见问题并提供修复建议
echo "$clippy_output" | while IFS= read -r line; do
    if [[ $line == *"direct implementation of \`ToString\`"* ]]; then
        print_warning "发现 ToString 直接实现"
        echo "  建议：将 impl ToString 改为 impl fmt::Display"
        echo "  示例："
        echo "    impl fmt::Display for YourType {"
        echo "        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {"
        echo "            write!(f, \"your_string\")"
        echo "        }"
        echo "    }"
    elif [[ $line == *"empty line after"* ]]; then
        print_warning "发现多余的空行"
        echo "  建议：移除文档注释或属性后的空行"
    elif [[ $line == *"redundant redefinition"* ]]; then
        print_warning "发现冗余的变量重新定义"
        echo "  建议：移除 'let var = var;' 这样的冗余赋值"
    elif [[ $line == *"should implement trait"* ]]; then
        print_warning "建议实现标准 trait"
        echo "  建议：使用标准库的 trait，如 FromStr 而不是自定义的 from_str 方法"
    elif [[ $line == *"too many arguments"* ]]; then
        print_warning "函数参数过多"
        echo "  建议：考虑使用结构体来组织参数"
    fi
done

# 4. 提供快速修复命令
print_section "快速修复命令"

echo "常用修复命令："
echo "1. 自动修复所有可修复的问题："
echo "   cargo clippy --fix --allow-dirty --allow-staged"
echo ""
echo "2. 检查特定类型的问题："
echo "   cargo clippy -- -W clippy::to_string_trait_impl"
echo "   cargo clippy -- -W clippy::empty_line_after_outer_attr"
echo ""
echo "3. 允许特定的 lint（如果无法修复）："
echo "   #[allow(clippy::too_many_arguments)]"
echo "   #[allow(clippy::only_used_in_recursion)]"

# 5. 运行最终检查
print_section "最终检查"
print_info "运行最终的 Clippy 检查..."

if cargo clippy --all-targets --all-features -- -D warnings; then
    print_success "所有 Clippy 检查通过！"
else
    print_warning "仍有一些 Clippy 警告需要手动处理"
    echo ""
    echo "如果某些警告无法修复，可以考虑："
    echo "1. 在代码中添加 #[allow(clippy::lint_name)]"
    echo "2. 在 Cargo.toml 中配置 clippy.toml"
    echo "3. 重构代码以符合 Clippy 建议"
fi

print_section "完成"
print_info "运行 './scripts/quick_test.sh' 来验证修复效果" 