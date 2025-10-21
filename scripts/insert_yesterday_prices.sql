-- 插入产品昨天的价格数据脚本
-- 注意：avg_price 字段是计算得出的平均价格

-- 方法1：基于最近的价格数据生成昨天的价格
INSERT INTO product_prices (
    product_id,
    price_date,
    min_price,
    min_price_change,
    max_price,
    max_price_change,
    avg_price,
    avg_price_change,
    market_id,
    status,
    remark,
    source,
    valid_hours,
    created_by
)
SELECT
    p.id AS product_id,
    DATE_SUB(CURDATE(), INTERVAL 1 DAY) AS price_date,
    -- 基于最近价格的90%作为最低价
    ROUND(recent_price.avg_price * 0.9, 2) AS min_price,
    -- 计算最低价变化：当前最低价 - 最近最低价
    ROUND(ROUND(recent_price.avg_price * 0.9, 2) - recent_price.min_price, 2) AS min_price_change,
    -- 基于最近价格的110%作为最高价
    ROUND(recent_price.avg_price * 1.1, 2) AS max_price,
    -- 计算最高价变化：当前最高价 - 最近最高价
    ROUND(ROUND(recent_price.avg_price * 1.1, 2) - recent_price.max_price, 2) AS max_price_change,
    -- 使用最近平均价作为昨天的基准价格
    recent_price.avg_price AS avg_price,
    -- 平均价变化设为0（因为是基准价格）
    0.00 AS avg_price_change,
    recent_price.market_id,
    'PENDING' AS status,
    '基于最近价格自动生成昨日价格' AS remark,
    'SYSTEM' AS source,
    24 AS valid_hours,
    'system' AS created_by
FROM
    products p
    -- 关联最近的价格记录（使用子查询获取每个产品的最新价格）
    JOIN (
        SELECT
            pp1.product_id,
            pp1.market_id,
            pp1.avg_price,
            pp1.min_price,
            pp1.max_price
        FROM product_prices pp1
        WHERE pp1.price_date = (
            SELECT MAX(pp2.price_date)
            FROM product_prices pp2
            WHERE pp2.product_id = pp1.product_id
            AND pp2.market_id = pp1.market_id
        )
    ) AS recent_price ON p.id = recent_price.product_id
WHERE
    p.is_disabled = false -- 只插入未禁用的产品
    AND NOT EXISTS (
        -- 避免重复插入已存在的产品价格数据
        SELECT 1
        FROM product_prices pp
        WHERE pp.product_id = p.id
        AND pp.price_date = DATE_SUB(CURDATE(), INTERVAL 1 DAY)
        AND pp.market_id = recent_price.market_id
    );

-- 方法2：如果没有历史价格数据，基于订单价格生成（作为备用方案）
INSERT INTO product_prices (
    product_id,
    price_date,
    min_price,
    min_price_change,
    max_price,
    max_price_change,
    avg_price,
    avg_price_change,
    market_id,
    status,
    remark,
    source,
    valid_hours,
    created_by
)
SELECT
    p.id AS product_id,
    DATE_SUB(CURDATE(), INTERVAL 1 DAY) AS price_date,
    -- 基于订单平均价格的90%作为最低价
    ROUND(order_avg.avg_order_price * 0.9, 2) AS min_price,
    0.00 AS min_price_change,
    -- 基于订单平均价格的110%作为最高价
    ROUND(order_avg.avg_order_price * 1.1, 2) AS max_price,
    0.00 AS max_price_change,
    -- 使用订单平均价
    order_avg.avg_order_price AS avg_price,
    0.00 AS avg_price_change,
    order_avg.market_id,
    'PENDING' AS status,
    '基于订单价格自动生成昨日价格' AS remark,
    'SYSTEM' AS source,
    24 AS valid_hours,
    'system' AS created_by
FROM
    products p
    -- 关联产品所属市场
    JOIN categories c ON p.category_id = c.id
    JOIN tenants t ON c.id = t.category_id
    -- 关联订单平均价格
    JOIN (
        SELECT
            od.product_code,
            o.market_id,
            AVG(od.actual_price) as avg_order_price,
            COUNT(*) as order_count
        FROM order_details od
        JOIN orders o ON od.order_id = o.id
        WHERE o.order_date >= DATE_SUB(CURDATE(), INTERVAL 30 DAY) -- 最近30天的订单
        GROUP BY od.product_code, o.market_id
        HAVING order_count >= 3 -- 至少有3个订单记录
    ) order_avg ON p.product_code = order_avg.product_code AND t.id = order_avg.market_id
