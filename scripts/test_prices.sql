-- 插入昨天的数据
INSERT INTO product_prices (
    product_id,
    price_date,
    min_price,
    max_price,
    avg_price,
    min_price_change,
    max_price_change,
    avg_price_change,
    market_id,
    status,
    source,
    created_by
)
SELECT 
    p.id,
    DATE_SUB(CURRENT_DATE, INTERVAL 1 DAY),
    @min_price := ROUND(10 + RAND() * 5, 2) as min_price,
    @max_price := ROUND(@min_price + RAND() * 10, 2) as max_price,
    ROUND((@min_price + @max_price) / 2, 2) as avg_price,
    0 as min_price_change,
    0 as max_price_change,
    0 as avg_price_change,
    1,
    'PUBLISHED',
    'SYSTEM',
    'system'
FROM products p
WHERE p.is_disabled = 0;

---插入或更新昨日数据

INSERT INTO product_prices (
    product_id,
    price_date,
    min_price,
    max_price,
    avg_price,
    min_price_change,
    max_price_change,
    avg_price_change,
    market_id,
    status,
    source,
    created_by
)
SELECT 
    p.id,
    DATE_SUB(CURRENT_DATE, INTERVAL 1 DAY),
    @min_price := ROUND(RAND() * 5, 2) as min_price,
    @max_price := ROUND(@min_price + RAND() * 2, 2) as max_price,
    ROUND((@min_price + @max_price) / 2, 2) as avg_price,
    0 as min_price_change,
    0 as max_price_change,
    0 as avg_price_change,
    1 as market_id,  -- 假设这是您的 market_id
    'PUBLISHED' as status,
    'SYSTEM' as source,
    'system' as created_by
FROM products p
ON DUPLICATE KEY UPDATE
    min_price = VALUES(min_price),
    max_price = VALUES(max_price),
    avg_price = VALUES(avg_price),
    min_price_change = VALUES(min_price_change),
    max_price_change = VALUES(max_price_change),
    avg_price_change = VALUES(avg_price_change),
    status = VALUES(status),
    source = VALUES(source),
    updated_at = NOW();

-- 插入今天的数据（基于昨天的价格计算涨跌）
INSERT INTO product_prices (
    product_id,
    price_date,
    min_price,
    max_price,
    avg_price,
    min_price_change,
    max_price_change,
    avg_price_change,
    market_id,
    status,
    source,
    created_by
)
SELECT 
    p.id,
    CURRENT_DATE,
    @today_min := ROUND(pp.min_price * (1 + (RAND() * 0.2 - 0.1)), 2) as min_price,
    @today_max := ROUND(pp.max_price * (1 + (RAND() * 0.2 - 0.1)), 2) as max_price,
    @today_avg := ROUND((@today_min + @today_max) / 2, 2) as avg_price,
    ROUND(@today_min - pp.min_price, 2) as min_price_change,
    ROUND(@today_max - pp.max_price, 2) as max_price_change,
    ROUND(@today_avg - pp.avg_price, 2) as avg_price_change,
    1,
    'PUBLISHED',
    'SYSTEM',
    'system'
FROM products p
JOIN product_prices pp ON p.id = pp.product_id 
    AND pp.price_date = DATE_SUB(CURRENT_DATE, INTERVAL 1 DAY)
WHERE p.is_disabled = 0
ON DUPLICATE KEY UPDATE
    min_price = VALUES(min_price),
    max_price = VALUES(max_price),
    avg_price = VALUES(avg_price),
    min_price_change = VALUES(min_price_change),
    max_price_change = VALUES(max_price_change),
    avg_price_change = VALUES(avg_price_change),
    status = VALUES(status),
    source = VALUES(source),
    updated_at = CURRENT_TIMESTAMP;



WITH customer_orders AS (
                        SELECT o.id as order_id
                        FROM orders o
                        JOIN tenants t ON o.customer_id = t.id
                        WHERE t.name_hash = 'b9m3qvnc'
                        AND t.tenant_type = 'CUSTOMER'
                        AND t.deleted_at IS NULL
                    )
                    SELECT 
                        o.order_code,
                        o.order_status, 
                        COALESCE(MIN(osh.created_at), o.created_at) as accepted_at
                    FROM orders o
                    JOIN customer_orders co ON o.id = co.order_id
                    LEFT JOIN order_status_history osh ON o.id = osh.order_id
                    WHERE (o.order_status = 'AFTER_SALE' OR o.order_status = 'ACCEPTED')
                    AND (osh.to_status = 'AFTER_SALE' OR osh.to_status = 'ACCEPTED')
                    AND o.deleted_at IS NULL
                    GROUP BY o.order_code, o.order_status, o.created_at
                    ORDER BY accepted_at DESC;


-- 初始化所有订单相关数据

SET FOREIGN_KEY_CHECKS = 0;
truncate table order_status_history;
truncate table exchange_items;
truncate table return_exchange_records;
truncate table refund_records;
truncate table provider_orders_assignments;
truncate table order_details;
truncate table orders;
truncate table reconciliation_statements;
truncate table reconciliation_statement_details;
truncate table reconciliation_statement_orders;

SET FOREIGN_KEY_CHECKS = 1;


