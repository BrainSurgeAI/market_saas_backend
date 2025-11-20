# 数据库导出脚本使用说明

## 脚本位置
`scripts/export_database.sh`

## 功能特性

- ✅ 支持导出表结构和数据
- ✅ 支持 Docker 容器连接和直接连接两种方式
- ✅ 支持导出整个数据库或指定表
- ✅ 支持只导出表结构或只导出数据
- ✅ 支持压缩导出文件
- ✅ 自动创建备份目录和信息文件

## 快速开始

### 1. 使用默认配置（推荐）

如果你的 MySQL 运行在 Docker 中，并且端口映射到了 3306：

```bash
# 导出整个数据库（表结构 + 数据）
./scripts/export_database.sh

# 导出指定表
./scripts/export_database.sh -t users,roles,permissions

# 只导出表结构
./scripts/export_database.sh --structure-only

# 只导出数据
./scripts/export_database.sh --data-only

# 压缩导出
./scripts/export_database.sh --compress
```

### 2. 使用 Docker 容器名称

如果你的 MySQL 运行在 Docker 容器中，可以直接指定容器名称：

```bash
# 查找 MySQL 容器名称
docker ps | grep mysql

# 使用容器名称导出
./scripts/export_database.sh -c mysql_market_saas
```

### 3. 自定义连接参数

```bash
./scripts/export_database.sh \
  -u root \
  -p your_password \
  -H 127.0.0.1 \
  -P 3306 \
  -d tenant_saas \
  -o ./backup
```

### 4. 使用环境变量

```bash
export MYSQL_USER=root
export MYSQL_PASS=your_password
export MYSQL_HOST=127.0.0.1
export MYSQL_PORT=3306
export MYSQL_DB=tenant_saas
export MYSQL_CONTAINER=mysql_container  # 可选
export OUTPUT_DIR=./backup  # 可选

./scripts/export_database.sh
```

## 参数说明

| 参数 | 简写 | 说明 | 默认值 |
|------|------|------|--------|
| `--help` | `-h` | 显示帮助信息 | - |
| `--user` | `-u` | 数据库用户名 | root |
| `--password` | `-p` | 数据库密码 | Nihaoccj123 |
| `--host` | `-H` | 数据库主机 | 127.0.0.1 |
| `--port` | `-P` | 数据库端口 | 3307 |
| `--database` | `-d` | 数据库名称 | tenant_saas |
| `--container` | `-c` | Docker 容器名称 | - |
| `--output` | `-o` | 输出目录 | ./database_backup |
| `--tables` | `-t` | 指定表（逗号分隔） | 所有表 |
| `--structure-only` | - | 只导出表结构 | - |
| `--data-only` | - | 只导出数据 | - |
| `--compress` | - | 压缩导出文件 | - |

## 输出文件

脚本会在指定的输出目录（默认 `./database_backup`）中创建以下文件：

1. **SQL 文件**: `{数据库名}_{类型}_{时间戳}.sql`
   - `full`: 表结构 + 数据
   - `structure`: 只有表结构
   - `data`: 只有数据

2. **信息文件**: `{数据库名}_{类型}_{时间戳}.info`
   - 包含导出时间、数据库信息等元数据

3. **压缩文件**（如果使用 `--compress`）: `{数据库名}_{类型}_{时间戳}.sql.gz`

## 示例

### 示例 1: 导出整个数据库

```bash
./scripts/export_database.sh
```

输出：
```
========================================
MySQL 数据库导出工具
========================================

检查数据库连接...
数据库连接成功
找到以下表: users, roles, permissions, ...

正在导出数据库...
导出成功！
文件: ./database_backup/tenant_saas_full_20240101_120000.sql
大小: 2.5M
信息文件: ./database_backup/tenant_saas_full_20240101_120000.info
```

### 示例 2: 使用 Docker 容器导出

```bash
# 先查找容器名称
docker ps | grep mysql
# 输出: abc123def456  mysql:8.0  ... 0.0.0.0:3307->3306/tcp  mysql_db

# 使用容器名称导出
./scripts/export_database.sh -c mysql_db
```

### 示例 3: 只导出表结构并压缩

```bash
./scripts/export_database.sh --structure-only --compress
```

### 示例 4: 导出指定表

```bash
./scripts/export_database.sh -t users,roles,permissions -o ./backup
```

## 常见问题

### Q: 如何找到 MySQL Docker 容器名称？

```bash
docker ps | grep mysql
# 或者
docker ps --format "table {{.Names}}\t{{.Image}}\t{{.Ports}}" | grep -i mysql
```

### Q: 如何恢复导出的数据库？

```bash
# 恢复整个数据库
mysql -h127.0.0.1 -P3307 -uroot -p tenant_saas < database_backup/tenant_saas_full_20240101_120000.sql

# 如果使用 Docker
docker exec -i mysql_market_saas mysql --default-character-set=utf8mb4 -uroot -p3m4c3n9q8J! market_saas < scripts/database_backup/market_saas_full_20251119_151412.sql   

# 如果是压缩文件
gunzip < database_backup/tenant_saas_full_20240101_120000.sql.gz | mysql -h127.0.0.1 -P3307 -uroot -p tenant_saas
```

### Q: 连接失败怎么办？

1. 检查 MySQL 是否运行：
   ```bash
   docker ps | grep mysql
   # 或
   mysql -h127.0.0.1 -P3307 -uroot -p -e "SELECT 1"
   ```

2. 检查端口是否正确：
   ```bash
   netstat -tlnp | grep 3307
   # 或
   docker ps | grep mysql
   ```

3. 检查用户名和密码是否正确

4. 如果使用 Docker，确保容器名称正确

### Q: 如何定时备份？

可以使用 cron 定时任务：

```bash
# 编辑 crontab
crontab -e

# 每天凌晨 2 点备份
0 2 * * * /path/to/project/scripts/export_database.sh --compress -o /path/to/backup
```

## 注意事项

1. 确保有足够的磁盘空间存储备份文件
2. 定期清理旧的备份文件
3. 备份文件包含敏感数据，请妥善保管
4. 建议在生产环境中使用压缩选项以节省空间
5. 如果数据库很大，导出可能需要一些时间

