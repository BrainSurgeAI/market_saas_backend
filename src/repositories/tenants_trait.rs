use crate::{
    common::AppError,
    dto::{
        tenants::{
            BaseTenantDTO, Provider, QueryTenantByType, TenantAddressUpdate, TenantCreateDTO,
            TenantDetailDTO,
        },
        financial::FinancialResponseDto,
        users::UserResponseDto,
    },
    map_db_err,
};
use anyhow::Result;
use async_trait::async_trait;

use sqlx::{MySql, QueryBuilder};
use tracing::{debug, error};

use super::{my_sql_repository::MySqlRepository, generate_tenant_name_hash};

#[async_trait]
pub trait TenantRepository: Send + Sync {
    async fn find_tenant_by_username(
        &self,
        username: &str,
    ) -> Result<Option<BaseTenantDTO>, AppError>;

    async fn create_tenant(
        &self,
        market_hash: &str,
        tenant: &TenantCreateDTO,
    ) -> Result<(), AppError>;

    async fn verify_tenant_exists(&self, hashed_name: &str) -> Result<bool, AppError>;

    // get the financials of a tenant
    async fn get_tenant_financials(&self, hashed_name: &str)
        -> Result<Option<FinancialResponseDto>, AppError>;

    // check if a tenant name hash exists
    async fn is_tenant_exist(&self, name_hash: &str) -> Result<bool, AppError>;

    async fn get_tenant_users(&self, hashed_name: &str) -> Result<Vec<UserResponseDto>, AppError>;

    // add a user to a tenant
    //async fn add_user_to_tenant(&self, hashed_name: &str, user: &User) -> Result<u64, AppError>;

    async fn update_tenant_self(
        &self,
        hashed_name: &str,
        tenant: &TenantAddressUpdate,
    ) -> Result<u64, AppError>;

    async fn get_all_providers_by_market(
        &self,
        market_hash: &str,
    ) -> Result<Vec<Provider>, AppError>;

    async fn get_all_tenants_by_market(
        &self,
        market_hash: &str,
        query: &QueryTenantByType,
    ) -> Result<Vec<BaseTenantDTO>, AppError>;

    async fn delete_tenant(&self, market_hash: &str, tenant_hash: &str) -> Result<u64, AppError>;

    async fn update_tenant_by_market(
        &self,
        market_hash: &str,
        tenant_hash: &str,
        tenant: &TenantCreateDTO,
    ) -> Result<u64, AppError>;

    // Helper function to check if a tenant has relationship with a market
    async fn has_relationship(
        &self,
        market_hash: &str,
        tenant_hash: &str,
    ) -> Result<bool, AppError>;

    /// Find tenant detail by hashed name of tenant
    /// Returns None if tenant not found
    /// # Arguments
    /// * `name_hash` - Hashed name of the tenant
    /// # Returns
    /// * `Option<TenantDetailDTO>` - Tenant detail DTO if found
    async fn find_tenant_detail_by_hashed_name(
        &self,
        name_hash: &str,
    ) -> Result<Option<TenantDetailDTO>, AppError>;
}

