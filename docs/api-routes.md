# API Routes Documentation

### 测试状态 ok 代表后端接口测试完成，但还没有与前端联调， P代表联调完成

## Public Path (公开路径)

| 方法 | 路径 | 描述 | 测试状态 |
|------|------|------|----------|
| GET | `/api/v1/login` | 用户登录 | P |
| GET | `/api/v1/register` | 用户注册页面 | |
| GET | `/api/v1/categories` | 获取一级分类 | P |
| GET | `/api/v1/price_announcements` | 获取价格公告 | P |
| GET | `/api/v1/openapi.json` | OpenAPI文档 | ok |

## Protected Path (受保护路径)

### 工作区

| 方法 | 路径 | 描述 | 测试状态 | 角色 |
|------|------|------|----------|
| GET | `/api/v1/workspace` | 工作区欢迎信息 | ok | 无角色使用，可以删除 |

### 用户管理

| 方法 | 路径 | 描述 | 测试状态 | 权限 |
|------|------|------|----------|
| GET | `/api/v1/users/me` | 获取当前用户信息 | p | all |
| GET | `/api/v1/notifications` | 获取用户通知列表 | ok | all |
| PATCH | `/api/v1/notifications/{message_id}` | 标记信息已读 | ok | all |
| PATCH | `/api/v1/users/{username}` | 更新用户 | |
| PATCH | `/api/v1/password/reset` | 重置用户密码 | ok | all |

### 租户管理

| 方法 | 路径 | 描述 | 测试状态 |
|------|------|------|----------|
| GET | `/api/v1/tenants/me` | 获取当前租户详情 | p | all |
| GET | `/api/v1/users/{username}/tenants` | 获取用户所属租户信息 | |
| GET | `/api/v1/tenants` | 根据市场获取租户列表 | |
| GET | `/api/v1/financials` | 获取租户财务信息 | |
| GET | `/api/v1/tenants/{hashed_name}/users` | 获取租户用户列表 | |
| POST | `/api/v1/tenants/{hashed_name}/users` | 创建租户用户 | |
| POST | `/api/v1/tenants` | 创建租户 | |
| PATCH | `/api/v1/tenants/{hashed_name}` | 租户自更新 | |
| PATCH | `/api/v1/markets/{market_hash}/tenants/{tenant_hash}` | 市场更新租户 | |
| PATCH | `/api/v1/markets/{market_hash}/tenants/{tenant_hash}/status` | 禁用租户 | |
| POST | `/api/v1/register` | 用户注册 | |
| POST | `/api/v1/superadmin/login` | 超级管理员登录 | |
| PATCH | `/api/v1/tenants/{hashed_name}/users/{username}/enable` | 启用用户 | |
| DELETE | `/api/v1/tenants/{hashed_name}/users/{username}` | 删除用户 | |
| PATCH | `/api/v1/tenants/{hashed_name}/users/{username}/disable` | 禁用用户 | |

### 角色和权限

| 方法 | 路径 | 描述 | 测试状态 |
|------|------|------|----------|
| GET | `/api/v1/roles` | 获取所有角色 | |
| GET | `/api/v1/permissions` | 获取所有权限 | |
| GET | `/api/v1/roles/{role_id}/permissions` | 根据角色ID获取权限 | |
| GET | `/api/v1/roles/{role_id}` | 根据ID获取角色 | |
| GET | `/api/v1/tenants/{tenant_name}/types/{tenant_type}/roles` | 根据租户类型获取角色 | |
| GET | `/api/v1/menus` | 根据角色获取菜单 | ok |
| PUT | `/api/v1/roles/{role_id}` | 根据ID更新角色 | |
| PUT | `/api/v1/permissions/{permission_id}` | 根据ID更新权限 | |
| PATCH | `/api/v1/roles/{role_id}/permissions` | 更新角色权限 | |
| PATCH | `/api/v1/superadmin/password` | 重置超级管理员密码 | |

### 产品管理

| 方法 | 路径 | 描述 | 测试状态 |
|------|------|------|----------|
| GET | `/api/v1/product_price_status_stats` | 获取产品价格状态统计 | ok |
| GET | `/api/v1/tenants/{tenant_hash}/products/{product_code}` | 获取产品详情 | |
| GET | `/api/v1/tenants/{tenant_hash}/products` | 获取产品概览 | |
| GET | `/api/v1/customers/{tenant_hash}/products` | 获取产品列表 | |
| GET | `/api/v1/processing_fees` | 获取加工费用 | |
| POST | `/api/v1/users/{username}/product_prices` | 批量创建产品价格 | |
| PUT | `/api/v1/tenants/{tenant_hash}/products/{product_code}` | 更新产品 | |
| PATCH | `/api/v1/tenants/{tenant_hash}/products/{product_code}/enable` | 启用产品 | |
| DELETE | `/api/v1/tenants/{tenant_hash}/products/{product_code}/archive` | 归档产品 | |

### 订单管理

