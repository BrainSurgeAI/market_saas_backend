use axum::{extract::Path, Extension};
use tracing::{debug, info};

use crate::{
    common::{ApiResponse, AppError},
    dto::{delivery_staff::DeliveryStaffDTO, ValidatedJSON},
    middleware::context::RequestContext,
    models::claims::Claims,
    repositories::{delivery_staff_traits::DeliveryStaffRepository},
    models::tenant_type::TenantType,
    utils::validate_json_fmt::Json,
};

pub async fn get_delivery_staff_by_provider<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Path(tenant_hash): Path<String>,
) -> Result<Json<ApiResponse<Vec<DeliveryStaffDTO>>>, AppError>
where
    T: DeliveryStaffRepository + Send + Sync,
{
    if claims.tenant_type != TenantType::Provider.to_string() {
        return Err(AppError::Forbidden(
            "Only provider can get delivery staff".to_string(),
        ));
    }

    info!("Getting delivery staff for provider: {}", tenant_hash);

    let delivery_staff = repo.get_delivery_staff_by_provider(&tenant_hash).await?;
    Ok(Json(ApiResponse::new(Some(delivery_staff), &context)))
}

pub async fn create_delivery_staff<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Path(tenant_hash): Path<String>,
    ValidatedJSON(delivery_staff): ValidatedJSON<DeliveryStaffDTO>,
) -> Result<Json<ApiResponse<DeliveryStaffDTO>>, AppError>
where
    T: DeliveryStaffRepository + Send + Sync,
{
    // 验证权限
    if claims.tenant_type != TenantType::Provider.to_string() {
        return Err(AppError::Forbidden(
            "Only provider can create delivery staff".to_string(),
        ));
    }

    // 检查身份证是否存在
    if repo.is_id_card_exists(&delivery_staff.id_card).await? {
        return Err(AppError::Conflict(format!(
            "身份证号 {} 已存在",
            delivery_staff.id_card
        )));
    }

    // 创建配送员
    let delivery_staff = repo
        .create_delivery_staff(&tenant_hash, &delivery_staff)
        .await?;
    Ok(Json(ApiResponse::new(Some(delivery_staff), &context)))
}

