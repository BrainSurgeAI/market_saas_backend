use market_saas_backend::common::ApiResponse;
use uuid::Uuid;

// 测试用的数据结构
#[derive(serde::Serialize, Debug, PartialEq, Clone)]
struct TestData {
    id: i32,
    name: String,
}

#[test]
fn test_api_response_error() {
    // 测试错误响应
    let response: ApiResponse<TestData> = ApiResponse::error(400, "Bad Request");

    // 验证响应字段
    assert_eq!(response.code, 400);
    assert_eq!(response.message, "Bad Request");
    assert!(response.data.is_none());
    // 错误响应会生成新的request_id
    assert!(!response.request_id.is_empty());
    assert!(response.timestamp.timestamp() > 0);
}

#[test]
fn test_api_response_error_with_string() {
    // 测试使用String的错误响应
    let error_message = "Internal Server Error".to_string();
    let response: ApiResponse<TestData> = ApiResponse::error(500, error_message);

    assert_eq!(response.code, 500);
    assert_eq!(response.message, "Internal Server Error");
    assert!(response.data.is_none());
}

#[test]
fn test_api_response_error_with_str_slice() {
    // 测试使用&str的错误响应
    let response: ApiResponse<TestData> = ApiResponse::error(404, "Not Found");

    assert_eq!(response.code, 404);
    assert_eq!(response.message, "Not Found");
    assert!(response.data.is_none());
}

#[test]
fn test_api_response_error_serialization() {
    // 测试错误响应序列化
    let response: ApiResponse<TestData> = ApiResponse::error(422, "Validation Error");

    let json_result = serde_json::to_string(&response);
    assert!(json_result.is_ok());

    let json_str = json_result.unwrap();

    // 验证错误响应的JSON格式
    assert!(json_str.contains("\"code\":422"));
    assert!(json_str.contains("\"message\":\"Validation Error\""));
    assert!(!json_str.contains("\"data\":")); // 错误响应没有data字段
    assert!(json_str.contains("\"requestId\""));
    assert!(json_str.contains("\"timestamp\""));
}

#[test]
fn test_api_response_request_id_format() {
    // 测试错误响应生成的request_id格式是否为有效UUID
    let response: ApiResponse<TestData> = ApiResponse::error(500, "Server Error");

    // 验证request_id是有效的UUID格式
    let parse_result = Uuid::parse_str(&response.request_id);
    assert!(parse_result.is_ok());
}

#[test]
fn test_api_response_timestamp_recent() {
    // 测试时间戳是否为最近时间
    use chrono::Utc;

    let before = Utc::now();
    let response: ApiResponse<TestData> = ApiResponse::error(400, "Test");
    let after = Utc::now();

    // 验证时间戳在合理范围内
    assert!(response.timestamp >= before);
    assert!(response.timestamp <= after);
}

#[test]
fn test_api_response_multiple_error_codes() {
    // 测试各种HTTP状态码
    let test_cases = vec![
        (200, "OK"),
        (201, "Created"),
        (400, "Bad Request"),
        (401, "Unauthorized"),
        (403, "Forbidden"),
        (404, "Not Found"),
        (422, "Unprocessable Entity"),
        (500, "Internal Server Error"),
        (502, "Bad Gateway"),
        (503, "Service Unavailable"),
    ];

    for (code, message) in test_cases {
        let response: ApiResponse<TestData> = ApiResponse::error(code, message);
        assert_eq!(response.code, code);
        assert_eq!(response.message, message);
        assert!(response.data.is_none());
    }
}

#[test]
fn test_api_response_empty_message() {
    // 测试空消息的边界情况
    let response: ApiResponse<TestData> = ApiResponse::error(400, "");
    assert_eq!(response.message, "");
    assert_eq!(response.code, 400);
}

#[test]
fn test_api_response_long_message() {
    // 测试长消息
    let long_message = "a".repeat(1000);
    let response: ApiResponse<TestData> = ApiResponse::error(400, long_message.clone());
    assert_eq!(response.message, long_message);
}

#[test]
fn test_api_response_serialization_data_skip() {
    // 测试当data为None时的序列化行为
    let response: ApiResponse<String> = ApiResponse::error(404, "Not Found");

    let json_result = serde_json::to_string(&response);
    assert!(json_result.is_ok());

    let json_str = json_result.unwrap();

    // 验证data字段被正确跳过
    assert!(!json_str.contains("\"data\":null"));
    assert!(!json_str.contains("\"data\":"));

    // 但其他字段应该存在
    assert!(json_str.contains("\"code\":404"));
    assert!(json_str.contains("\"message\":\"Not Found\""));
    assert!(json_str.contains("\"requestId\""));
    assert!(json_str.contains("\"timestamp\""));
}

#[test]
fn test_api_response_different_data_types() {
    // 测试不同数据类型的错误响应能正确工作
    let _string_response: ApiResponse<String> = ApiResponse::error(400, "String type");
    let _number_response: ApiResponse<i32> = ApiResponse::error(400, "Number type");
    let _bool_response: ApiResponse<bool> = ApiResponse::error(400, "Bool type");
    let _vec_response: ApiResponse<Vec<i32>> = ApiResponse::error(400, "Vec type");

    // 如果能成功创建这些响应而不发生编译错误，测试就通过了
    assert!(true);
}

#[test]
fn test_api_response_message_into_string() {
    // 测试message参数的Into<String> trait
    let response1: ApiResponse<TestData> = ApiResponse::error(400, "str literal");
    let response2: ApiResponse<TestData> = ApiResponse::error(400, String::from("owned string"));
    let response3: ApiResponse<TestData> =
        ApiResponse::error(400, format!("formatted {}", "string"));

    assert_eq!(response1.message, "str literal");
    assert_eq!(response2.message, "owned string");
    assert_eq!(response3.message, "formatted string");
}

#[test]
fn test_api_response_debug_trait() {
    // 测试Debug trait是否正常工作
    let response: ApiResponse<TestData> = ApiResponse::error(500, "Debug test");
    let debug_output = format!("{:?}", response);

    // Debug输出应该包含主要字段
    assert!(debug_output.contains("ApiResponse"));
    assert!(debug_output.contains("500"));
    assert!(debug_output.contains("Debug test"));
}

#[test]
fn test_api_response_concurrent_creation() {
    // 测试并发创建响应时request_id的唯一性
    use std::collections::HashSet;
    use std::thread;

    let handles: Vec<_> = (0..10)
        .map(|i| {
            thread::spawn(move || {
                let response: ApiResponse<TestData> =
                    ApiResponse::error(400, format!("Error {}", i));
                response.request_id
            })
        })
        .collect();

    let mut request_ids = HashSet::new();
    for handle in handles {
        let request_id = handle.join().unwrap();
        assert!(
            request_ids.insert(request_id),
            "Request ID should be unique"
        );
    }

    assert_eq!(request_ids.len(), 10);
}
