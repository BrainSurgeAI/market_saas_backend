# 系统日志表设计与实现文档

## 1. 背景与目标

### 1.1 背景
当前系统中缺乏统一的、结构化的日志记录机制，导致问题追踪困难、性能分析受限，同时也难以满足审计和合规要求。

### 1.2 目标
设计并实现一个全面的系统日志功能，包括：
- 统一的日志表结构
- 高效的日志写入机制
- 灵活的日志查询接口
- 自动化的日志维护流程

## 2. 系统日志表设计

### 2.1 表结构

```sql
CREATE TABLE system_log (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT COMMENT '主键ID',
    event_time TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) COMMENT '事件发生时间，精确到微秒',
    log_level ENUM('DEBUG', 'INFO', 'WARN', 'ERROR', 'FATAL') NOT NULL COMMENT '日志级别',
    category VARCHAR(64) NOT NULL COMMENT '日志分类，如：auth, order, payment等',
    
    -- 上下文信息
    user_id VARCHAR(64) NULL COMMENT '操作用户ID',
    tenant_id INT NULL COMMENT '租户ID',
    tenant_type VARCHAR(16) NULL COMMENT '租户类型',
    ip_address VARCHAR(45) NULL COMMENT '操作IP地址 (IPv4/IPv6)',
    request_id VARCHAR(64) NULL COMMENT '关联请求ID，用于追踪',
    
    -- 操作信息
    component VARCHAR(64) NOT NULL COMMENT '组件/服务名称',
    action VARCHAR(64) NOT NULL COMMENT '操作类型，如：login, create, update, delete',
    resource_type VARCHAR(64) NULL COMMENT '资源类型，如：user, order, product',
    resource_id VARCHAR(128) NULL COMMENT '资源ID',
    
    -- 详细信息
    message TEXT NOT NULL COMMENT '日志消息',
    details JSON NULL COMMENT '详细信息，JSON格式存储灵活数据',
    
    -- 异常信息
    error_code VARCHAR(32) NULL COMMENT '错误代码',
    error_stack TEXT NULL COMMENT '错误堆栈',
    
    -- 性能指标
    execution_time INT UNSIGNED NULL COMMENT '执行时间(毫秒)',
    
    -- 元数据
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '记录创建时间',
    
    PRIMARY KEY (id),
    INDEX idx_event_time (event_time),
    INDEX idx_log_level (log_level),
    INDEX idx_category (category),
    INDEX idx_user_id (user_id),
    INDEX idx_tenant_id (tenant_id),
    INDEX idx_request_id (request_id),
    INDEX idx_component_action (component, action),
    INDEX idx_resource (resource_type, resource_id(50)),
    INDEX idx_error_code (error_code),
    INDEX idx_created_at (created_at)
) ENGINE=InnoDB
  DEFAULT CHARSET=utf8mb4
  COLLATE=utf8mb4_unicode_ci
  COMMENT='系统日志表';
```

### 2.2 分区策略

```sql
ALTER TABLE system_log
PARTITION BY RANGE (TO_DAYS(event_time)) (
    PARTITION p_2023_10 VALUES LESS THAN (TO_DAYS('2023-11-01')),
    PARTITION p_2023_11 VALUES LESS THAN (TO_DAYS('2023-12-01')),
    PARTITION p_2023_12 VALUES LESS THAN (TO_DAYS('2024-01-01')),
    PARTITION p_future VALUES LESS THAN MAXVALUE
);
```

### 2.3 辅助视图

```sql
-- 错误日志视图
CREATE VIEW v_error_logs AS
SELECT 
    id, event_time, category, user_id, tenant_id, 
    component, action, resource_type, resource_id, 
    message, error_code, error_stack 
FROM system_log 
WHERE log_level IN ('ERROR', 'FATAL');

-- 用户活动视图
CREATE VIEW v_user_activity AS
SELECT 
    event_time, user_id, tenant_id, ip_address, 
    component, action, resource_type, resource_id
FROM system_log 
WHERE user_id IS NOT NULL;
```

### 2.4 维护存储过程