| 方法 | 路径 | 描述 | 测试状态 |
|------|------|------|----------|
| GET | `/api/v1/tenants/{tenant_hash}/orders` | 获取订单列表 | |
| GET | `/api/v1/tenants/{tenant_hash}/orders/{order_code}` | 根据订单编码获取订单 | |
| GET | `/api/v1/tenants/{tenant_hash}/orders/after_sale` | 获取供应商售后订单 | |
| GET | `/api/v1/providers/{provider_hash}/orders/today-summary` | 获取供应商今日订单汇总 | |
| POST | `/api/v1/customers/{customer_hash}/orders` | 创建订单 | |
| POST | `/api/v1/customers/{customer_hash}/orders/{order_code}/operations` | 退货/换货订单 | |
| POST | `/api/v1/tenants/{tenant_hash}/orders/{order_code}/operations` | 更新订单状态 | |
| PUT | `/api/v1/providers/{provider_hash}/orders/{order_code}/update-quantities` | 更新实际数量 | |
| PATCH | `/api/v1/markets/{market_hash}/orders/{order_code}/dispatch` | 派单 | |
| PATCH | `/api/v1/providers/{provider_hash}/orders/{order_code}/processing` | 更新订单状态为处理中 | |
| PATCH | `/api/v1/tenants/{tenant_hash}/orders/{order_code}/operations` | 更新订单状态 | |

### 价格管理

| 方法 | 路径 | 描述 | 测试状态 |
|------|------|------|----------|
| GET | `/api/v1/product_prices` | 获取价格员已发布价格 | |
| PATCH | `/api/v1/tenants/{tenant_hash}/prices/aprox_price` | 获取近似价格 | |

### 分类管理

| 方法 | 路径 | 描述 | 测试状态 |
|------|------|------|----------|
| GET | `/api/v1/categories-tree` | 获取分类树（包含子分类） | ok |

### 折扣管理

| 方法 | 路径 | 描述 | 测试状态 |
|------|------|------|----------|
| GET | `/api/v1/tenants/{tenant_hash}/discounts` | 获取客户折扣列表 | |
| POST | `/api/v1/tenants/{tenant_hash}/discounts` | 根据租户和分类创建折扣 | |
| PATCH | `/api/v1/tenants/{tenant_hash}/discounts/{discount_id}` | 更新折扣 | |

### 配送员管理

| 方法 | 路径 | 描述 | 测试状态 |
|------|------|------|----------|
| GET | `/api/v1/tenants/{tenant_hash}/delivery_staffs` | 获取供应商配送员列表 | |
| POST | `/api/v1/tenants/{tenant_hash}/delivery_staffs` | 创建配送员 | |
| PATCH | `/api/v1/tenants/{tenant_hash}/delivery_staffs/{id_card}/status` | 禁用/启用配送员 | |

### 对账单管理

| 方法 | 路径 | 描述 | 测试状态 |
|------|------|------|----------|
| GET | `/api/v1/reconciliation_statements` | 根据租户和日期获取对账单 | |
| GET | `/api/v1/reconciliation_statements/{id}` | 根据ID获取对账单订单 | |
| POST | `/api/v1/reconciliation_statements` | 创建对账单 | |

## 路由参数说明

### 通用参数
| 参数 | 描述 |
|------|------|
| `{username}` | 用户标识符 |
| `{tenant_hash}`/`{tenant_name}` | 租户标识符（哈希或名称） |
| `{customer_hash}` | 客户标识符 |
| `{provider_hash}` | 供应商标识符 |
| `{market_hash}` | 市场标识符 |
| `{role_id}` | 角色标识符 |
| `{permission_id}` | 权限标识符 |
| `{order_code}` | 订单标识符 |
| `{product_code}` | 产品标识符 |
| `{message_id}` | 通知消息标识符 |
| `{discount_id}` | 折扣标识符 |
| `{id_card}` | 配送员身份证号 |
| `{id}` | 通用ID标识符 |

### 多段参数
| 路径模式 | 描述 |
|----------|------|
| `tenants/{tenant_name}/types/{tenant_type}/roles` | 租户和租户类型组合 |
| `markets/{market_hash}/tenants/{tenant_hash}` | 市场和租户组合 |
| `tenants/{tenant_hash}/products/{product_code}` | 租户和产品组合 |
| `providers/{provider_hash}/orders/{order_code}` | 供应商和订单组合 |
| `customers/{customer_hash}/orders/{order_code}` | 客户和订单组合 |

---

## 统计信息

- **总计API端点数量**：58个
- **涵盖主要功能模块**：用户管理、租户管理、产品管理、订单管理、价格管理、分类管理、折扣管理、配送员管理、对账单管理
- **HTTP方法分布**：
  - GET: 24个端点
  - POST: 12个端点
  - PUT: 4个端点
  - PATCH: 17个端点
  - DELETE: 1个端点

**使用说明**：在"测试状态"列中标记：
- ✅ - 测试通过
- ❌ - 测试失败
- ⚠️ - 部分功能有问题
- 🔄 - 正在测试中
- 空白 - 未测试

*最后更新时间：$(date)*