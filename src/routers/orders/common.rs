use axum::{routing::get, Router};

use crate::repositories::my_sql_repository::MySqlRepository;
use crate::services::orders::common::{get_exchange_and_return_order_details_by_order_code, get_order_by_order_code, get_orders};

/// 公共路由所有租户都可访问
/// 用于获取订单列表和订单详情，依据claims中的tenant_hash和tenant_type获取订单
pub(super) fn common_routes() -> Router {
    Router::new()
        .route("/", get(get_orders::<MySqlRepository>))
        .route(
            "/{order_code}",
            get(get_order_by_order_code::<MySqlRepository>),
        )  .route(
            "/{order_code}/exchange-and-return-order-details",
            get(get_exchange_and_return_order_details_by_order_code::<MySqlRepository>),
        )
}
