# Database init

```
mysql -u root -p tenant_saas < scripts/database.sql
```

# Authentication & Authorization

This API uses JWT (JSON Web Token) based authentication and authorization. All protected endpoints require a valid JWT token to be included in the request headers.

## Token Format

```json of the token payload
{
    "username": "ryman1981",
    "roles": ["MARKET_ADMIN"],
    "permissions": ["tenant:create"],
    "exp": 1739073963
}
```

### Request Header

The token must be included in the `Authorization` header using the Bearer scheme:

```
Authorization: Bearer <token>
```

### Token Validation

- **Expiration**: Tokens are valid for 1 hour
- **Validation Process**: The `auth_middleware.rs` handles token validation and authorization:
  1. Validates JWT format and expiration
  2. Extracts roles and permissions from payload
  3. Verifies user authorization based on:
     - Required roles/permissions for the endpoint
     - Permissions defined in `permissions.yaml`
     - Request URL path matching

### Error Responses

- **403 Forbidden**: Returned when the user lacks required roles or permissions
- **401 Unauthorized**: Returned when:
  - Token is invalid or expired
  - Username in token not found in database
  - Username doesn't match request URL path

// ... existing code ...
docker tag cloud-blh/market-saas-backend:v0.1 swr.cn-north-4.myhuaweicloud.com/cloud-blh/market-saas-backend:v0.1
docker push swr.cn-north-4.myhuaweicloud.com/cloud-blh/market-saas-backend:v0.1




