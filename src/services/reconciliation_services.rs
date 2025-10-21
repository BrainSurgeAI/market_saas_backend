use axum::{extract::Path, Extension};
use chrono::{Datelike, NaiveDate};

use crate::{
    common::{ApiResponse, AppError},
    dto::reconciliation_statement::{ReconciliationStatementDTO, ReconciliationStatementOrderDTO},
    middleware::context::RequestContext,
    models::claims::Claims,
    repositories::reconciliation_statement_traits::ReconciliationStatementRepository,
    utils::validate_json_fmt::Json,
};

/// 获取指定日期所在月份的第一天和最后一天
///
/// # 参数
///
/// * `date_str` - 日期字符串，格式为"YYYY-MM-DD"
///
/// # 返回值
///
/// 返回一个元组，包含月份第一天和最后一天的字符串表示
fn get_month_range(date_str: &str) -> (String, String) {
    // 尝试解析日期字符串
    let date = match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => {
            // 如果解析失败，使用当前日期

            chrono::Local::now().date_naive()
        }
    };

    // 获取年月
    let year = date.year();
    let month = date.month();

    // 计算月份第一天
    let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();

    // 计算月份最后一天（下个月第一天减一天）
    let last_day = if month == 12 {
        // 如果是12月，则下个月是下一年的1月
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
    } else {
        // 否则是当年的下一个月
        NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
    }
    .pred_opt()
    .unwrap();

    (
        first_day.format("%Y-%m-%d").to_string(),
        last_day.format("%Y-%m-%d").to_string(),
    )
}

pub async fn create_reconciliation_statement<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
) -> Result<Json<ApiResponse<Vec<i32>>>, AppError>
where
    T: ReconciliationStatementRepository + Send + Sync,
{
    let statement_ids = repo.create_statements().await?;
    Ok(Json(ApiResponse::new(Some(statement_ids), &context)))
}

pub async fn get_statements_by_tenant_and_date<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<Vec<ReconciliationStatementDTO>>>, AppError>
where
    T: ReconciliationStatementRepository + Send + Sync,
{
    // 使用当前日期获取月份范围
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let (start_date, end_date) = get_month_range(&today);

    let statements = repo
        .get_statements_by_tenant_and_date(
            &claims.tenant_hash,
            &claims.tenant_type,
            &start_date,
            &end_date,
        )
        .await?;

    Ok(Json(ApiResponse::new(Some(statements), &context)))
}

pub async fn get_statement_orders_by_id<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<Vec<ReconciliationStatementOrderDTO>>>, AppError>
where
    T: ReconciliationStatementRepository + Send + Sync,
{
    let statement_orders = repo
        .get_statement_orders_by_id(&claims.tenant_hash, &claims.tenant_type, id)
        .await?;

    Ok(Json(ApiResponse::new(Some(statement_orders), &context)))
}
