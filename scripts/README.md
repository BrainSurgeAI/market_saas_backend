# Market SaaS Backend - 测试脚本指南

本目录包含了 Market SaaS Backend 项目的完整测试工具链，提供从开发到部署的全方位测试支持。

## 📁 脚本概览

| 脚本 | 用途 | 适用场景 |
|------|------|----------|
| `run_all_tests.sh` | 全面测试套件 | 发布前、代码审查 |
| `quick_test.sh` | 快速测试 | 日常开发 |
| `ci_test.sh` | CI/CD 测试 | 持续集成环境 |
| `fix_clippy.sh` | Clippy 修复助手 | 代码质量改进 |
| `quick_clippy_fix.sh` | 快速 Clippy 修复 | 快速修复常见问题 |
| `coverage_report.sh` | 覆盖率报告生成器 | 测试覆盖率分析 |

## 🚀 快速开始

### 日常开发工作流
```bash
# 1. 快速检查代码质量
./scripts/quick_test.sh

# 2. 快速修复常见 Clippy 问题
./scripts/quick_clippy_fix.sh

# 3. 深度修复 Clippy 警告
./scripts/fix_clippy.sh

# 4. 生成覆盖率报告
./scripts/coverage_report.sh --open
```

### 发布前检查
```bash
# 运行完整测试套件
./scripts/run_all_tests.sh --clean
```

## 📋 详细脚本说明

### 1. run_all_tests.sh - 全面测试套件

**功能特性：**
- ✅ 代码格式检查
- ✅ Clippy 静态分析
- ✅ 单元测试
- ✅ 集成测试
- ✅ 文档测试
- ✅ 模块特定测试
- ✅ 特性标志测试
- ✅ 代码覆盖率分析
- ✅ 安全审计
- ✅ 依赖检查
- ✅ 性能测试（可选）

**使用方法：**
```bash
# 基本用法
./scripts/run_all_tests.sh

# 跳过覆盖率分析
./scripts/run_all_tests.sh --no-coverage

# 跳过安全审计
./scripts/run_all_tests.sh --no-security

# 包含性能测试
./scripts/run_all_tests.sh --performance

# 先清理构建产物
./scripts/run_all_tests.sh --clean

# 查看帮助
./scripts/run_all_tests.sh --help
```

**输出示例：**
```
============================================
Market SaaS Backend - Comprehensive Test Suite
============================================

--- Checking Dependencies ---
✓ All required dependencies are available

--- Checking Code Formatting ---
✓ Code formatting is correct

--- Running Clippy Lints ---
✓ Clippy lints passed

--- Running Unit Tests ---
✓ Unit tests passed

Total Tests: 10
Passed: 10
Failed: 0
Success Rate: 100%
✓ All tests completed successfully! 🎉
```

### 2. quick_test.sh - 快速测试

**适用场景：**
- 日常开发中的快速验证
- 提交前的基本检查
- IDE 集成测试

**包含检查：**
- 代码格式检查
- Clippy 检查
- 单元测试
- 集成测试

**使用方法：**
```bash
./scripts/quick_test.sh
```

**执行时间：** 通常 30-60 秒

### 3. ci_test.sh - CI/CD 测试

**专为持续集成环境设计：**
- 严格的检查标准
- 详细的错误报告
- 不同的退出码
- CI 产物生成

**使用方法：**
```bash
# 基本 CI 测试
./scripts/ci_test.sh

# 跳过覆盖率检查
./scripts/ci_test.sh --skip-coverage

# 跳过安全审计
./scripts/ci_test.sh --skip-security

# 设置覆盖率阈值
./scripts/ci_test.sh --coverage-threshold 85
```

**退出码：**
- `0`: 成功
- `1`: 格式检查失败
- `2`: Lint 检查失败
- `3`: 测试失败
- `4`: 覆盖率不足
- `5`: 安全问题

**生成的 CI 产物：**
- `target/ci-artifacts/cobertura.xml` - 覆盖率报告
- `target/ci-artifacts/tarpaulin-report.html` - HTML 覆盖率报告
- `target/ci-artifacts/test-summary.txt` - 测试摘要

### 4. fix_clippy.sh - Clippy 修复助手

**功能：**
- 自动应用可修复的 Clippy 建议
- 显示剩余需要手动修复的问题
- 提供修复建议

**使用方法：**
```bash
./scripts/fix_clippy.sh
```

**常见手动修复建议：**
1. **函数参数过多 (>7)** - 考虑使用结构体
2. **文档注释后的空行** - 移除多余的空行
3. **直接实现 ToString** - 改为实现 Display trait
4. **缺少 Default 实现** - 添加 `#[derive(Default)]`
5. **应该实现标准 trait** - 使用如 FromStr 等标准 trait

### 5. coverage_report.sh - 覆盖率报告生成器

**生成多种格式的覆盖率报告：**
- HTML 格式（交互式查看）
- XML 格式（CI/CD 集成）
- JSON 格式（程序化分析）

**使用方法：**
```bash
# 生成覆盖率报告
./scripts/coverage_report.sh

# 生成后自动打开浏览器
./scripts/coverage_report.sh --open

# 不清理之前的数据
./scripts/coverage_report.sh --no-clean

# 设置覆盖率阈值
./scripts/coverage_report.sh --threshold 85

# 查看帮助
./scripts/coverage_report.sh --help
```