#[async_trait]
impl TenantRepository for MySqlRepository {
    async fn verify_tenant_exists(&self, hashed_name: &str) -> Result<bool, AppError> {
        let tenant_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM tenants WHERE name_hash = ?)",
        )
        .bind(hashed_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!("Failed to verify tenant existence"))?;

        Ok(tenant_exists.unwrap_or(false))
    }

    async fn find_tenant_by_username(
        &self,
        username: &str,
    ) -> Result<Option<BaseTenantDTO>, AppError> {
        debug!("Finding tenant by username: {}", username);

        sqlx::query_as::<_, BaseTenantDTO>(
            r#"SELECT t.id, t.name, t.tenant_type, t.address, t.business_scope, 
               t.license_image, t.status, t.name_hash, t.created_at, t.updated_at, 
               t.verified_at, t.deleted_at FROM tenants t INNER JOIN users u ON t.id = u.tenant_id 
               WHERE u.username = ? AND u.tenant_id IS NOT NULL AND t.deleted_at IS NULL 
               ORDER BY t.created_at DESC LIMIT 1"#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!("Failed to find tenant by username"))
    }

    async fn create_tenant(
        &self,
        market_hash: &str,
        tenant: &TenantCreateDTO,
    ) -> Result<(), AppError> {
        let name_hash = generate_tenant_name_hash(&tenant.name).map_err(|e| {
            error!("Error hashing tenant name: {:?}", e);
            AppError::Internal("Error hashing tenant name".to_string())
        })?;

        let is_exist = self.is_tenant_exist(&name_hash).await?;
        if is_exist {
            error!("Tenant {} already exists", tenant.name);
            return Err(AppError::Conflict("Tenant already exists".to_string()));
        }

        let record = sqlx::query!("SELECT id FROM tenants WHERE name_hash = ?", market_hash)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get tenant id"))?;

        if record.is_none() {
            error!("Market {} not found", market_hash);
            return Err(AppError::NotFound("Market not found".to_string()));
        }

        let market_id = record.unwrap().id;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Failed to begin transaction"))?;

        let result = sqlx::query(
            "INSERT INTO tenants (name, name_hash, address, business_scope, tenant_type) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&tenant.name)
        .bind(&name_hash)
        .bind(&tenant.address)
        .bind(&tenant.business_scope)
        .bind(tenant.tenant_type.to_uppercase())
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to create tenant"))?;

        if result.rows_affected() == 0 {
            return Err(AppError::Internal("Failed to create tenant".to_string()));
        }

        let tenant_id = result.last_insert_id();

        let result = sqlx::query(
            "INSERT INTO tenant_relationships (market_id, provider_id, status) VALUES (?, ?, ?)",
        )
        .bind(market_id)
        .bind(tenant_id)
        .bind("ACTIVE")
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to create tenant relationship"))?;

        if result.rows_affected() == 0 {
            return Err(AppError::Internal(
                "Failed to create tenant relationship".to_string(),
            ));
        }

        tx.commit()
            .await
            .map_err(map_db_err!("Failed to commit transaction"))?;

        Ok(())
    }

    async fn get_tenant_financials(
        &self,
        hashed_name: &str,
    ) -> Result<Option<FinancialResponseDto>, AppError> {
        sqlx::query_as::<_, FinancialResponseDto>(
            r#"
           SELECT pf.* FROM tenants t 
           INNER JOIN provider_financial_profiles pf ON t.id = pf.tenant_id 
           WHERE t.name_hash = ?
           ORDER BY pf.created_at DESC
           LIMIT 1
           "#,
        )
        .bind(hashed_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get tenant financials"))
    }

    async fn is_tenant_exist(&self, name_hash: &str) -> Result<bool, AppError> {
        // let name_hash = tenant_name_hash(name).unwrap();
        let tenant = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM tenants WHERE name_hash = ?)",
        )
        .bind(name_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!("Failed to find tenant by name"))?;

        Ok(tenant.unwrap_or(false))
    }

    async fn get_tenant_users(&self, hashed_name: &str) -> Result<Vec<UserResponseDto>, AppError> {
        let users = sqlx::query_as::<_, UserResponseDto>(
            r#"SELECT u.id, u.name, u.username, u.email, u.phone, r.name as role,
            t.name as tenant_name, u.created_at, u.updated_at, u.deleted_at
            FROM users u 
            INNER JOIN tenants t ON u.tenant_id = t.id 
            INNER JOIN user_roles ur ON u.id = ur.user_id
            INNER JOIN roles r ON ur.role_id = r.id
            WHERE t.name_hash = ?
            ORDER BY u.created_at DESC"#,
        )
        .bind(hashed_name)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get tenant users"))?;

        Ok(users)
    }

    // async fn add_user_to_tenant(&self, hashed_name: &str, user: &User) -> Result<u64, AppError> {
    //     // 检查用户是否已存在
    //     let is_user_existing = self.is_user_exists(&user.username).await?;
    //     if is_user_existing {
    //         error!("User {} already exists", user.username);
    //         return Err(AppError::Validation("User already exists".to_string()));
    //     }

    //     let hashed_password =
    //         bcrypt::hash(user.password.clone(), bcrypt::DEFAULT_COST).map_err(|e| {
    //             error!("Error hashing password: {:?}", e);
    //             AppError::Internal("Error hashing password".to_string())
    //         })?;

    //     let mut tx = self
    //         .pool
    //         .begin()
    //         .await
    //         .map_err(map_db_err!("Failed to begin transaction"))?;

    //     let result = sqlx::query(
    //         "INSERT INTO users (username, email, phone, password_hash, tenant_id)
    //      SELECT ?, ?, ?, ?, id
    //      FROM tenants
    //      WHERE name_hash = ?",
    //     )
    //     .bind(&user.username)
    //     .bind(&user.email)
    //     .bind(&user.phone)
    //     .bind(&hashed_password)
    //     .bind(hashed_name)
    //     .execute(&mut *tx)
    //     .await
    //     .map_err(map_db_err!("Failed to add user to tenant"))?;

    //     let user_id = result.last_insert_id();

    //     for role_id in &user.roles {
    //         sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES (?, ?)")
    //             .bind(user_id)
    //             .bind(role_id)
    //             .execute(&mut *tx)
    //             .await
    //             .map_err(map_db_err!("Failed to create user role"))?;
    //     }

    //     tx.commit()
    //         .await
    //         .map_err(map_db_err!("Failed to commit transaction"))?;

    //     Ok(user_id)
    // }

    async fn update_tenant_self(
        &self,
        hashed_name: &str,
        tenant: &TenantAddressUpdate,
    ) -> Result<u64, AppError> {
        let result =
            sqlx::query("UPDATE tenants SET address = ?, business_scope = ? WHERE name_hash = ?")
                .bind(&tenant.address)
                .bind(&tenant.business_scope)
                .bind(hashed_name)
                .execute(&self.pool)
                .await
                .map_err(map_db_err!("Failed to update tenant"))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Tenant not found".to_string()));
        }

        Ok(result.rows_affected())
    }

    async fn get_all_providers_by_market(
        &self,
        market_hash: &str,
    ) -> Result<Vec<Provider>, AppError> {
        let providers = sqlx::query_as::<_, Provider>(
            r#"SELECT 
                    p.id, 
                    p.name,
                    p.business_scope
                FROM 
                    tenants p
                JOIN 
                    tenant_relationships tr ON p.id = tr.provider_id
                JOIN 
                    tenants m ON tr.market_id = m.id
                WHERE 
                    m.name_hash = ?
                    AND p.tenant_type = 'PROVIDER'
                    AND p.deleted_at IS NULL
                    AND tr.status = 'ACTIVE'
                ORDER BY 
                    p.name;"#,
        )
        .bind(market_hash)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get all providers by market"))?;

        Ok(providers)
    }

    async fn get_all_tenants_by_market(
        &self,
        market_hash: &str,
        query: &QueryTenantByType,
    ) -> Result<Vec<BaseTenantDTO>, AppError> {
        let mut builder: QueryBuilder<MySql> = QueryBuilder::new(
            "SELECT t.id, t.name, t.tenant_type, t.address, t.business_scope, \
             t.license_image, t.status, t.name_hash, t.created_at, t.updated_at, \
             t.verified_at, t.deleted_at \
             FROM tenants t \
             JOIN tenant_relationships tr ON t.id = tr.provider_id \
             JOIN tenants market ON tr.market_id = market.id \
             WHERE market.name_hash = ",
        );
        builder.push_bind(market_hash);
        builder.push(" AND market.tenant_type = 'MARKET'");

        if let Some(tenant_type) = &query.tenant_type {
            builder.push(" AND t.tenant_type = ");
            builder.push_bind(tenant_type.to_uppercase());
        }

        builder.push(" ORDER BY t.updated_at DESC");

        let tenants = builder
            .build_query_as::<BaseTenantDTO>()
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get all tenants by market"))?;

        Ok(tenants)
    }

    async fn delete_tenant(&self, market_hash: &str, tenant_hash: &str) -> Result<u64, AppError> {
        self.has_relationship(market_hash, tenant_hash).await?;

        let result = sqlx::query("UPDATE tenants SET deleted_at = CASE 
                                                    WHEN deleted_at IS NULL THEN NOW() ELSE NULL END WHERE name_hash = ?")
            .bind(tenant_hash)
            .execute(&self.pool)
            .await
            .map_err(map_db_err!("Failed to delete tenant"))?;

        Ok(result.rows_affected())
    }

    async fn update_tenant_by_market(
        &self,
        market_hash: &str,
        tenant_hash: &str,
        tenant: &TenantCreateDTO,
    ) -> Result<u64, AppError> {
        self.has_relationship(market_hash, tenant_hash).await?;

        let result = sqlx::query("UPDATE tenants SET name = ?, address = ?, business_scope = ?, tenant_type = ? WHERE name_hash = ?")
            .bind(&tenant.name)
            .bind(&tenant.address)
            .bind(&tenant.business_scope)
            .bind(tenant.tenant_type.to_uppercase())
            .bind(tenant_hash)
            .execute(&self.pool)
            .await
            .map_err(map_db_err!("Failed to update tenant"))?;

        Ok(result.rows_affected())
    }

    async fn has_relationship(
        &self,
        market_hash: &str,
        tenant_hash: &str,
    ) -> Result<bool, AppError> {
        let has_relationship = sqlx::query_scalar::<_, bool>(
            r#"SELECT EXISTS (SELECT 1 FROM tenant_relationships tr
                JOIN tenants market ON tr.market_id = market.id
                JOIN tenants other ON tr.provider_id = other.id
                WHERE 
                    market.name_hash = ?
                    AND other.name_hash = ?
                    AND market.tenant_type = 'MARKET') AS has_relationship;
             "#,
        )
        .bind(market_hash)
        .bind(tenant_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!("Failed to check if tenant has relationship"))?;

        if !has_relationship.unwrap_or(false) {
            return Err(AppError::Forbidden(
                "No relationship between tenants".to_string(),
            ));
        }

        Ok(true)
    }

    async fn find_tenant_detail_by_hashed_name(
        &self,
        name_hash: &str,
    ) -> Result<Option<TenantDetailDTO>, AppError> {
        // 先查询租户基本信息
        let tenant_base =
            sqlx::query_as::<_, BaseTenantDTO>("SELECT * FROM tenants WHERE name_hash = ?")
                .bind(name_hash)
                .fetch_optional(&self.pool)
                .await
                .map_err(map_db_err!("Failed to find tenant by name hash"))?;

        // 如果租户不存在，直接返回None
        if let Some(tenant) = tenant_base {
            // 创建基本的租户详情DTO
            let mut tenant_detail = TenantDetailDTO {
                id: tenant.id,
                name: tenant.name,
                address: tenant.address,
                tenant_type: tenant.tenant_type.clone(),
                name_hash: tenant.name_hash,
                business_scope: tenant.business_scope,
                license_image: tenant.license_image,
                status: tenant.status,
                created_at: tenant.created_at,
                updated_at: tenant.updated_at,
                verified_at: tenant.verified_at,
                deleted_at: tenant.deleted_at,
                financial: None,
            };

            // 使用QueryBuilder动态构建SQL，根据租户类型决定是否查询财务信息
            if tenant.tenant_type == "PROVIDER" {
                let mut query_builder: QueryBuilder<MySql> = QueryBuilder::new(
                    "SELECT * FROM provider_financial_profiles WHERE tenant_id = ",
                );
                query_builder.push_bind(tenant.id);
                query_builder.push(" AND deleted_at IS NULL ORDER BY created_at DESC LIMIT 1");

                let financial = query_builder
                    .build_query_as::<FinancialResponseDto>()
                    .fetch_optional(&self.pool)
                    .await
                    .map_err(map_db_err!("Failed to find financial info for tenant"))?;

                tenant_detail.financial = financial;
            }

            return Ok(Some(tenant_detail));
        }

        Ok(None)
    }
}
