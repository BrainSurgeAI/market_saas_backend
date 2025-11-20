use std::{
    sync::Arc,
    task::{Context, Poll},
    time::Instant,
};

use axum::{body::Body, http::Request, response::Response};

use futures::future::BoxFuture;
use serde_json::json;
use tower::{Layer, Service};

use crate::{
    //dto::system_log::SystemLogDTO,
    dto::system_log::SystemLogDTO,
    middleware::context::RequestContext,
    models::claims::Claims,
    repositories::system_log_repo::SystemLogRepository,
    services::system_log_service::SystemLogService, // repositories::system_log_repo::SystemLogRepository,
                                                    // services::system_log_service::SystemLogService,
};

/// 操作日志上下文
#[derive(Clone)]
pub struct OperationContext {
    pub category: String,
    pub component: String,
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
}

impl OperationContext {
    #[allow(dead_code)]
    pub fn new(category: &str, component: &str, action: &str) -> Self {
        Self {
            category: category.to_string(),
            component: component.to_string(),
            action: action.to_string(),
            resource_type: None,
            resource_id: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_resource(mut self, resource_type: &str, resource_id: Option<&str>) -> Self {
        self.resource_type = Some(resource_type.to_string());
        self.resource_id = resource_id.map(|s| s.to_string());
        self
    }
}

/// 日志中间件层
#[derive(Clone)]
pub struct LoggingLayer<L> {
    log_service: Arc<SystemLogService<L>>,
}

impl<L> LoggingLayer<L>
where
    L: SystemLogRepository + Send + Sync + 'static,
{
    pub fn new(log_service: Arc<SystemLogService<L>>) -> Self {
        Self { log_service }
    }
}

impl<S, L> Layer<S> for LoggingLayer<L>
where
    S: Service<Request<Body>, Response = Response> + Send + 'static,
    L: SystemLogRepository + Send + Sync + 'static,
    S::Future: Send + 'static,
{
    type Service = LoggingMiddleware<S, L>;

    fn layer(&self, inner: S) -> Self::Service {
        LoggingMiddleware {
            inner,
            log_service: Arc::clone(&self.log_service),
        }
    }
}

/// 日志中间件服务
#[derive(Clone)]
pub struct LoggingMiddleware<S, L> {
    inner: S,
    log_service: Arc<SystemLogService<L>>,
}

impl<S, L> Service<Request<Body>> for LoggingMiddleware<S, L>
where
    S: Service<Request<Body>, Response = Response> + Send + 'static,
    L: SystemLogRepository + Send + Sync + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<Body>) -> Self::Future {
        let log_service = Arc::clone(&self.log_service);
        let start_time = Instant::now();

        let operation_context = request.extensions().get::<OperationContext>().cloned();
        // if let Some(ctx) = &operation_context {
        //     tracing::debug!(
        //         "操作上下文: category={}, component={}, action={}",
        //         ctx.category,
        //         ctx.component,
        //         ctx.action
        //     );
        // }
        // 获取请求上下文和用户信息
        let request_context = request.extensions().get::<RequestContext>().cloned();
        // if let Some(ctx) = &request_context {
        //     tracing::debug!(
        //         "请求上下文: request_id={}, client_ip={:?}",
        //         ctx.request_id,
        //         ctx.client_ip
        //     );
        // } else {
        //     tracing::debug!("未找到请求上下文");
        // }

        let claims = request.extensions().get::<Claims>().cloned();
      

        let future = self.inner.call(request);
        Box::pin(async move {
            // 调用内部服务处理请求
            let response = future.await?;

            // 如果存在操作上下文，记录日志
            if let Some(operation) = operation_context {
                let elapsed = start_time.elapsed().as_millis() as u32;
                let status = response.status();

                // 准备日志信息
                let mut log_builder = SystemLogDTO::builder()
                    .category(&operation.category)
                    .component(&operation.component)
                    .action(&operation.action);

                // 添加资源信息
                if let Some(resource_type) = &operation.resource_type {
                    log_builder = log_builder.resource_type(resource_type);
                }

                if let Some(resource_id) = &operation.resource_id {
                    log_builder = log_builder.resource_id(resource_id);
                }

                // 添加请求上下文
                if let Some(ctx) = &request_context {
                    log_builder = log_builder
                        .request_id(&ctx.request_id)
                        .ip_address(ctx.client_ip.as_deref());
                }

                // 添加用户信息
                if let Some(user) = &claims {
                    log_builder = log_builder
                        .user_id(&user.username)
                        .tenant_type(&user.tenant_type);

                    if let Ok(tenant_id) = user.tenant_hash.parse::<i32>() {
                        log_builder = log_builder.tenant_id(tenant_id);
                    }
                }

                let log = log_builder.build().with_execution_time(elapsed);

                // 根据响应状态决定日志类型
                if status.is_success() {
                    let log = log
                        .with_message(&format!(
                            "操作成功: {}/{}",
                            operation.component, operation.action
                        ))
                        .with_details(json!({
                            "status_code": status.as_u16(),
                        }));

                    tokio::spawn(async move {
                        if let Err(e) = log_service.log_info(log).await {
                            tracing::error!("记录操作日志失败: {}", e);
                        }
                    });
                } else {
                    let log = log
                        .with_message(&format!(
                            "操作失败: {}/{}",
                            operation.component, operation.action
                        ))
                        .with_details(json!({
                            "status_code": status.as_u16(),
                        }))
                        .with_error_code(&format!("HTTP_{}", status.as_u16()));

                    tokio::spawn(async move {
                        if let Err(e) = log_service.log_error(log).await {
                            tracing::error!("记录操作失败日志失败: {}", e);
                        }
                    });
                }
            }
          
            Ok(response)
        })
    }
}