```sql
SELECT 
    u.username AS username,
    c.id AS assigned_category_id,
    c.name AS assigned_category_name,
    c.level AS assigned_category_level,
    COUNT(DISTINCT p.id) AS total_products,
    COUNT(DISTINCT CASE WHEN pp.id IS NULL THEN p.id END) AS products_without_price
FROM 
    users u
JOIN 
    user_roles ur ON u.id = ur.user_id
JOIN 
    roles r ON ur.role_id = r.id AND r.name = 'PRICER'
JOIN 
    user_category_assignments uca ON u.id = uca.user_id
JOIN 
    categories c ON uca.category_id = c.id
LEFT JOIN 
    categories c_child ON (
        (c.level = 1 AND (c_child.id = c.id OR c_child.parent_id = c.id OR c_child.parent_id IN (SELECT id FROM categories WHERE parent_id = c.id))) OR
        (c.level = 2 AND (c_child.id = c.id OR c_child.parent_id = c.id)) OR
        (c.level = 3 AND c_child.id = c.id)
    )
JOIN 
    products p ON p.category_id = c_child.id AND c_child.level = 3
LEFT JOIN 
    product_prices pp ON p.id = pp.product_id AND pp.status = 'PUBLISHED' AND pp.price_date = CURRENT_DATE
WHERE 
    u.username = 'testpricer'
GROUP BY 
    u.id, u.name, c.id, c.name, c.level
ORDER BY 
    c.level, c.name;


--------------------------
   SELECT 
    p.id AS product_id,
    p.name AS product_name,
    p.unit,
    c.id AS assigned_category_id,
    c.name AS assigned_category_name,
    c.level AS assigned_category_level,
    COALESCE(yesterday_pp.min_price, 0) AS yesterday_min_price,
    COALESCE(yesterday_pp.avg_price, 0) AS yesterday_avg_price,
    COALESCE(yesterday_pp.max_price, 0) AS yesterday_max_price
FROM 
    users u
JOIN 
    user_roles ur ON u.id = ur.user_id
JOIN 
    roles r ON ur.role_id = r.id AND r.name = 'PRICER'
JOIN 
    user_category_assignments uca ON u.id = uca.user_id
JOIN 
    categories c ON uca.category_id = c.id
LEFT JOIN 
    categories c_child ON (
        (c.level = 1 AND (c_child.id = c.id OR c_child.parent_id = c.id OR c_child.parent_id IN (SELECT id FROM categories WHERE parent_id = c.id))) OR
        (c.level = 2 AND (c_child.id = c.id OR c_child.parent_id = c.id)) OR
        (c.level = 3 AND c_child.id = c.id)
    ) AND c_child.level = 3
JOIN 
    products p ON p.category_id = c_child.id
LEFT JOIN 
    product_prices today_pp ON p.id = today_pp.product_id 
    AND today_pp.status = 'PUBLISHED' 
    AND today_pp.price_date = CURRENT_DATE
LEFT JOIN 
    product_prices yesterday_pp ON p.id = yesterday_pp.product_id 
    AND yesterday_pp.status = 'PUBLISHED' 
    AND yesterday_pp.price_date = DATE_SUB(CURRENT_DATE, INTERVAL 1 DAY)
WHERE 
    u.username = 'testpricer' -- 替换为用户名参数
    AND today_pp.id IS NULL -- 只选择今天没有报价的产品
ORDER BY 
    c.level, c.name, p.name;



    --------------------------------
    -- 优化后的查询
-- 修正后的查询
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
    (SELECT status FROM product_prices 
     WHERE product_id = p.id 
     AND price_date = CURRENT_DATE 
     LIMIT 1) AS price_status
FROM 
    users u
JOIN 
    user_roles ur ON u.id = ur.user_id AND ur.role_id = (SELECT id FROM roles WHERE name = 'AUDITOR' LIMIT 1)
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
LEFT JOIN 
    product_prices today_pp ON p.id = today_pp.product_id 
    AND today_pp.price_date = CURRENT_DATE
WHERE 
    u.username = 'blhAuditor' -- 替换为用户名参数
    AND NOT EXISTS (
        SELECT 1 FROM product_prices 
        WHERE product_id = p.id 
        AND status IS NULL 
        AND price_date = CURRENT_DATE
    )
ORDER BY 
    c.level, c.name, p.name;


----- 询价员统计栏SQL
    
    
     SELECT
    c.id AS category_id,
    c.name AS category_name,
  
    COUNT(DISTINCT p.id) AS total_products,
    COUNT(DISTINCT CASE WHEN NOT EXISTS (
        SELECT 1 FROM product_prices pp 
        WHERE pp.product_id = p.id 
        AND pp.price_date = CURRENT_DATE
    ) THEN p.id END) AS products_without_price,
    COUNT(DISTINCT CASE WHEN pp.status = 'PENDING' THEN p.id END) AS products_pending,
    COUNT(DISTINCT CASE WHEN pp.status = 'PUBLISHED' THEN p.id END) AS products_published
FROM 
    users u
JOIN 
    user_roles ur ON u.id = ur.user_id
JOIN 
    roles r ON ur.role_id = r.id AND r.name = 'AUDITOR'
JOIN 
    user_category_assignments uca ON u.id = uca.user_id
JOIN 
    categories c ON uca.category_id = c.id
JOIN (
    -- Para categorias de nível 1, encontre todas as categorias de nível 3 relacionadas
    SELECT c3.id, c1.id AS root_id
    FROM categories c1
    JOIN categories c2 ON c2.parent_id = c1.id AND c2.level = 2
    JOIN categories c3 ON c3.parent_id = c2.id AND c3.level = 3
    WHERE c1.level = 1
    
    UNION ALL
    
    -- Para categorias de nível 2, encontre todas as categorias de nível 3 relacionadas
    SELECT c3.id, c2.id AS root_id
    FROM categories c2
    JOIN categories c3 ON c3.parent_id = c2.id AND c3.level = 3
    WHERE c2.level = 2
    
    UNION ALL
    
    -- Categorias de nível 3 estão diretamente relacionadas a si mesmas
    SELECT c3.id, c3.id AS root_id
    FROM categories c3
    WHERE c3.level = 3
) AS category_mapping ON (
    c.id = category_mapping.root_id
)
JOIN 
    products p ON p.category_id = category_mapping.id
LEFT JOIN 
    product_prices pp ON p.id = pp.product_id 
    AND pp.price_date = CURRENT_DATE
WHERE 
    u.username = 'blhAuditor' -- Substitua pelo nome de usuário desejado
GROUP BY 
    u.username, c.id, c.name, c.level
ORDER BY 
    c.level, c.name;



    SELECT 
    p.id, 
    p.name, 
    c.name as category, 
    pr.avg_price, 
    p.unit, 
    img.temp_url as image 
FROM 
    products p 
    JOIN categories c ON p.category_id = c.id 
    JOIN product_prices pr ON pr.product_id = p.id 
    LEFT JOIN temp_image_urls img ON img.product_code = p.product_code 
WHERE 
    price_date = (CURDATE())
ORDER BY 
    p.id;