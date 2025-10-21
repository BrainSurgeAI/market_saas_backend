# ApiResponse Usage Guide

## Overview

`ApiResponse<T>` is the unified API response structure in the project, used to standardize the response format of all API endpoints, ensuring consistency and predictability.

## Response Format

```json
{
  "code": 200,
  "message": "success",
  "data": { /* response data, optional */ },
  "requestId": "123e4567-e89b-12d3-a456-426614174000",
  "timestamp": "2023-01-01T00:00:00Z"
}
```

## Basic Usage

### 1. Success Responses

```rust
use crate::common::ApiResponse;
use crate::middleware::context::RequestContext;

// Success response with data
let users = vec![user1, user2, user3];
let response = ApiResponse::ok(users, &context);

// Success response without data
let response = ApiResponse::ok_empty(&context);

// Using the default new method
let response = ApiResponse::new(Some(data), &context);
let response = ApiResponse::new(None, &context);
```

### 2. Resource Creation Responses

```rust
// 201 Created - resource created successfully
let response = ApiResponse::created(Some(created_user), &context);

// 202 Accepted - request accepted, processing asynchronously
let response = ApiResponse::accepted(Some(task_id), &context);

// 204 No Content - operation successful, no return data
let response = ApiResponse::no_content(&context);
```

### 3. Error Responses

```rust
// Basic error responses
let response: ApiResponse<()> = ApiResponse::error(404, "User not found");
let response: ApiResponse<()> = ApiResponse::error(400, "Invalid request");

// Dynamic error messages
let response: ApiResponse<()> = ApiResponse::error(
    500, 
    format!("Database error: {}", err)
);

// Custom request_id error response (mainly for testing)
let response: ApiResponse<()> = ApiResponse::error_with_request_id(
    400, 
    "Test error", 
    "test-request-id"
);
```

### 4. Custom Status Code and Message

```rust
// Custom response
let response = ApiResponse::with_code_and_message(
    201,
    "Resource created successfully",
    Some(created_resource),
    &context
);

// Custom success message
let response = ApiResponse::with_code_and_message(
    200,
    "Operation completed",
    None,
    &context
);
```

## Usage in Service Layer

### Typical Service Method Pattern

```rust
use axum::Extension;
use crate::common::{ApiResponse, AppError};
use crate::middleware::context::RequestContext;
use crate::utils::validate_json_fmt::Json;

pub async fn create_user<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Json(payload): Json<UserCreateDto>,
) -> Result<Json<ApiResponse<UserResponseDto>>, AppError>
where
    T: UserRepository + Send + Sync,
{
    // Business logic processing
    let user = repo.create_user(&payload).await?;
    
    // Return 201 Created response
    Ok(Json(ApiResponse::created(Some(user), &context)))
}

pub async fn get_user<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Path(user_id): Path<i32>,
) -> Result<Json<ApiResponse<UserResponseDto>>, AppError>
where
    T: UserRepository + Send + Sync,
{
    let user = repo.get_user_by_id(user_id).await?;
    
    // Return 200 OK response
    Ok(Json(ApiResponse::ok(user, &context)))
}

pub async fn delete_user<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Path(user_id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError>
where
    T: UserRepository + Send + Sync,
{
    repo.delete_user(user_id).await?;
    
    // Return 204 No Content response
    Ok(Json(ApiResponse::no_content(&context)))
}
```

## Best Practices

### 1. Status Code Selection

- **200 OK**: Successfully retrieved data or operation successful
- **201 Created**: Resource created successfully
- **202 Accepted**: Request accepted, processing asynchronously
- **204 No Content**: Operation successful, no return data needed
- **400 Bad Request**: Client request error
- **401 Unauthorized**: Unauthorized
- **403 Forbidden**: Access forbidden
- **404 Not Found**: Resource not found
- **409 Conflict**: Resource conflict
- **422 Unprocessable Entity**: Validation failed
- **500 Internal Server Error**: Internal server error

### 2. Message Standards

```rust
// ✅ Good practice - clear, specific messages
ApiResponse::error(404, "User with ID 123 not found")
ApiResponse::error(400, "Invalid email format")
ApiResponse::created(Some(user), &context) // Use default "created" message

// ❌ Avoid - vague, unhelpful messages
ApiResponse::error(500, "Error")
ApiResponse::error(400, "Bad request")
```

### 3. Data Type Handling

```rust
// ✅ Explicit data types
let response: ApiResponse<Vec<User>> = ApiResponse::ok(users, &context);
let response: ApiResponse<User> = ApiResponse::ok(user, &context);
let response: ApiResponse<()> = ApiResponse::no_content(&context);

// ✅ Handle optional data
let user_option = repo.find_user_by_email(&email).await?;
let response = match user_option {
    Some(user) => ApiResponse::ok(user, &context),
    None => ApiResponse::error(404, "User not found"),
};
```

### 4. Error Handling Integration

```rust
// Integration with AppError
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let error_context = self.error_context();
        let response = ApiResponse::<()>::error(
            error_context.error_code, 
            error_context.message
        );
        (error_context.status_code, Json(response)).into_response()
    }
}
```

## Testing Examples

```rust
#[tokio::test]
async fn test_create_user_success() {
    let context = create_test_context();
    let user_data = UserCreateDto { /* ... */ };
    
    let response = create_user(repo, context, Json(user_data)).await.unwrap();
    
    assert_eq!(response.0.code, 201);
    assert_eq!(response.0.message, "created");
    assert!(response.0.data.is_some());
}

#[tokio::test]
async fn test_get_user_not_found() {
    let response: ApiResponse<()> = ApiResponse::error(404, "User not found");
    
    assert_eq!(response.code, 404);
    assert_eq!(response.message, "User not found");
    assert!(response.data.is_none());
}
```

## Serialization Behavior

- When the `data` field is `None`, it will be skipped during JSON serialization
- The `request_id` field is renamed to `requestId` in JSON
- Timestamps use ISO 8601 format

```json
// When data is Some
{
  "code": 200,
  "message": "success",
  "data": {"id": 1, "name": "John"},
  "requestId": "uuid-here",
  "timestamp": "2023-01-01T00:00:00Z"
}

// When data is None
{
  "code": 204,
  "message": "no content",
  "requestId": "uuid-here",
  "timestamp": "2023-01-01T00:00:00Z"
}
```

## Performance Considerations

1. **Avoid unnecessary cloning**: Use `ApiResponse::ok(data, &context)` instead of `ApiResponse::new(Some(data), &context)`
2. **Proper use of error responses**: Error responses automatically generate UUIDs, suitable for error scenarios
3. **Choose appropriate methods**: Select the most suitable constructor method based on specific scenarios

## Migration Guide

If you're migrating from an old response format, follow these steps:

1. Replace all manually constructed responses with `ApiResponse` methods
2. Ensure all success responses pass `RequestContext`
3. Standardize error response status codes and message formats
4. Update test cases to verify the new response format 