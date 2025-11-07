use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::middleware::context::RequestContext;

/// Unified API response structure for standardizing all API endpoint responses
///
/// # Examples
///
/// ```rust
/// // Success response with data
/// let response = ApiResponse::ok(user_data, &context);
///
/// // Resource creation response
/// let response = ApiResponse::created(Some(user_data), &context);
///
/// // Error response
/// let response: ApiResponse<()> = ApiResponse::error(404, "Resource not found");
/// ```
#[derive(Serialize, Debug, utoipa::ToSchema)]
pub(crate) struct ApiResponse<T> {
    /// HTTP status code
    pub(crate) code: i32,
    /// Response message
    pub(crate) message: String,

    /// Response data (optional, skipped during serialization when None)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) data: Option<T>,

    /// Request unique identifier
    #[serde(rename = "requestId")]
    pub(crate) request_id: String,

    /// Response generation timestamp
    pub(crate) timestamp: DateTime<Utc>,
}

#[allow(dead_code)]
impl<T> ApiResponse<T> {
    /// Creates a success response with default 200 status code
    ///
    /// # Parameters
    /// - `data`: Response data
    /// - `context`: Request context containing request_id and other information
    ///
    /// # Examples
    /// ```rust
    /// let response = ApiResponse::new(Some(user_data), &context);
    /// ```
    pub(crate) fn new(data: Option<T>, context: &RequestContext) -> Self {
        Self::with_code_and_message(200, "success", data, context)
    }

    /// Creates a response with custom status code and message
    ///
    /// # Parameters
    /// - `code`: HTTP status code
    /// - `message`: Response message
    /// - `data`: Response data
    /// - `context`: Request context
    ///
    /// # Examples
    /// ```rust
    /// let response = ApiResponse::with_code_and_message(
    ///     201,
    ///     "Resource created successfully",
    ///     Some(created_resource),
    ///     &context
    /// );
    /// ```
    pub(crate) fn with_code_and_message(
        code: i32,
        message: impl Into<String>,
        data: Option<T>,
        context: &RequestContext,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            data,
            request_id: context.request_id.clone(),
            timestamp: Utc::now(),
        }
    }

    /// Creates a 201 Created response for successful resource creation
    pub(crate) fn created(data: Option<T>, context: &RequestContext) -> Self {
        Self::with_code_and_message(201, "created", data, context)
    }

    /// Creates a 202 Accepted response for accepted but not yet processed requests
    pub(crate) fn accepted(data: Option<T>, context: &RequestContext) -> Self {
        Self::with_code_and_message(202, "accepted", data, context)
    }

    /// Creates a 204 No Content response for successful operations with no return data
    pub(crate) fn no_content(context: &RequestContext) -> Self {
        Self::with_code_and_message(204, "no content", None, context)
    }

    /// Creates an error response with auto-generated UUID as request_id
    ///
    /// # Parameters
    /// - `code`: HTTP error status code
    /// - `message`: Error message
    ///
    /// # Examples
    /// ```rust
    /// let response: ApiResponse<()> = ApiResponse::error(404, "User not found");
    /// let response: ApiResponse<()> = ApiResponse::error(500, format!("Database error: {}", err));
    /// ```
    pub(crate) fn error(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
            request_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
        }
    }

    /// Creates an error response with custom request_id (mainly for testing)
    ///
    /// # Parameters
    /// - `code`: HTTP error status code
    /// - `message`: Error message
    /// - `request_id`: Custom request ID
    pub(crate) fn error_with_request_id(
        code: i32,
        message: impl Into<String>,
        request_id: String,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
            request_id,
            timestamp: Utc::now(),
        }
    }
}

#[allow(dead_code)]
// Convenience methods for common success responses
impl<T> ApiResponse<T> {
    /// Creates a 200 OK response with data
    ///
    /// # Parameters
    /// - `data`: Response data (non-Option type)
    /// - `context`: Request context
    ///
    /// # Examples
    /// ```rust
    /// let response = ApiResponse::ok(user_list, &context);
    /// ```
    pub(crate) fn ok(data: T, context: &RequestContext) -> Self {
        Self::new(Some(data), context)
    }

    /// Creates a 200 OK response without data
    ///
    /// # Parameters
    /// - `context`: Request context
    ///
    /// # Examples
    /// ```rust
    /// let response = ApiResponse::ok_empty(&context);
    /// ```
    pub(crate) fn ok_empty(context: &RequestContext) -> Self {
        Self::new(None, context)
    }
}
