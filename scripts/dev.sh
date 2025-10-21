#!/bin/bash

# 开发辅助脚本
set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

function print_step() {
    echo -e "${GREEN}[STEP]${NC} $1"
}

function print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

function print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查代码格式
function check_format() {
    print_step "检查代码格式..."
    if cargo fmt -- --check; then
        echo "✅ 代码格式正确"
    else
        print_error "代码格式不正确，运行 cargo fmt 修复"
        return 1
    fi
}

# 运行 clippy 检查
function check_clippy() {
    print_step "运行 Clippy 检查..."
    if cargo clippy -- -D warnings; then
        echo "✅ Clippy 检查通过"
    else
        print_error "Clippy 检查失败"
        return 1
    fi
}

# 运行单元测试
function run_unit_tests() {
    print_step "运行单元测试..."
    
    echo "🧪 运行 Auth DTO 测试..."
    cargo test dto::auth::tests --lib -- --nocapture
    
    echo "🧪 运行 Discount DTO 测试..."
    cargo test dto::discount::tests --lib -- --nocapture
    
    echo "🧪 运行 Price DTO 测试..."
    cargo test dto::price::tests --lib -- --nocapture
    
    echo "🧪 运行 Products DTO 测试..."
    cargo test dto::products::tests --lib -- --nocapture
    
    echo "🧪 运行 Users DTO 测试..."
    cargo test dto::users::tests --lib -- --nocapture
    
    echo "✅ 所有 DTO 单元测试通过"
}

# 运行所有测试
function run_all_tests() {
    print_step "运行所有测试..."
    cargo test --lib
    echo "✅ 所有测试通过"
}

# 生成测试覆盖率
function generate_coverage() {
    print_step "生成测试覆盖率报告..."
    
    if ! command -v cargo-tarpaulin &> /dev/null; then
        print_warning "cargo-tarpaulin 未安装，正在安装..."
        cargo install cargo-tarpaulin
    fi
    
    cargo tarpaulin --out html --output-dir ./coverage
    echo "✅ 覆盖率报告生成完成，查看 ./coverage/tarpaulin-report.html"
}

# 快速检查（格式+clippy+单元测试）
function quick_check() {
    print_step "执行快速检查..."
    check_format
    check_clippy
    run_unit_tests
    echo "🎉 快速检查全部通过！"
}

# 完整检查（包含覆盖率）
function full_check() {
    print_step "执行完整检查..."
    check_format
    check_clippy
    run_all_tests
    generate_coverage
    echo "🎉 完整检查全部通过！"
}

# 修复代码格式
function fix_format() {
    print_step "修复代码格式..."
    cargo fmt
    echo "✅ 代码格式已修复"
}

# 交互式测试菜单
function interactive_test() {
    echo "请选择要运行的测试:"
    echo "1) Auth DTO 测试"
    echo "2) Discount DTO 测试"
    echo "3) Price DTO 测试"
    echo "4) Products DTO 测试"
    echo "5) Users DTO 测试"
    echo "6) 所有 DTO 测试"
    echo "7) 返回主菜单"
    
    read -p "输入选择 (1-7): " choice
    
    case $choice in
        1) cargo test dto::auth::tests --lib -- --nocapture ;;
        2) cargo test dto::discount::tests --lib -- --nocapture ;;
        3) cargo test dto::price::tests --lib -- --nocapture ;;
        4) cargo test dto::products::tests --lib -- --nocapture ;;
        5) cargo test dto::users::tests --lib -- --nocapture ;;
        6) run_unit_tests ;;
        7) return ;;
        *) echo "无效选择" && interactive_test ;;
    esac
}

# 主菜单
function main_menu() {
    echo "==================================="
    echo "🦀 Rust 开发辅助脚本"
    echo "==================================="
    echo "1) 快速检查 (格式+clippy+单元测试)"
    echo "2) 完整检查 (包含覆盖率)"
    echo "3) 只运行单元测试"
    echo "4) 交互式测试"
    echo "5) 检查代码格式"
    echo "6) 修复代码格式"
    echo "7) 运行 Clippy"
    echo "8) 生成覆盖率报告"
    echo "9) 退出"
    echo "==================================="
    
    read -p "输入选择 (1-9): " choice
    
    case $choice in
        1) quick_check ;;
        2) full_check ;;
        3) run_unit_tests ;;
        4) interactive_test ;;
        5) check_format ;;
        6) fix_format ;;
        7) check_clippy ;;
        8) generate_coverage ;;
        9) exit 0 ;;
        *) echo "无效选择" && main_menu ;;
    esac
    
    echo ""
    read -p "按 Enter 键继续..."
    main_menu
}

# 解析命令行参数
case "${1:-menu}" in
    "quick"|"q") quick_check ;;
    "full"|"f") full_check ;;
    "test"|"t") run_unit_tests ;;
    "format") check_format ;;
    "fix") fix_format ;;
    "clippy") check_clippy ;;
    "coverage"|"c") generate_coverage ;;
    "menu"|"m"|*) main_menu ;;
esac 