-- 临时图片URL表
CREATE TABLE IF NOT EXISTS temp_image_urls (
    id INT AUTO_INCREMENT PRIMARY KEY COMMENT '主键ID',
    product_code VARCHAR(10) NOT NULL COMMENT '产品代码',
    bucket_name VARCHAR(100) NOT NULL COMMENT 'OBS桶名',
    object_key VARCHAR(500) NOT NULL COMMENT 'OBS对象键',
    temp_url TEXT NOT NULL COMMENT '临时URL',
    expires_at DATETIME NOT NULL COMMENT '过期时间',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',

    -- 索引
    INDEX idx_product_code (product_code),
    INDEX idx_bucket_name (bucket_name),
    INDEX idx_expires_at (expires_at),
    INDEX idx_product_bucket (product_code, bucket_name),

    -- 唯一约束：同一产品在同一个桶中只能有一个有效URL
    UNIQUE KEY uk_product_bucket (product_code, bucket_name),

    -- 外键约束（如果products表存在）
    CONSTRAINT fk_temp_url_product
        FOREIGN KEY (product_code) REFERENCES products(product_code)
        ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='临时图片URL表';

-- 创建视图：查询有效的临时URL
CREATE OR REPLACE VIEW v_active_temp_urls AS
SELECT
    product_code,
    bucket_name,
    object_key,
    temp_url,
    expires_at,
    TIMESTAMPDIFF(SECOND, NOW(), expires_at) AS remaining_seconds,
    CASE
        WHEN expires_at > NOW() THEN 'ACTIVE'
        ELSE 'EXPIRED'
    END AS status,
    created_at,
    updated_at
FROM temp_image_urls
WHERE expires_at > NOW()
ORDER BY expires_at DESC;