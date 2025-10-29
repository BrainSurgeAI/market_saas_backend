#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
华为云OBS临时URL管理脚本
用于生成产品图片的临时URL并存储到数据库中
"""

import os
import sys
import logging
from datetime import datetime, timedelta
from typing import List, Dict, Optional
import mysql.connector
from obs import ObsClient
from dotenv import load_dotenv

# 加载环境变量
load_dotenv()

# 配置日志
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler('obs_manager.log'),
        logging.StreamHandler(sys.stdout)
    ]
)
logger = logging.getLogger(__name__)

class OBSManager:
    """华为云OBS管理器"""

    def __init__(self, config: Dict):
        """
        初始化OBS管理器

        Args:
            config: 配置字典，包含OBS和数据库连接信息
        """
        self.config = config

        # 初始化OBS客户端
        self.obs_client = ObsClient(
            access_key_id=config['obs']['access_key'],
            secret_access_key=config['obs']['secret_key'],
            server=config['obs']['endpoint']
        )

        # 数据库连接配置
        self.db_config = config['database']

        logger.info("OBS管理器初始化完成")

    def get_database_connection(self):
        """获取数据库连接"""
        try:
            conn = mysql.connector.connect(**self.db_config)
            return conn
        except mysql.connector.Error as e:
            logger.error(f"数据库连接失败: {e}")
            raise

    def get_products_from_database(self) -> List[str]:
        """
        从数据库获取所有产品代码

        Returns:
            产品代码列表
        """
        product_codes = []

        try:
            conn = self.get_database_connection()
            cursor = conn.cursor()

            query = "SELECT product_code FROM products WHERE is_disabled = FALSE"
            cursor.execute(query)

            product_codes = [row[0] for row in cursor.fetchall()]

            cursor.close()
            conn.close()

            logger.info(f"从数据库获取到 {len(product_codes)} 个产品代码")

        except mysql.connector.Error as e:
            logger.error(f"查询产品代码失败: {e}")
            raise

        return product_codes

    def check_obs_object_exists(self, bucket_name: str, object_key: str) -> bool:
        """
        检查OBS对象是否存在

        Args:
            bucket_name: OBS桶名
            object_key: 对象键

        Returns:
            对象是否存在
        """
        try:
            resp = self.obs_client.getObjectMetadata(bucket_name, object_key)
            return resp.status < 300
        except Exception as e:
            # 检查是否是404错误（对象不存在）
            if hasattr(e, 'status') and e.status == 404:
                logger.warning(f"OBS对象不存在: {object_key}")
                return False
            else:
                logger.error(f"检查OBS对象失败 {object_key}: {e}")
                return False

    def generate_temp_url(self, bucket_name: str, object_key: str, expires_in: int = 7200) -> Optional[str]:
        """
        生成OBS对象的临时URL

        Args:
            bucket_name: OBS桶名
            object_key: 对象键
            expires_in: 过期时间（秒），默认1小时

        Returns:
            临时URL字符串
        """
        try:
            # 生成临时URL，使用相对时间（秒数）
            resp = self.obs_client.createSignedUrl(
                method='GET',
                bucketName=bucket_name,
                objectKey=object_key,
                expires=expires_in
            )

            # 检查响应是否包含signedUrl
            if isinstance(resp, dict) and 'signedUrl' in resp:
                temp_url = resp['signedUrl']
                logger.debug(f"生成临时URL成功: {object_key} -> {temp_url[:100]}...")
                return temp_url
            elif hasattr(resp, 'signedUrl') and resp.signedUrl:
                temp_url = resp.signedUrl
                logger.debug(f"生成临时URL成功: {object_key} -> {temp_url[:100]}...")
                return temp_url
            else:
                logger.error(f"生成临时URL失败 {object_key}: 响应中没有signedUrl")
                return None

        except Exception as e:
            logger.error(f"生成临时URL异常 {object_key}: {e}")
            return None

    def clear_temp_urls(self, bucket_name: str):
        """
        清空临时URL表

        Args:
            bucket_name: OBS桶名
        """
        try:
            conn = self.get_database_connection()
            cursor = conn.cursor()

            # 只清空当前桶的URL，保留其他桶的URL
            delete_query = "DELETE FROM temp_image_urls WHERE bucket_name = %s"
            cursor.execute(delete_query, (bucket_name,))

            affected_rows = cursor.rowcount
            conn.commit()

            cursor.close()
            conn.close()

            logger.info(f"清空临时URL表，删除了 {affected_rows} 条记录")

        except mysql.connector.Error as e:
            logger.error(f"清空临时URL表失败: {e}")
            raise

    def save_temp_url_to_database(self, bucket_name: str, product_code: str,
                                 object_key: str, temp_url: str, expires_in: int = 3600):
        """
        保存临时URL到数据库

        Args:
            bucket_name: OBS桶名
            product_code: 产品代码
            object_key: 对象键
            temp_url: 临时URL
            expires_in: 过期时间（秒）
        """
        try:
            conn = self.get_database_connection()
            cursor = conn.cursor()

            # 计算过期时间
            expires_at = datetime.now() + timedelta(seconds=expires_in)

            # 插入或更新临时URL记录
            upsert_query = """
            INSERT INTO temp_image_urls
            (product_code, bucket_name, object_key, temp_url, expires_at, created_at, updated_at)
            VALUES (%s, %s, %s, %s, %s, NOW(), NOW())
            ON DUPLICATE KEY UPDATE
            temp_url = VALUES(temp_url),
            expires_at = VALUES(expires_at),
            updated_at = NOW()
            """

            cursor.execute(upsert_query, (
                product_code, bucket_name, object_key, temp_url, expires_at
            ))

            conn.commit()
            cursor.close()
            conn.close()

            logger.info(f"保存临时URL成功: {product_code} -> {object_key}")

        except mysql.connector.Error as e:
            logger.error(f"保存临时URL失败 {product_code}: {e}")
            raise

    def process_product_images(self, bucket_name: str, prefix: str = "products/",
                             expires_in: int = 3600, clear_existing: bool = True):
        """
        处理产品图片，生成临时URL

        Args:
            bucket_name: OBS桶名
            prefix: 对象前缀，默认为"products/"
            expires_in: 过期时间（秒），默认1小时
            clear_existing: 是否清空现有记录，默认True
        """
        logger.info(f"开始处理产品图片，桶名: {bucket_name}, 前缀: {prefix}")

        # 清空现有临时URL
        if clear_existing:
            self.clear_temp_urls(bucket_name)

        # 获取所有产品代码
        product_codes = self.get_products_from_database()

        if not product_codes:
            logger.warning("没有找到产品代码")
            return

        success_count = 0
        failure_count = 0

        for product_code in product_codes:
            try:
                # 构建对象键
                object_key = f"{prefix}{product_code}.webp"

                # 检查OBS对象是否存在
                if not self.check_obs_object_exists(bucket_name, object_key):
                    logger.warning(f"OBS对象不存在，跳过: {object_key}")
                    failure_count += 1
                    continue

                # 生成临时URL
                temp_url = self.generate_temp_url(bucket_name, object_key, expires_in)

                if temp_url:
                    # 保存到数据库
                    self.save_temp_url_to_database(
                        bucket_name, product_code, object_key, temp_url, expires_in
                    )
                    success_count += 1
                else:
                    failure_count += 1

            except Exception as e:
                logger.error(f"处理产品图片失败 {product_code}: {e}")
                failure_count += 1

        logger.info(f"产品图片处理完成! 成功: {success_count}, 失败: {failure_count}")

    def refresh_expired_urls(self, bucket_name: str, prefix: str = "products/",
                           expires_in: int = 3600):
        """
        刷新过期的临时URL

        Args:
            bucket_name: OBS桶名
            prefix: 对象前缀
            expires_in: 新的过期时间（秒）
        """
        logger.info(f"开始刷新过期的临时URL，桶名: {bucket_name}")

        try:
            conn = self.get_database_connection()
            cursor = conn.cursor()

            # 查找过期的记录
            query = """
            SELECT product_code, object_key FROM temp_image_urls
            WHERE bucket_name = %s AND expires_at < NOW()
            """
            cursor.execute(query, (bucket_name,))
            expired_records = cursor.fetchall()

            cursor.close()
            conn.close()

            if not expired_records:
                logger.info("没有找到过期的临时URL")
                return

            logger.info(f"找到 {len(expired_records)} 个过期的临时URL")

            success_count = 0
            failure_count = 0

            for product_code, object_key in expired_records:
                try:
                    # 生成新的临时URL
                    temp_url = self.generate_temp_url(bucket_name, object_key, expires_in)

                    if temp_url:
                        # 更新数据库
                        self.save_temp_url_to_database(
                            bucket_name, product_code, object_key, temp_url, expires_in
                        )
                        success_count += 1
                    else:
                        failure_count += 1

                except Exception as e:
                    logger.error(f"刷新临时URL失败 {product_code}: {e}")
                    failure_count += 1

            logger.info(f"过期URL刷新完成! 成功: {success_count}, 失败: {failure_count}")

        except mysql.connector.Error as e:
            logger.error(f"刷新过期URL失败: {e}")
            raise

    def cleanup_expired_urls(self, bucket_name: str):
        """
        清理过期的临时URL记录

        Args:
            bucket_name: OBS桶名
        """
        try:
            conn = self.get_database_connection()
            cursor = conn.cursor()

            # 删除过期记录
            query = "DELETE FROM temp_image_urls WHERE bucket_name = %s AND expires_at < NOW()"
            cursor.execute(query, (bucket_name,))

            affected_rows = cursor.rowcount
            conn.commit()

            cursor.close()
            conn.close()

            logger.info(f"清理了 {affected_rows} 条过期的临时URL记录")

        except mysql.connector.Error as e:
            logger.error(f"清理过期URL失败: {e}")
            raise


def main():
    """主函数"""
    # 配置信息
    config = {
        'obs': {
            'access_key': os.getenv('OBS_ACCESS_KEY', 'your_access_key'),
            'secret_key': os.getenv('OBS_SECRET_KEY', 'your_secret_key'),
            'endpoint': os.getenv('OBS_ENDPOINT', 'https://obs.cn-north-1.myhuaweicloud.com')
        },
        'database': {
            'host': os.getenv('DB_HOST', 'localhost'),
            'port': int(os.getenv('DB_PORT', 3306)),
            'user': os.getenv('DB_USER', 'root'),
            'password': os.getenv('DB_PASSWORD', 'password'),
            'database': os.getenv('DB_NAME', 'market_saas'),
            'charset': 'utf8mb4',
            'autocommit': True
        }
    }

    # 创建OBS管理器
    manager = OBSManager(config)

    # OBS配置
    bucket_name = os.getenv('OBS_BUCKET_NAME', 'your-bucket-name')
    prefix = 'products/'
    expires_in = int(os.getenv('URL_EXPIRES_IN', 3600))  # 1小时

    # 解析命令行参数
    if len(sys.argv) > 1:
        command = sys.argv[1].lower()

        if command == 'refresh':
            # 刷新过期的URL
            manager.refresh_expired_urls(bucket_name, prefix, expires_in)
        elif command == 'cleanup':
            # 清理过期的URL
            manager.cleanup_expired_urls(bucket_name)
        elif command == 'rebuild':
            # 重新生成所有URL
            clear_existing = len(sys.argv) > 2 and sys.argv[2].lower() == 'clear'
            manager.process_product_images(bucket_name, prefix, expires_in, clear_existing)
        else:
            print("可用命令:")
            print("  refresh  - 刷新过期的临时URL")
            print("  cleanup  - 清理过期的URL记录")
            print("  rebuild  [clear] - 重新生成所有URL (加clear参数会先清空现有记录)")
    else:
        # 默认：处理所有产品图片
        manager.process_product_images(bucket_name, prefix, expires_in)


if __name__ == '__main__':
    main()