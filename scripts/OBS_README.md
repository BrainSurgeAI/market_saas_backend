# 华为云OBS临时URL管理脚本

这个脚本用于管理华为云OBS存储中产品图片的临时URL，自动生成并刷新过期的URL。

## 功能特性

- 自动从数据库获取产品代码
- 检查OBS中对应的产品图片是否存在
- 生成临时URL并存储到数据库
- 支持URL过期后自动刷新
- 完整的日志记录
- 灵活的命令行操作

## 安装依赖

```bash
pip install -r requirements.txt
```

## 配置

1. 复制环境变量模板：
```bash
cp .env.example .env
```

2. 编辑 `.env` 文件，填入你的配置信息：
```env
# 华为云OBS配置
OBS_ACCESS_KEY=your_access_key_here
OBS_SECRET_KEY=your_secret_key_here
OBS_ENDPOINT=https://obs.cn-north-1.myhuaweicloud.com
OBS_BUCKET_NAME=your-bucket-name

# 数据库配置
DB_HOST=localhost
DB_PORT=3306
DB_USER=root
DB_PASSWORD=your_password_here
DB_NAME=market_saas

# URL配置
URL_EXPIRES_IN=3600
```

## 数据库设置

执行SQL脚本创建必要的表和视图：

```bash
mysql -u username -p database_name < temp_image_urls.sql
```

## 使用方法

### 基本用法

```bash
# 生成所有产品图片的临时URL（默认操作）
python huawei_obs_manager.py

# 重新生成所有URL，先清空现有记录
python huawei_obs_manager.py rebuild clear

# 只刷新过期的URL
python huawei_obs_manager.py refresh

# 清理过期的URL记录
python huawei_obs_manager.py cleanup
```

### 命令说明

- **无参数**：默认生成所有产品图片的临时URL
- **rebuild**：重新生成所有URL
  - `rebuild clear`：先清空现有记录再重新生成
- **refresh**：只刷新已过期的URL
- **cleanup**：删除过期的URL记录

## 数据库表结构

### temp_image_urls 表

| 字段名 | 类型 | 说明 |
|--------|------|------|
| id | INT | 主键ID |
| product_code | VARCHAR(10) | 产品代码 |
| bucket_name | VARCHAR(100) | OBS桶名 |
| object_key | VARCHAR(500) | OBS对象键 |
| temp_url | TEXT | 临时URL |
| expires_at | DATETIME | 过期时间 |
| created_at | TIMESTAMP | 创建时间 |
| updated_at | TIMESTAMP | 更新时间 |

### v_active_temp_urls 视图

查询当前有效的临时URL，包含剩余时间等信息。

## 日志记录

脚本会生成 `obs_manager.log` 日志文件，包含：
- 操作详情
- 成功/失败统计
- 错误信息
- 性能数据

## 定时任务

建议设置定时任务定期刷新URL：

```bash
# 每小时刷新过期的URL
0 * * * * cd /path/to/scripts && python huawei_obs_manager.py refresh

# 每天凌晨2点清理过期记录
0 2 * * * cd /path/to/scripts && python huawei_obs_manager.py cleanup
```

## 注意事项

1. 确保OBS中的图片命名格式为 `products/{product_code}.webp`
2. 产品代码必须在数据库的 `products` 表中存在
3. 脚本只会处理未禁用的产品 (`is_disabled = FALSE`)
4. 临时URL默认过期时间为1小时，可在配置中调整

## 故障排除

### 常见问题

1. **OBS连接失败**
   - 检查访问密钥是否正确
   - 确认endpoint地址是否正确
   - 检查网络连接

2. **数据库连接失败**
   - 检查数据库服务是否运行
   - 确认连接参数是否正确
   - 检查防火墙设置

3. **图片不存在**
   - 确认OBS中的图片路径是否正确
   - 检查产品代码是否匹配
   - 验证文件格式是否为 `.webp`