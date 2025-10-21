#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
调试OBS URL生成问题
测试不同的过期时间格式
"""

from dotenv import load_dotenv
load_dotenv()
import os
from datetime import datetime, timedelta, timezone
from obs import ObsClient
import time

def debug_obs_url():
    """调试OBS URL生成问题"""

    # 获取配置
    access_key = os.getenv('OBS_ACCESS_KEY')
    secret_key = os.getenv('OBS_SECRET_KEY')
    endpoint = os.getenv('OBS_ENDPOINT')
    bucket_name = os.getenv('OBS_BUCKET_NAME')

    print(f"调试OBS URL生成...")
    print(f"Bucket: {bucket_name}")

    # 创建OBS客户端
    try:
        client = ObsClient(
            access_key_id=access_key,
            secret_access_key=secret_key,
            server=endpoint
        )
        print("✓ OBS客户端创建成功")
    except Exception as e:
        print(f"✗ OBS客户端创建失败: {e}")
        return

    test_object = "products/100100101.webp"

    # 测试不同的过期时间格式
    test_cases = [
        {
            "name": "当前时间戳 + 3600秒（原始方法）",
            "expires": int((datetime.now() + timedelta(seconds=3600)).timestamp()),
            "type": "timestamp"
        },
        {
            "name": "UTC时间戳 + 3600秒",
            "expires": int((datetime.now(timezone.utc) + timedelta(seconds=3600)).timestamp()),
            "type": "utc_timestamp"
        },
        {
            "name": "Unix时间戳（当前时间 + 1小时）",
            "expires": int(time.time()) + 3600,
            "type": "unix_timestamp"
        },
        {
            "name": "datetime对象（UTC）",
            "expires": datetime.now(timezone.utc) + timedelta(hours=1),
            "type": "datetime_utc"
        },
        {
            "name": "简单秒数（3600）",
            "expires": 3600,
            "type": "seconds"
        },
        {
            "name": "较大时间戳（24小时后）",
            "expires": int(time.time()) + 86400,
            "type": "24h_timestamp"
        },
        {
            "name": "较小时间戳（10分钟后）",
            "expires": int(time.time()) + 600,
            "type": "10m_timestamp"
        }
    ]

    for i, test_case in enumerate(test_cases, 1):
        print(f"\n--- 测试 {i}: {test_case['name']} ---")
        print(f"过期值: {test_case['expires']} (类型: {type(test_case['expires'])})")

        try:
            resp = client.createSignedUrl(
                method='GET',
                bucketName=bucket_name,
                objectKey=test_object,
                expires=test_case['expires']
            )

            print(f"响应类型: {type(resp)}")
            print(f"响应状态: {resp.status if hasattr(resp, 'status') else 'N/A'}")

            if hasattr(resp, 'signedUrl') and resp.signedUrl:
                print(f"✓ URL生成成功")
                print(f"  URL长度: {len(resp.signedUrl)}")
                print(f"  URL前100字符: {resp.signedUrl[:100]}...")

                # 分析URL中的Expires参数
                if 'Expires=' in resp.signedUrl:
                    url_expires = resp.signedUrl.split('Expires=')[1].split('&')[0]
                    print(f"  URL中的Expires值: {url_expires}")
                    print(f"  Expires值类型: {type(url_expires)}")

                    # 验证Expires值是否合理
                    try:
                        expires_int = int(url_expires)
                        current_time = int(time.time())
                        time_diff = expires_int - current_time
                        print(f"  当前时间戳: {current_time}")
                        print(f"  过期时间戳: {expires_int}")
                        print(f"  时间差（秒）: {time_diff}")
                        print(f"  时间差（小时）: {time_diff / 3600:.2f}")

                        if time_diff > 0:
                            print(f"  ✓ URL未过期")
                        else:
                            print(f"  ✗ URL已过期")
                    except ValueError:
                        print(f"  ✗ Expires值不是有效整数")

                return resp.signedUrl
            elif isinstance(resp, dict) and 'signedUrl' in resp:
                print(f"✓ URL生成成功（字典格式）")
                url = resp['signedUrl']
                print(f"  URL长度: {len(url)}")
                print(f"  URL前100字符: {url[:100]}...")
                return url
            else:
                error_msg = getattr(resp, 'errorMessage', '未知错误')
                print(f"✗ URL生成失败: {error_msg}")

        except Exception as e:
            print(f"✗ 测试失败: {e}")

    # 测试手动构建URL
    print(f"\n--- 手动构建预签名URL测试 ---")
    try:
        from urllib.parse import quote
        import hmac
        import hashlib
        import base64

        # 华为云OBS签名算法测试
        method = 'GET'
        host = f"{bucket_name}.{endpoint.replace('https://', '').replace('http://', '')}"
        object_key = test_object
        expires = int(time.time()) + 3600

        # 构建待签名字符串
        canonical_request = f"{method}\n\n\n{expires}\n/{object_key}"

        print(f"规范请求字符串:\n{canonical_request}")

        # 这里需要正确的签名算法，需要查看华为云OBS文档
        print("手动构建URL需要详细的华为云OBS签名算法文档")

    except Exception as e:
        print(f"手动构建测试失败: {e}")

if __name__ == '__main__':
    debug_obs_url()