```sql
-- 清理旧日志
DELIMITER //
CREATE PROCEDURE sp_clean_old_logs(IN retention_days INT)
BEGIN
    SET @cutoff_date = DATE_SUB(CURRENT_DATE(), INTERVAL retention_days DAY);
    DELETE FROM system_log 
    WHERE event_time < @cutoff_date;
END //
DELIMITER ;

-- 日志统计
DELIMITER //
CREATE PROCEDURE sp_log_stats(IN start_date DATE, IN end_date DATE)
BEGIN
    SELECT 
        DATE(event_time) AS log_date,
        log_level,
        category,
        COUNT(*) AS log_count,
        AVG(execution_time) AS avg_execution_time
    FROM system_log
    WHERE event_time BETWEEN start_date AND end_date
    GROUP BY log_date, log_level, category
    ORDER BY log_date DESC, log_count DESC;
END //
DELIMITER ;
```

## 3. 实现计划

### 3.1 数据库变更 (T+1 ~ T+2)
- [ ] 创建system_log表
- [ ] 实现表分区
- [ ] 创建辅助视图
- [ ] 创建维护存储过程
- [ ] 编写回滚脚本

### 3.2 后端实现 (T+3 ~ T+7)

#### 3.2.1 日志DTO设计
- [ ] 创建`SystemLogDTO`类，对应表字段
- [ ] 实现参数验证
- [ ] 添加Builder模式简化创建过程

#### 3.2.2 日志存储库
- [ ] 创建`SystemLogRepository`接口
- [ ] 实现MySQL版本的存储库
- [ ] 添加批量写入支持
- [ ] 实现查询方法（按时间、级别、类别等）

#### 3.2.3 日志服务层
- [ ] 创建`SystemLogService`接口
- [ ] 实现同步写入方法
- [ ] 实现异步写入方法（使用消息队列）
- [ ] 实现查询和统计方法

#### 3.2.4 AOP切面实现
- [ ] 设计`@LogOperation`注解
- [ ] 实现方法调用日志切面
- [ ] 实现异常捕获日志切面

#### 3.2.5 异常处理集成
- [ ] 改造全局异常处理器，加入日志记录
- [ ] 实现错误码标准化

### 3.3 前端实现 (T+8 ~ T+12)

#### 3.3.1 日志查询页面
- [ ] 设计日志查询界面
- [ ] 实现多条件查询表单
- [ ] 开发日志列表展示组件
- [ ] 实现日志详情查看功能

#### 3.3.2 日志统计与分析
- [ ] 设计统计分析仪表盘
- [ ] 实现日志趋势图表
- [ ] 开发错误分布统计
- [ ] 实现用户操作审计报表

#### 3.3.3 日志导出功能
- [ ] 添加多格式导出（CSV, Excel）
- [ ] 实现大数据量分页导出

### 3.4 运维工具 (T+13 ~ T+14)
- [ ] 开发日志清理定时任务
- [ ] 实现日志归档功能
- [ ] 添加分区管理工具
- [ ] 开发性能监控告警

## 4. 测试计划

### 4.1 单元测试
- [ ] 日志DTO验证测试
- [ ] 存储库方法测试
- [ ] 服务层逻辑测试
- [ ] AOP切面功能测试

### 4.2 集成测试
- [ ] 日志写入性能测试
- [ ] 大数据量查询测试
- [ ] 并发写入测试

### 4.3 系统测试
- [ ] 与现有功能集成测试
- [ ] 端到端操作日志验证
- [ ] UI交互测试

## 5. 上线与维护

### 5.1 上线准备
- [ ] 编写详细使用文档
- [ ] 准备开发人员培训材料
- [ ] 制定灰度发布计划

### 5.2 监控与维护
- [ ] 设置日志表大小监控
- [ ] 配置性能指标告警
- [ ] 制定日志清理策略

## 6. 风险与缓解措施

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 日志表增长过快 | 数据库存储压力大 | 实施分区策略、定期归档、设置合理保留期 |
| 日志写入影响业务性能 | 系统响应变慢 | 使用异步写入、批量插入、单独的日志数据库 |
| 查询大量日志数据超时 | 管理界面响应慢 | 优化索引、分页查询、结果缓存、预聚合统计 |
| 日志内容泄露敏感信息 | 信息安全风险 | 实现脱敏处理、访问控制、审计跟踪 |

## 7. 时间进度

| 阶段 | 开始时间 | 结束时间 | 负责人 |
|------|----------|----------|--------|
| 数据库变更 | T+1 | T+2 | 待定 |
| 后端实现 | T+3 | T+7 | 待定 |
| 前端实现 | T+8 | T+12 | 待定 |
| 运维工具 | T+13 | T+14 | 待定 |
| 测试 | T+15 | T+18 | 待定 |
| 文档与培训 | T+19 | T+20 | 待定 |
| 上线 | T+21 | | 全体 |

