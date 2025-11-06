# 订单 API 文档

本文档描述了订单相关的所有 API 接口，包括 HTTP 方法、URL、请求体格式和响应格式。

## 通用响应格式

所有 API 接口都使用统一的响应格式：

```json
{
  "code": 200,
  "message": "success",
  "data": {},
  "requestId": "string"
}
```

- `code`: HTTP 状态码
- `message`: 响应消息
- `data`: 响应数据（根据接口不同而变化）
- `requestId`: 请求 ID

---

## 1. 创建订单

**HTTP 方法:** `POST`

**URL:** `/api/v1/orders`

**请求体:**

```json
{
  "totalAmount": "200.00",
  "deliveryInfo": {
    "deliveryDate": "2024-01-15",
    "address": "北京市朝阳区xxx街道xxx号",
    "contactName": "张三",
    "contactPhone": "13800138000"
  },
  "items": [
    {
      "productId": "P001",
      "name": "测试产品",
      "categoryId": 1,
      "category": "测试分类",
      "unit": "个",
      "quantity": "2.00",
      "price": "100.00",
      "originalPrice": "120.00",
      "originalTotal": "240.00",
      "discountRate": "0.80",
      "total": "200.00",
      "customNote": "备注信息（可选）",
      "processingServices": [
        {
          "type": "加工类型",
          "description": "加工描述（可选）"
        }
      ]
    }
  ]
}
```

**响应:**

```json
{
  "code": 200,
  "message": "success",
  "data": {
    "orderCode": "ORD20240115001",
    "totalAmount": "200.00",
    "actualAmount": "200.00",
    "deliveryDate": "2024-01-15",
    "deliveryAddress": "北京市朝阳区xxx街道xxx号",
    "orderStatus": "PENDING",
    "createdAt": "2024-01-14T10:30:00Z",
    "afterSaleAt": null
  },
  "requestId": "xxx"
}
```

---

## 2. 获取订单列表

**HTTP 方法:** `GET`

**URL:** `/api/v1/orders`

**查询参数:**

- `page` (可选): 页码，默认为 1
- `pageSize` (可选): 每页数量，默认为 20
- `orderStatus` (可选): 订单状态过滤，如 "PENDING", "CONFIRMED", "PROCESSING", "STOCKED", "AFTER_SALE", "REJECTED", "COMPLETED"

**示例:** `/api/v1/orders?page=1&pageSize=20&orderStatus=PENDING`

**响应:**

```json
{
  "code": 200,
  "message": "success",
  "data": [
    {
      "orderCode": "ORD20240115001",
      "totalAmount": "200.00",
      "actualAmount": "200.00",
      "deliveryDate": "2024-01-15",
      "deliveryAddress": "北京市朝阳区xxx街道xxx号",
      "orderStatus": "PENDING",
      "createdAt": "2024-01-14T10:30:00Z",
      "afterSaleAt": null
    }
  ],
  "requestId": "xxx"
}
```

---

## 3. 根据订单号获取订单详情

**HTTP 方法:** `GET`

**URL:** `/api/v1/orders/{order_code}`

**路径参数:**

- `order_code`: 订单号

**响应:**

```json
{
  "code": 200,
  "message": "success",
  "data": {
    "order": {
      "id": 1,
      "orderCode": "ORD20240115001",
      "customerName": "客户名称",
      "orderStatus": "PENDING",
      "totalAmount": "200.00",
      "discountAmount": "0.00",
      "actualAmount": "200.00",
      "deliveryDate": "2024-01-15",
      "deliveryAddress": "北京市朝阳区xxx街道xxx号",
      "contactName": "张三",
      "contactPhone": "13800138000",
      "remark": null,
      "createdBy": "创建人",
      "createdAt": "2024-01-14T10:30:00Z",
      "confirmBy": null,
      "confirmedAt": null,
      "processedBy": null,
      "processedAt": null,
      "stockedBy": null,
      "stockedAt": null,
      "afterSaleAt": null,
      "rejectBy": null,
      "rejectedAt": null,
      "rejectReason": null,
      "completedBy": null,
      "deliveryStaffName": null,
      "deliveryStaffPhone": null,
      "providerName": null,
      "completedAt": null
    },
    "items": [
      {
        "id": 1,
        "productId": "P001",
        "name": "测试产品",
        "categoryId": 1,
        "category": "测试分类",
        "unit": "个",
        "quantity": "2.00",
        "price": "120.00",
        "discountRate": "0.80",
        "actualPrice": "100.00",
        "actualQuantity": null,
        "actualAmount": null,
        "total": "200.00",
        "processingRequirements": null,
        "remark": null,
        "status": null
      }
    ],
    "receipts": [
      {
        "id": 1,
        "orderId": "ORD20240115001",
        "productId": "P001",
        "productName": "测试产品",
        "operationType": "SIGN",
        "quantity": "2.00",
        "reason": "签收原因",
        "unit": "个",
        "evidenceImages": null
      }
    ]
  },
  "requestId": "xxx"
}
```