/// 禁用或启用配送员
pub async fn disable_or_enable_delivery_staff<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Path((tenant_hash, id_card)): Path<(String, String)>,
) -> Result<Json<ApiResponse<()>>, AppError>
where
    T: DeliveryStaffRepository + Send + Sync,
{
    info!(
        "Disabling or enabling delivery staff for tenant: {}",
        tenant_hash
    );
    if claims.tenant_type != TenantType::Provider.to_string() {
        return Err(AppError::Forbidden(
            "Only provider can disable or enable delivery staff".to_string(),
        ));
    }

    debug!(
        "Disabling or enabling delivery staff for tenant: {}",
        tenant_hash
    );
    repo.disable_or_enable_delivery_staff(&tenant_hash, &id_card)
        .await?;
    Ok(Json(ApiResponse::new(Some(()), &context)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use mockall::predicate::*;
    use mockall::*;

    mock! {
        DeliveryStaffRepo {}
        #[async_trait::async_trait]
        impl DeliveryStaffRepository for DeliveryStaffRepo {
            async fn get_delivery_staff_by_provider(&self, provider_hash: &str) -> Result<Vec<DeliveryStaffDTO>, AppError>;
            async fn create_delivery_staff(&self, provider_hash: &str, delivery_staff: &DeliveryStaffDTO) -> Result<DeliveryStaffDTO, AppError>;
            async fn is_id_card_exists(&self, id_card: &str) -> Result<bool, AppError>;
            async fn disable_or_enable_delivery_staff(&self, provider_hash: &str, id_card: &str) -> Result<(), AppError>;
        }
    }

    // 辅助函数创建有效的测试数据
    fn create_test_context() -> (RequestContext, String) {
        let request_id = "test-request-id".to_string();
        let tenant_hash = "test_provider_hash".to_string();

        let context = RequestContext {
            request_id,
            client_ip: None,
        };

        (context, tenant_hash)
    }

    fn create_provider_claims(tenant_hash: &str) -> Claims {
        Claims {
            tenant_type: "PROVIDER".to_string(),
            tenant_name: "测试供应商".to_string(),
            tenant_hash: tenant_hash.to_string(),
            username: "test_user".to_string(),
            roles: vec!["PROVIDER".to_string()],
           
            exp: 0,
            is_super_admin: false,
        }
    }

    fn create_customer_claims() -> Claims {
        Claims {
            tenant_type: "CUSTOMER".to_string(),
            tenant_name: "测试客户".to_string(),
            tenant_hash: "customer_hash".to_string(),
            username: "test_user".to_string(),
            roles: vec!["CUSTOMER".to_string()],
            exp: 0,
            is_super_admin: false,
        }
    }

    #[allow(dead_code)]
    fn create_valid_staff() -> DeliveryStaffDTO {
        DeliveryStaffDTO {
            name: "张三".to_string(),
            phone: "13800138000".to_string(),
            id_card: "110101199001011234".to_string(),
            created_by: "admin".to_string(),
            status: 1,
            remark: Some("测试备注".to_string()),
            created_at: Some(Utc::now()),
        }
    }

    #[tokio::test]
    async fn test_get_delivery_staff_success() {
        // 准备测试数据
        let provider_hash = "test_provider_hash".to_string();
        let context = RequestContext {
            request_id: "test-request-id".to_string(),
            client_ip: None,
        };
        let claims = create_provider_claims(&provider_hash);
        // 创建模拟的配送人员数据
        let mock_staff = vec![
            DeliveryStaffDTO {
                name: "张三".to_string(),
                phone: "13800138000".to_string(),
                id_card: "110101199001011234".to_string(),
                created_by: "admin".to_string(),
                status: 1,
                remark: Some("测试备注".to_string()),
                created_at: Some(Utc::now()),
            },
            DeliveryStaffDTO {
                name: "李四".to_string(),
                phone: "13900139000".to_string(),
                id_card: "110101199001011235".to_string(),
                created_by: "admin".to_string(),
                status: 1,
                remark: None,
                created_at: Some(Utc::now()),
            },
        ];

        // 创建Mock实例
        let mut mock_repo = MockDeliveryStaffRepo::new();

        // 设置mock行为
        mock_repo
            .expect_get_delivery_staff_by_provider()
            .with(eq(provider_hash.clone()))
            .times(1)
            .returning(move |_| Ok(mock_staff.clone()));

        // 执行测试
        let result = get_delivery_staff_by_provider(
            Extension(mock_repo),
            Extension(context),
            Extension(claims),
            Path(provider_hash),
        )
        .await;

        // 验证结果
        assert!(result.is_ok());
        let api_response = result.unwrap().0;
        assert_eq!(api_response.code, 200);

        let staff_list = api_response.data.unwrap();
        assert_eq!(staff_list.len(), 2);
        assert_eq!(staff_list[0].name, "张三");
        assert_eq!(staff_list[1].name, "李四");
    }

    #[tokio::test]
    async fn test_get_delivery_staff_forbidden() {
        // 准备测试数据 - 使用非供应商角色
        let (context, customer_hash) = create_test_context();
        let claims = create_customer_claims();

        // 创建Mock实例
        let mock_repo = MockDeliveryStaffRepo::new();

        // 执行测试
        let result = get_delivery_staff_by_provider(
            Extension(mock_repo),
            Extension(context),
            Extension(claims),
            Path(customer_hash),
        )
        .await;

        // 验证结果 - 应该返回禁止访问错误
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Forbidden(msg) => {
                assert_eq!(msg, "Only provider can get delivery staff");
            }
            err => panic!("预期是Forbidden错误，但得到了: {:?}", err),
        }
    }

    #[tokio::test]
    async fn test_get_delivery_staff_not_found() {
        // 准备测试数据
        let (context, provider_hash) = create_test_context();
        let claims = create_provider_claims(&provider_hash);

        // 创建Mock实例
        let mut mock_repo = MockDeliveryStaffRepo::new();

        // 设置mock行为 - 模拟未找到供应商
        mock_repo
            .expect_get_delivery_staff_by_provider()
            .with(eq(provider_hash.clone()))
            .times(1)
            .returning(|_| Err(AppError::NotFound("Provider not found".to_string())));

        // 执行测试
        let result = get_delivery_staff_by_provider(
            Extension(mock_repo),
            Extension(context),
            Extension(claims),
            Path(provider_hash),
        )
        .await;

        // 验证结果
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::NotFound(msg) => {
                assert_eq!(msg, "Provider not found");
            }
            err => panic!("预期是NotFound错误，但得到了: {:?}", err),
        }
    }

    #[tokio::test]
    async fn test_disable_or_enable_delivery_staff_success() {
        // 准备测试数据
        let (context, tenant_hash) = create_test_context();
        let claims = create_provider_claims(&tenant_hash);
        let id_card = "110101199001011234".to_string();

        // 创建Mock实例
        let mut mock_repo = MockDeliveryStaffRepo::new();

        // 设置mock行为 - 成功禁用/启用
        mock_repo
            .expect_disable_or_enable_delivery_staff()
            .with(eq(tenant_hash.clone()), eq(id_card.clone()))
            .times(1)
            .returning(|_, _| Ok(()));

        // 执行测试
        let result = disable_or_enable_delivery_staff(
            Extension(mock_repo),
            Extension(context),
            Extension(claims),
            Path((tenant_hash, id_card)),
        )
        .await;

        // 验证结果
        assert!(result.is_ok());
        let api_response = result.unwrap().0;
        assert_eq!(api_response.code, 200);
    }

    #[tokio::test]
    async fn test_disable_or_enable_delivery_staff_forbidden() {
        // 准备测试数据 - 使用非供应商角色
        let (context, tenant_hash) = create_test_context();
        let claims = create_customer_claims();
        let id_card = "110101199001011234".to_string();

        // 创建Mock实例
        let mock_repo = MockDeliveryStaffRepo::new();

        // 执行测试
        let result = disable_or_enable_delivery_staff(
            Extension(mock_repo),
            Extension(context),
            Extension(claims),
            Path((tenant_hash, id_card)),
        )
        .await;

        // 验证结果 - 应该返回禁止访问错误
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Forbidden(msg) => {
                assert_eq!(msg, "Only provider can disable or enable delivery staff");
            }
            err => panic!("预期是Forbidden错误，但得到了: {:?}", err),
        }
    }

    #[tokio::test]
    async fn test_disable_or_enable_delivery_staff_not_found() {
        // 准备测试数据
        let (context, tenant_hash) = create_test_context();
        let claims = create_provider_claims(&tenant_hash);
        let id_card = "110101199001011234".to_string();

        // 创建Mock实例
        let mut mock_repo = MockDeliveryStaffRepo::new();

        // 设置mock行为 - 配送员不存在
        mock_repo
            .expect_disable_or_enable_delivery_staff()
            .with(eq(tenant_hash.clone()), eq(id_card.clone()))
            .times(1)
            .returning(|_, _| Err(AppError::NotFound("配送员不存在".to_string())));

        // 执行测试
        let result = disable_or_enable_delivery_staff(
            Extension(mock_repo),
            Extension(context),
            Extension(claims),
            Path((tenant_hash, id_card)),
        )
        .await;

        // 验证结果
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::NotFound(msg) => {
                assert_eq!(msg, "配送员不存在");
            }
            err => panic!("预期是NotFound错误，但得到了: {:?}", err),
        }
    }

    #[tokio::test]
    async fn test_disable_or_enable_delivery_staff_id_card_invalid() {
        // 准备测试数据
        let (context, tenant_hash) = create_test_context();
        let claims = create_provider_claims(&tenant_hash);
        let id_card = "invalid_id_card".to_string(); // 格式错误的身份证号

        // 创建Mock实例
        let mut mock_repo = MockDeliveryStaffRepo::new();

        // 设置mock行为 - 身份证号格式错误
        mock_repo
            .expect_disable_or_enable_delivery_staff()
            .with(eq(tenant_hash.clone()), eq(id_card.clone()))
            .times(1)
            .returning(|_, _| Err(AppError::Validation("身份证号格式错误".to_string())));

        // 执行测试
        let result = disable_or_enable_delivery_staff(
            Extension(mock_repo),
            Extension(context),
            Extension(claims),
            Path((tenant_hash, id_card)),
        )
        .await;

        // 验证结果
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Validation(msg) => {
                assert_eq!(msg, "身份证号格式错误");
            }
            err => panic!("预期是Validation错误，但得到了: {:?}", err),
        }
    }

    #[tokio::test]
    async fn test_disable_or_enable_delivery_staff_tenant_not_found() {
        // 准备测试数据
        let (context, tenant_hash) = create_test_context();
        let claims = create_provider_claims(&tenant_hash);
        let id_card = "110101199001011234".to_string();

        // 创建Mock实例
        let mut mock_repo = MockDeliveryStaffRepo::new();

        // 设置mock行为 - 供应商不存在
        mock_repo
            .expect_disable_or_enable_delivery_staff()
            .with(eq(tenant_hash.clone()), eq(id_card.clone()))
            .times(1)
            .returning(|_, _| Err(AppError::NotFound("供应商不存在".to_string())));

        // 执行测试
        let result = disable_or_enable_delivery_staff(
            Extension(mock_repo),
            Extension(context),
            Extension(claims),
            Path((tenant_hash, id_card)),
        )
        .await;

        // 验证结果
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::NotFound(msg) => {
                assert_eq!(msg, "供应商不存在");
            }
            err => panic!("预期是NotFound错误，但得到了: {:?}", err),
        }
    }

    #[tokio::test]
    async fn test_disable_or_enable_delivery_staff_database_error() {
        // 准备测试数据
        let (context, tenant_hash) = create_test_context();
        let claims = create_provider_claims(&tenant_hash);
        let id_card = "110101199001011234".to_string();

        // 创建Mock实例
        let mut mock_repo = MockDeliveryStaffRepo::new();

        // 设置mock行为 - 数据库错误
        mock_repo
            .expect_disable_or_enable_delivery_staff()
            .with(eq(tenant_hash.clone()), eq(id_card.clone()))
            .times(1)
            .returning(|_, _| Err(AppError::Internal("数据库连接失败".to_string())));

        // 执行测试
        let result = disable_or_enable_delivery_staff(
            Extension(mock_repo),
            Extension(context),
            Extension(claims),
            Path((tenant_hash, id_card)),
        )
        .await;

        // 验证结果
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Internal(msg) => {
                assert_eq!(msg, "数据库连接失败");
            }
            err => panic!("预期是Internal错误，但得到了: {:?}", err),
        }
    }

    #[tokio::test]
    async fn test_disable_or_enable_delivery_staff_already_in_state() {
        // 准备测试数据
        let (context, tenant_hash) = create_test_context();
        let claims = create_provider_claims(&tenant_hash);
        let id_card = "110101199001011234".to_string();

        // 创建Mock实例
        let mut mock_repo = MockDeliveryStaffRepo::new();

        // 设置mock行为 - 状态已经是目标状态
        mock_repo
            .expect_disable_or_enable_delivery_staff()
            .with(eq(tenant_hash.clone()), eq(id_card.clone()))
            .times(1)
            .returning(|_, _| Err(AppError::Conflict("配送员已经是该状态".to_string())));

        // 执行测试
        let result = disable_or_enable_delivery_staff(
            Extension(mock_repo),
            Extension(context),
            Extension(claims),
            Path((tenant_hash, id_card)),
        )
        .await;

        // 验证结果
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Conflict(msg) => {
                assert_eq!(msg, "配送员已经是该状态");
            }
            err => panic!("预期是Conflict错误，但得到了: {:?}", err),
        }
    }

    #[tokio::test]
    async fn test_disable_or_enable_delivery_staff_empty_id_card() {
        // 准备测试数据
        let (context, tenant_hash) = create_test_context();
        let claims = create_provider_claims(&tenant_hash);
        let id_card = "".to_string(); // 空身份证号

        // 创建Mock实例
        let mut mock_repo = MockDeliveryStaffRepo::new();

        // 设置mock行为 - 空身份证号
        mock_repo
            .expect_disable_or_enable_delivery_staff()
            .with(eq(tenant_hash.clone()), eq(id_card.clone()))
            .times(1)
            .returning(|_, _| Err(AppError::Validation("身份证号不能为空".to_string())));

        // 执行测试
        let result = disable_or_enable_delivery_staff(
            Extension(mock_repo),
            Extension(context),
            Extension(claims),
            Path((tenant_hash, id_card)),
        )
        .await;

        // 验证结果
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Validation(msg) => {
                assert_eq!(msg, "身份证号不能为空");
            }
            err => panic!("预期是Validation错误，但得到了: {:?}", err),
        }
    }
}