## 8. 扩展与未来计划

- [ ] 集成ELK或其他日志分析平台
- [ ] 添加AI异常检测
- [ ] 实现跨服务日志追踪
- [ ] 开发自定义分析报表功能 

## 9. 实际应用示例

### 9.1 创建配送员操作日志示例

当通过API创建新的配送员时，系统日志记录如下：

```sql
-- 同步写入日志
INSERT INTO system_log (
    event_time,
    log_level,
    category,
    user_id,
    tenant_id,
    tenant_type,
    ip_address,
    request_id,
    component,
    action,
    resource_type,
    resource_id,
    message,
    details,
    execution_time
) VALUES (
    CURRENT_TIMESTAMP(6),                    -- 当前时间精确到微秒
    'INFO',                                  -- 日志级别：正常信息
    'staff',                                 -- 分类：配送员管理
    'admin_user',                            -- 操作用户：管理员
    123,                                     -- 租户ID：供应商ID
    'PROVIDER',                              -- 租户类型：供应商
    '192.168.1.100',                         -- 客户端IP
    'req-7a8b9c-20230601-123456',           -- 请求ID
    'delivery_staff_service',                -- 组件：配送员服务
    'create',                                -- 动作：创建
    'delivery_staff',                        -- 资源类型：配送员
    'ID_CARD_110101199001011234',           -- 资源ID：身份证号
    '创建配送员成功：张三',                     -- 简明消息
    JSON_OBJECT(                             -- 详细信息
        'name', '张三',
        'phone', '13800138000',
        'id_card', '110101199001011234',
        'provider_hash', 'test_provider_hash',
        'status', 1,
        'remark', '测试备注'
    ),
    45                                       -- 执行时间：45毫秒
);
```

如果创建配送员时发生错误（如身份证号已存在），则记录如下：

```sql
-- 异常情况日志
INSERT INTO system_log (
    event_time,
    log_level,
    category,
    user_id,
    tenant_id,
    tenant_type,
    ip_address,
    request_id,
    component,
    action,
    resource_type,
    resource_id,
    message,
    details,
    error_code,
    error_stack,
    execution_time
) VALUES (
    CURRENT_TIMESTAMP(6),                    -- 当前时间
    'ERROR',                                 -- 日志级别：错误
    'staff',                                 -- 分类：配送员管理
    'admin_user',                            -- 操作用户
    123,                                     -- 租户ID
    'PROVIDER',                              -- 租户类型
    '192.168.1.100',                         -- 客户端IP
    'req-7a8b9c-20230601-123457',           -- 请求ID
    'delivery_staff_service',                -- 组件
    'create',                                -- 动作
    'delivery_staff',                        -- 资源类型
    'ID_CARD_110101199001011234',           -- 资源ID
    '创建配送员失败：身份证号已存在',             -- 错误消息
    JSON_OBJECT(                             -- 详细信息
        'name', '张三',
        'phone', '13800138000',
        'id_card', '110101199001011234',
        'provider_hash', 'test_provider_hash',
        'status', 1
    ),
    'CONFLICT_ERROR',                        -- 错误代码
    'AppError::Conflict at delivery_staff_services.rs:52\n  at create_delivery_staff',  -- 错误堆栈
    30                                       -- 执行时间
);
```

### 9.2 Rust代码示例

在服务层中记录日志的代码示例：