---

## 4. 分配订单给供应商

**HTTP 方法:** `PATCH`

**URL:** `/api/v1/orders/{order_code}/assign`

**路径参数:**

- `order_code`: 订单号

**请求体:**

```json
{
  "providerId": 1,
  "confirmedBy": "确认人姓名"
}
```

**响应:**

```json
{
  "code": 200,
  "message": "success",
  "data": null,
  "requestId": "xxx"
}
```

---

## 5. 供应商备货

**HTTP 方法:** `PATCH`

**URL:** `/api/v1/orders/{order_code}/preparing`

**路径参数:**

- `order_code`: 订单号

**请求体:**

```json
{
  "idCard": "110101199001011234"
}
```

**响应:**

```json
{
  "code": 200,
  "message": "success",
  "data": "PROCESSING",
  "requestId": "xxx"
}
```

---

## 6. 配送到市场

**HTTP 方法:** `PUT`

**URL:** `/api/v1/orders/{order_code}/deliver-to-market`

**路径参数:**

- `order_code`: 订单号

**请求体:**

```json
{
  "stockedBy": "入库操作人",
  "items": [
    {
      "id": 1,
      "actualQuantity": "2.00"
    }
  ]
}
```

**响应:**

```json
{
  "code": 200,
  "message": "success",
  "data": null,
  "requestId": "xxx"
}
```

---

## 7. 验收子订单

**HTTP 方法:** `PATCH`

**URL:** `/api/v1/orders/{order_code}/inspect-sub-orders`

**路径参数:**

- `order_code`: 订单号

**请求体:**

```json
{
  "operateBy": "操作人",
  "receipt": {
    "id": 1,
    "orderId": "ORD20240115001",
    "productId": "P001",
    "productName": "测试产品",
    "operationType": "SIGN",
    "quantity": "2.00",
    "reason": "签收原因",
    "unit": "个",
    "evidenceImages": "image1.jpg,image2.jpg"
  }
}
```

**operationType 可选值:**
- `SIGN`: 签收
- `RETURN`: 退货
- `EXCHANGE`: 换货

**响应:**

```json
{
  "code": 200,
  "message": "success",
  "data": null,
  "requestId": "xxx"
}
```

---

## 8. 开始验收订单

**HTTP 方法:** `PATCH`

**URL:** `/api/v1/orders/{order_code}/begin-inspect-order`

**路径参数:**

- `order_code`: 订单号

**请求体:** 无

**响应:**

```json
{
  "code": 200,
  "message": "success",
  "data": "MARKET_INSPECTING",
  "requestId": "xxx"
}
```

---

## 9. 接受订单

**HTTP 方法:** `PATCH`

**URL:** `/api/v1/orders/{order_code}/accept`

**路径参数:**

- `order_code`: 订单号

**请求体:** 无

**响应:**

```json
{
  "code": 200,
  "message": "success",
  "data": null,
  "requestId": "xxx"
}
```

---

## 10. 换货请求

**HTTP 方法:** `PATCH`

**URL:** `/api/v1/orders/{order_code}/exchange-request`

**路径参数:**

- `order_code`: 订单号

**请求体:** 无

**响应:**

```json
{
  "code": 200,
  "message": "success",
  "data": null,
  "requestId": "xxx"
}
```

---

## 11. 退货请求

**HTTP 方法:** `PATCH`

**URL:** `/api/v1/orders/{order_code}/return`

**路径参数:**

- `order_code`: 订单号

**请求体:** 无

**说明:**

- 客户租户首次调用会将订单状态从 `CUSTOMER_INSPECTING` 更新为 `RETURN_REQUESTED`。
- 市场租户或已申请退货的客户再次调用，会将订单状态更新为 `RETURNED`。

**响应（客户首次申请退货）:**

```json
{
  "code": 200,
  "message": "success",
  "data": "RETURN_REQUESTED",
  "requestId": "xxx"
}
```

**响应（市场确认退货）:**

```json
{
  "code": 200,
  "message": "success",
  "data": "RETURNED",
  "requestId": "xxx"
}
```

---

## 12. 开始换货验收

**HTTP 方法:** `PATCH`