**生成的报告：**
- `target/coverage/tarpaulin-report.html` - 交互式 HTML 报告
- `target/coverage/cobertura.xml` - XML 格式报告
- `target/coverage/tarpaulin-report.json` - JSON 格式报告
- `target/coverage/coverage-summary.txt` - 摘要报告

## 🔧 环境要求

### 必需工具
- `cargo` - Rust 包管理器
- `rustc` - Rust 编译器

### 可选工具（推荐安装）
```bash
# 代码覆盖率工具
cargo install cargo-tarpaulin

# 安全审计工具
cargo install cargo-audit

# 依赖检查工具
cargo install cargo-outdated

# 未使用依赖检查（需要 nightly）
cargo install cargo-udeps
```

## 📊 工作流建议

### 功能开发工作流
```bash
# 1. 开始开发前
./scripts/quick_test.sh

# 2. 开发过程中（频繁运行）
./scripts/quick_test.sh

# 3. 功能完成后
./scripts/fix_clippy.sh
./scripts/quick_test.sh

# 4. 提交前
./scripts/run_all_tests.sh
```

### 代码审查工作流
```bash
# 1. 审查前运行完整测试
./scripts/run_all_tests.sh --clean

# 2. 检查覆盖率
./scripts/coverage_report.sh --open

# 3. 验证 CI 兼容性
./scripts/ci_test.sh
```

### 发布工作流
```bash
# 1. 完整测试套件
./scripts/run_all_tests.sh --clean --performance

# 2. 安全检查
./scripts/run_all_tests.sh --no-coverage

# 3. 生成最终覆盖率报告
./scripts/coverage_report.sh --threshold 80
```

## 🐛 故障排除

### 常见问题

**1. cargo-tarpaulin 安装失败**
```bash
# 在 Ubuntu/Debian 上
sudo apt-get install libssl-dev pkg-config

# 在 macOS 上
brew install openssl pkg-config

# 然后重新安装
cargo install cargo-tarpaulin
```

**2. 测试超时**
```bash
# 增加超时时间（在脚本中修改 TEST_TIMEOUT）
export TEST_TIMEOUT=600  # 10 分钟
```

**3. 覆盖率报告生成失败**
```bash
# 检查是否有足够的磁盘空间
df -h

# 清理之前的构建产物
cargo clean
```

**4. Clippy 检查失败**
```bash
# 更新 Clippy 到最新版本
rustup update
rustup component add clippy

# 查看具体错误
cargo clippy --all-targets --all-features -- -D warnings
```

**5. 权限问题**
```bash
# 确保脚本有执行权限
chmod +x scripts/*.sh

# 如果在 Windows WSL 中
dos2unix scripts/*.sh
```

### 性能优化建议

**1. 增量编译**
```bash
# 启用增量编译（开发环境）
export CARGO_INCREMENTAL=1
```

**2. 缓存优化**
```bash
# 使用 sccache 加速编译
cargo install sccache
export RUSTC_WRAPPER=sccache
```

**3. 测试配置**
```bash
# 注意：本项目的测试脚本已配置为单线程模式
# 这是为了确保测试的稳定性和兼容性
export RUST_TEST_THREADS=1
```

## 🔗 集成指南

### GitHub Actions 集成
```yaml
name: Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Install dependencies
        run: |
          cargo install cargo-tarpaulin cargo-audit
      - name: Run CI tests
        run: ./scripts/ci_test.sh
      - name: Upload coverage
        uses: codecov/codecov-action@v1
        with:
          file: target/ci-artifacts/cobertura.xml
```

### GitLab CI 集成
```yaml
test:
  stage: test
  script:
    - cargo install cargo-tarpaulin cargo-audit
    - ./scripts/ci_test.sh
  artifacts:
    reports:
      coverage_report:
        coverage_format: cobertura
        path: target/ci-artifacts/cobertura.xml
    paths:
      - target/ci-artifacts/
```

### VS Code 集成
在 `.vscode/tasks.json` 中添加：
```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Quick Test",
      "type": "shell",
      "command": "./scripts/quick_test.sh",
      "group": "test",
      "presentation": {
        "echo": true,
        "reveal": "always",
        "focus": false,
        "panel": "shared"
      }
    }
  ]
}
```

## 📈 最佳实践

1. **频繁运行快速测试** - 在开发过程中经常运行 `quick_test.sh`
2. **提交前完整测试** - 每次提交前运行 `run_all_tests.sh`
3. **定期检查覆盖率** - 使用 `coverage_report.sh` 监控测试覆盖率
4. **及时修复 Clippy 警告** - 使用 `fix_clippy.sh` 保持代码质量
5. **CI/CD 集成** - 在 CI 管道中使用 `ci_test.sh`

## 📞 支持

如果遇到问题或需要帮助：
1. 查看脚本的 `--help` 选项
2. 检查本文档的故障排除部分
3. 查看项目的 issue 跟踪器
4. 联系开发团队

---

**注意：** 所有脚本都设计为从项目根目录运行。确保在运行脚本前位于正确的目录中。 