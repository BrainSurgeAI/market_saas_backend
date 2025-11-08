#!/bin/bash
# quick_export.sh - 快速导出数据库的便捷脚本
# 这是一个简单的包装脚本，方便快速使用

# 切换到脚本所在目录
cd "$(dirname "$0")/.."

# 运行导出脚本
./scripts/export_database.sh "$@"

