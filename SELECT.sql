SELECT 
p.id AS product_id,
p.name AS product_name,
p.unit,
c.name AS assigned_category_name,
COALESCE(yesterday_pp.min_price, 0) AS yesterday_min_price,
COALESCE(yesterday_pp.avg_price, 0) AS yesterday_avg_price,
COALESCE(yesterday_pp.max_price, 0) AS yesterday_max_price,
COALESCE(today_pp.min_price, 0) AS today_min_price,
COALESCE(today_pp.avg_price, 0) AS today_avg_price,
COALESCE(today_pp.max_price, 0) AS today_max_price,
today_pp.status AS price_status,
today_pp.price_source AS price_source,
today_pp.price_date AS publish_date
FROM 
users u
JOIN 
user_roles ur ON u.id = ur.user_id AND ur.role_id = (SELECT id FROM roles WHERE name = 'PRICER' LIMIT 1)
JOIN 
user_category_assignments uca ON u.id = uca.user_id
JOIN 
categories c ON uca.category_id = c.id
JOIN (
    -- 为一级分类找到所有关联的三级分类
    SELECT c3.id, c1.id AS root_id
    FROM categories c1
    JOIN categories c2 ON c2.parent_id = c1.id AND c2.level = 2
    JOIN categories c3 ON c3.parent_id = c2.id AND c3.level = 3
    WHERE c1.level = 1
    
    UNION ALL
    
    -- 为二级分类找到所有关联的三级分类
    SELECT c3.id, c2.id AS root_id
    FROM categories c2
    JOIN categories c3 ON c3.parent_id = c2.id AND c3.level = 3
    WHERE c2.level = 2
    
    UNION ALL
    
    -- 三级分类直接关联自己
    SELECT c3.id, c3.id AS root_id
    FROM categories c3
    WHERE c3.level = 3
) AS category_mapping ON (
    c.id = category_mapping.root_id
)
JOIN 
products p ON p.category_id = category_mapping.id
LEFT JOIN 
product_prices yesterday_pp ON p.id = yesterday_pp.product_id 
AND yesterday_pp.status = 'PUBLISHED' 
AND yesterday_pp.price_date = DATE_SUB(CURRENT_DATE, INTERVAL 1 DAY)
LEFT JOIN (
    -- 获取当天或最近日期的价格
    SELECT pp1.*,
    CASE WHEN pp1.price_date = CURRENT_DATE THEN 'TODAY' ELSE 'HISTORY' END AS price_source
    FROM product_prices pp1
    JOIN (
        SELECT product_id, MAX(price_date) as latest_date
        FROM product_prices
        WHERE status = 'PENDING' 
        AND price_date <= CURRENT_DATE
        GROUP BY product_id
    ) latest ON pp1.product_id = latest.product_id AND pp1.price_date = latest.latest_date
    WHERE pp1.status = 'PENDING'
) AS today_pp ON p.id = today_pp.product_id
WHERE 
u.username ='Malonglong'  
-- AND NOT EXISTS (
--     SELECT 1 FROM product_prices 
--     WHERE product_id = p.id 
--     AND price_date = CURRENT_DATE AND status = 'PENDING');
    AND EXISTS (
        SELECT 1 FROM product_prices 
        WHERE product_id = p.id 
        AND price_date = CURRENT_DATE
        AND status = 'PENDING');