**URL:** `/api/v1/orders/{order_code}/begin-exchange-inspect-order`

**路径参数:**

- `order_code`: 订单号

**请求体:** 无

**说明:**

- 市场调用用于确认换货商品质量，状态将变更为 `EXCHANGE_INSPECTING`。
- 客户调用用于最终验收换货商品，状态将变更为 `EXCHANGE_COMPLETED`。

**响应（市场开始换货验收）:**

```json
{
  "code": 200,
  "message": "success",
  "data": "EXCHANGE_INSPECTING",
  "requestId": "xxx"
}
```

**响应（客户验收换货完成）:**

```json
{
  "code": 200,
  "message": "success",
  "data": "EXCHANGE_COMPLETED",
  "requestId": "xxx"
}
```

---

## 13. 配送到客户

**HTTP 方法:** `PATCH`

**URL:** `/api/v1/orders/{order_code}/deliver-to-customer`

**路径参数:**

- `order_code`: 订单号

**请求体:** 无

**说明:**

- 正常流程：市场从 `MARKET_ACCEPTED` 状态发货，状态将更新为 `MARKET_DELIVERING`。
- 换货流程：市场需要在换货验收通过后再次发货，状态将更新为 `CUSTOMER_INSPECTING`，随后客户调用“开始换货验收”接口完成验收。

**响应（普通配送场景）:**

```json
{
  "code": 200,
  "message": "success",
  "data": "MARKET_DELIVERING",
  "requestId": "xxx"
}
```

**响应（换货再次配送）:**

```json
{
  "code": 200,
  "message": "success",
  "data": "CUSTOMER_INSPECTING",
  "requestId": "xxx"
}
```

---

## 14. 获取售后订单列表

**HTTP 方法:** `GET`

**URL:** `/api/v1/tenants/{tenant_hash}/orders/after_sale`

**路径参数:**

- `tenant_hash`: 租户哈希值

**响应:**

```json
{
  "code": 200,
  "message": "success",
  "data": [
    {
      "orderCode": "ORD20240115001",
      "orderStatus": "AFTER_SALE",
      "acceptedAt": "2024-01-14T10:30:00Z"
    }
  ],
  "requestId": "xxx"
}
```

---

## 15. 取消订单

**HTTP 方法:** `DELETE`

**URL:** `/api/v1/orders/{order_code}/cancel`

**路径参数:**

- `order_code`: 订单号

**请求体:** 无

**响应:**

```json
{
  "code": 200,
  "message": "success",
  "data": null,
  "requestId": "xxx"
}
```

---

## 16. 获取供应商今日订单汇总

**HTTP 方法:** `GET`

**URL:** `/api/v1/providers/{provider_hash}/orders/today-summary`

**路径参数:**

- `provider_hash`: 供应商哈希值

**响应:**

```json
{
  "code": 200,
  "message": "success",
  "data": [
    {
      "productId": "P001",
      "productName": "测试产品",
      "totalQuantity": "10.00",
      "unit": "个",
      "processingRequirements": "加工要求",
      "customerName": "客户名称",
      "remark": "备注信息"
    }
  ],
  "requestId": "xxx"
}
```

---

## 订单状态说明

订单可能的状态值：

- `PENDING`: 待分配
- `ASSIGNED`: 已分配供应商
- `SUPPLIER_PREPARING`: 供应商备货中
- `SUPPLIER_DELIVERING`: 供应商配送至市场中
- `MARKET_INSPECTING`: 市场验货中
- `MARKET_ACCEPTED`: 市场验货完成
- `MARKET_DELIVERING`: 市场配送至客户中
- `CUSTOMER_INSPECTING`: 客户验货中
- `COMPLETED`: 订单完成
- `RETURN_REQUESTED`: 已提交退货申请
- `RETURNED`: 退货完成
- `EXCHANGE_REQUESTED`: 换货申请中
- `EXCHANGE_IN_PROGRESS`: 换货备货中
- `EXCHANGE_DELIVERING`: 换货配送至市场中
- `EXCHANGE_INSPECTING`: 换货验货中
- `EXCHANGE_NEW_DELIVERING`: 换货新商品配送至客户中
- `EXCHANGE_COMPLETED`: 换货流程完成
- `CANCELLED`: 订单已取消

---

## 错误响应格式

当请求出错时，响应格式如下：

```json
{
  "code": 400,
  "message": "错误描述信息",
  "data": null,
  "requestId": "xxx"
}
```

常见错误码：

- `400`: 请求参数错误
- `401`: 未授权
- `403`: 禁止访问
- `404`: 资源不存在
- `500`: 服务器内部错误