WHERE
    p.is_disabled = false
    AND NOT EXISTS (
        -- 避免重复插入已存在的产品价格数据
        SELECT 1
        FROM product_prices pp
        WHERE pp.product_id = p.id
        AND pp.price_date = DATE_SUB(CURDATE(), INTERVAL 1 DAY)
        AND pp.market_id = order_avg.market_id
    )
    AND NOT EXISTS (
        -- 避免与方法1重复插入
        SELECT 1
        FROM product_prices pp2
        WHERE pp2.product_id = p.id
        AND pp2.price_date = DATE_SUB(CURDATE(), INTERVAL 1 DAY)
        AND pp2.remark LIKE '%基于最近价格%'
    );

-- 方法3：如果以上都没有，使用默认价格（最后备用方案）
INSERT INTO product_prices (
    product_id,
    price_date,
    min_price,
    min_price_change,
    max_price,
    max_price_change,
    avg_price,
    avg_price_change,
    market_id,
    status,
    remark,
    source,
    valid_hours,
    created_by
)
SELECT
    p.id AS product_id,
    DATE_SUB(CURDATE(), INTERVAL 1 DAY) AS price_date,
    10.00 AS min_price, -- 默认最低价
    0.00 AS min_price_change,
    20.00 AS max_price, -- 默认最高价
    0.00 AS max_price_change,
    15.00 AS avg_price, -- 默认平均价 = (10+20)/2
    0.00 AS avg_price_change,
    t.id AS market_id,
    'PENDING' AS status,
    '使用默认价格生成昨日价格' AS remark,
    'SYSTEM' AS source,
    24 AS valid_hours,
    'system' AS created_by
FROM
    products p
    -- 关联产品所属市场
    CROSS JOIN tenants t
WHERE
    t.tenant_type = 'MARKET' -- 只选择市场类型的租户
    AND p.is_disabled = false
    AND NOT EXISTS (
        -- 避免重复插入已存在的产品价格数据
        SELECT 1
        FROM product_prices pp
        WHERE pp.product_id = p.id
        AND pp.price_date = DATE_SUB(CURDATE(), INTERVAL 1 DAY)
        AND pp.market_id = t.id
    );

-- 如果需要根据前一天的实际价格数据来计算变化值，可以使用以下更复杂的版本：

/*
INSERT INTO product_prices (
    product_id,
    price_date,
    min_price,
    min_price_change,
    max_price,
    max_price_change,
    avg_price,
    avg_price_change,
    market_id,
    status,
    remark,
    source,
    valid_hours,
    created_by
)
SELECT
    p.id AS product_id,
    DATE_SUB(CURDATE(), INTERVAL 1 DAY) AS price_date,
    -- 使用产品价格的70%作为当天的最低价
    ROUND(p.price * 0.7, 2) AS min_price,
    -- 计算最低价变化：当前最低价 - 前一天最低价
    CASE
        WHEN prev_price.min_price IS NOT NULL
        THEN ROUND(ROUND(p.price * 0.7, 2) - prev_price.min_price, 2)
        ELSE 0.00
    END AS min_price_change,
    -- 使用产品价格的130%作为当天的最高价
    ROUND(p.price * 1.3, 2) AS max_price,
    -- 计算最高价变化：当前最高价 - 前一天最高价
    CASE
        WHEN prev_price.max_price IS NOT NULL
        THEN ROUND(ROUND(p.price * 1.3, 2) - prev_price.max_price, 2)
        ELSE 0.00
    END AS max_price_change,
    -- 计算平均价：(最低价 + 最高价) / 2
    ROUND((ROUND(p.price * 0.7, 2) + ROUND(p.price * 1.3, 2)) / 2, 2) AS avg_price,
    -- 计算平均价变化：当前平均价 - 前一天平均价
    CASE
        WHEN prev_price.avg_price IS NOT NULL
        THEN ROUND(ROUND((ROUND(p.price * 0.7, 2) + ROUND(p.price * 1.3, 2)) / 2, 2) - prev_price.avg_price, 2)
        ELSE 0.00
    END AS avg_price_change,
    p.market_id,
    'PENDING' AS status,
    '系统自动生成昨日价格' AS remark,
    'SYSTEM' AS source,
    24 AS valid_hours,
    'system' AS created_by
FROM
    products p
    -- 关联前一天的价格数据用于计算变化值
    LEFT JOIN product_prices prev_price ON p.id = prev_price.product_id
        AND prev_price.price_date = DATE_SUB(CURDATE(), INTERVAL 2 DAY)
        AND prev_price.market_id = p.market_id
WHERE
    p.status = 'ACTIVE'
    AND NOT EXISTS (
        SELECT 1
        FROM product_prices pp
        WHERE pp.product_id = p.id
        AND pp.price_date = DATE_SUB(CURDATE(), INTERVAL 1 DAY)
        AND pp.market_id = p.market_id
    );
*/