```rust
// 在 create_delivery_staff 服务方法中
async fn create_delivery_staff<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Path(tenant_hash): Path<String>,
    Json(delivery_staff): Json<DeliveryStaffDTO>,
) -> Result<Json<ApiResponse<()>>, AppError>
where
    T: DeliveryStaffRepository + Send + Sync,
{
    // 开始计时
    let start_time = std::time::Instant::now();
    
    // 准备日志基本信息
    let log_data = SystemLogDTO::builder()
        .category("staff")
        .component("delivery_staff_service")
        .action("create")
        .resource_type("delivery_staff")
        .resource_id(&delivery_staff.id_card)
        .user_id(&claims.username)
        .tenant_id(claims.tenant_id)
        .tenant_type(&claims.tenant_type)
        .request_id(&context.request_id)
        .ip_address(context.client_ip.as_deref())
        .build();
    
    // 业务逻辑
    if claims.tenant_type != TenantType::Provider.to_string() {
        // 记录错误日志
        let elapsed = start_time.elapsed().as_millis() as u32;
        let error_message = "Only provider can create delivery staff";
        let error = AppError::Forbidden(error_message.to_string());
        
        log_service.log_error(
            log_data
                .with_level(LogLevel::Error)
                .with_message(&format!("创建配送员失败：{}", error_message))
                .with_error_code("FORBIDDEN_ERROR")
                .with_error_stack(&error.to_string())
                .with_execution_time(elapsed)
        ).await;
        
        return Err(error);
    }

    // 验证数据
    if let Err(validation_error) = delivery_staff.validate() {
        // 记录验证错误日志
        let elapsed = start_time.elapsed().as_millis() as u32;
        
        log_service.log_error(
            log_data
                .with_level(LogLevel::Error)
                .with_message(&format!("创建配送员失败：数据验证错误"))
                .with_details(json!({
                    "validation_errors": validation_error.to_string(),
                    "staff_data": delivery_staff
                }))
                .with_error_code("VALIDATION_ERROR")
                .with_execution_time(elapsed)
        ).await;
        
        return Err(AppError::Validation(validation_error));
    }
    
    // 检查身份证是否存在
    if repo.is_id_card_exists(&delivery_staff.id_card).await? {
        let elapsed = start_time.elapsed().as_millis() as u32;
        let error_message = format!("身份证号 {} 已存在", delivery_staff.id_card);
        
        log_service.log_error(
            log_data
                .with_level(LogLevel::Error)
                .with_message(&format!("创建配送员失败：{}", error_message))
                .with_details(json!(delivery_staff))
                .with_error_code("CONFLICT_ERROR")
                .with_execution_time(elapsed)
        ).await;
        
        return Err(AppError::Conflict(error_message));
    }
    
    // 创建配送员
    match repo.create_delivery_staff(&tenant_hash, &delivery_staff).await {
        Ok(_) => {
            // 记录成功日志
            let elapsed = start_time.elapsed().as_millis() as u32;
            
            log_service.log_info(
                log_data
                    .with_level(LogLevel::Info)
                    .with_message(&format!("创建配送员成功：{}", delivery_staff.name))
                    .with_details(json!(delivery_staff))
                    .with_execution_time(elapsed)
            ).await;
            
            Ok(Json(ApiResponse::new(Some(()), &context)))
        },
        Err(err) => {
            // 记录数据库错误日志
            let elapsed = start_time.elapsed().as_millis() as u32;
            
            log_service.log_error(
                log_data
                    .with_level(LogLevel::Error)
                    .with_message(&format!("创建配送员失败：数据库错误"))
                    .with_details(json!({
                        "error": err.to_string(),
                        "staff_data": delivery_staff
                    }))
                    .with_error_code("DB_ERROR")
                    .with_error_stack(&err.to_string())
                    .with_execution_time(elapsed)
            ).await;
            
            Err(err)
        }
    }
}
```

### 9.3 查询示例

#### 查询特定供应商的配送员操作日志

```sql
-- 查询特定供应商的所有配送员操作日志
SELECT 
    event_time,
    log_level,
    action,
    resource_id,
    message,
    execution_time
FROM system_log
WHERE 
    category = 'staff'
    AND resource_type = 'delivery_staff'
    AND tenant_id = 123
    AND tenant_type = 'PROVIDER'
ORDER BY 
    event_time DESC
LIMIT 100;
```

#### 查询创建失败的配送员操作

```sql
-- 查询配送员创建失败的记录
SELECT 
    event_time,
    user_id,
    ip_address,
    resource_id,
    message,
    error_code,
    details
FROM system_log
WHERE 
    category = 'staff'
    AND resource_type = 'delivery_staff'
    AND action = 'create'
    AND log_level = 'ERROR'
ORDER BY 
    event_time DESC
LIMIT 50;
```

#### 统计特定时间段内各类操作的执行时间

```sql
-- 统计上个月各类操作的平均执行时间
SELECT 
    action,
    COUNT(*) as operation_count,
    AVG(execution_time) as avg_time_ms,
    MAX(execution_time) as max_time_ms
FROM system_log
WHERE 
    category = 'staff'
    AND resource_type = 'delivery_staff'
    AND event_time >= DATE_SUB(CURRENT_DATE(), INTERVAL 1 MONTH)
GROUP BY 
    action
ORDER BY 
    avg_time_ms DESC;
``` 