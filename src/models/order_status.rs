use crate::common::AppError;
use std::fmt;

/// 订单状态，表示订单在整个生命周期中的不同阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum OrderStatus {
    // === Normal flows ===
    Pending,            // 客户   -创建，订单初始状态
    Assigned,           // 市场   -已分配给供应商
    SupplierPreparing,  // 供应商  -确认订单开始备货
    SupplierDelivering, // 供应商  -备货完成，开始配送到市场
    MarketInspecting,   // 市场 - 市场开始验货
    MarketAccepted,     // 市场 - 验收完成，无退换货
    MarketDelivering,   // 市场 - 开始送货
    CustomerInspecting, // 客户 - 开始验货
    Completed,          // 客户 - 完成订单，无退换货

    // === 退货流程状态 ===
    ReturnRequested, // 市场/客户 - 申请退货
    Returned,

    // === 换货流程状态 ===
    ExchangeRequested,     // 市场/客户 - 申请换货
    ExchangeInProgress,    // 供应商 - 换货进行中
    ExchangeDelivering,    // 供应商 - 换货商品配送中（到市场）
    ExchangeInspecting,    // 市场 - 检验换货商品
    ExchangeNewDelivering, // 市场 - 新商品配送中
    ExchangeCompleted,     // 换货完成

    // === 通用终态 ===
    Cancelled, // 订单取消
}

impl OrderStatus {
    fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_uppercase().as_str() {
            "PENDING" => Some(Self::Pending),
            "ASSIGNED" => Some(Self::Assigned),
            "SUPPLIER_PREPARING" => Some(Self::SupplierPreparing),
            "SUPPLIER_DELIVERING" => Some(Self::SupplierDelivering),
            "MARKET_INSPECTING" => Some(Self::MarketInspecting),
            "MARKET_ACCEPTED" => Some(Self::MarketAccepted),
            "MARKET_DELIVERING" => Some(Self::MarketDelivering),
            "CUSTOMER_INSPECTING" => Some(Self::CustomerInspecting),
            "COMPLETED" => Some(Self::Completed),
            "RETURN_REQUESTED" => Some(Self::ReturnRequested),
            "EXCHANGE_REQUESTED" => Some(Self::ExchangeRequested),
            "EXCHANGE_IN_PROGRESS" => Some(Self::ExchangeInProgress),
            "EXCHANGE_DELIVERING" => Some(Self::ExchangeDelivering),
            "EXCHANGE_INSPECTING" => Some(Self::ExchangeInspecting),
            "EXCHANGE_NEW_DELIVERING" => Some(Self::ExchangeNewDelivering),
            "EXCHANGE_COMPLETED" => Some(Self::ExchangeCompleted),
            "CANCELLED" => Some(Self::Cancelled),
            "RETURNED" => Some(Self::Returned),
            _ => None,
        }
    }

    pub(crate) fn to_str(&self) -> &'static str {
        match self {
            OrderStatus::Pending => "PENDING",
            OrderStatus::Assigned => "ASSIGNED",
            OrderStatus::SupplierPreparing => "SUPPLIER_PREPARING",
            OrderStatus::SupplierDelivering => "SUPPLIER_DELIVERING",
            OrderStatus::MarketInspecting => "MARKET_INSPECTING",
            OrderStatus::MarketAccepted => "MARKET_ACCEPTED",
            OrderStatus::MarketDelivering => "MARKET_DELIVERING",
            OrderStatus::CustomerInspecting => "CUSTOMER_INSPECTING",
            OrderStatus::ReturnRequested => "RETURN_REQUESTED",
            OrderStatus::ExchangeRequested => "EXCHANGE_REQUESTED",
            OrderStatus::ExchangeInProgress => "EXCHANGE_IN_PROGRESS",
            OrderStatus::ExchangeDelivering => "EXCHANGE_DELIVERING",
            OrderStatus::ExchangeInspecting => "EXCHANGE_INSPECTING",
            OrderStatus::ExchangeNewDelivering => "EXCHANGE_NEW_DELIVERING",
            OrderStatus::ExchangeCompleted => "EXCHANGE_COMPLETED",
            OrderStatus::Returned => "RETURNED",
            OrderStatus::Completed => "COMPLETED",
            OrderStatus::Cancelled => "CANCELLED",
        }
    }

    /// 检查状态是否为终态（不可再转移）
    pub(super) fn is_terminal(&self) -> bool {
        matches!(
            self,
            OrderStatus::Completed | OrderStatus::Cancelled | OrderStatus::ExchangeCompleted
        )
    }
}

impl TryFrom<&str> for OrderStatus {
    type Error = AppError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::parse(s).ok_or_else(|| AppError::Validation(format!("Unknown Order Status: {}", s)))
    }
}

impl fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_str_to_order_status_sucess() {
        assert_eq!(
            OrderStatus::try_from("MARKET_ACCEPTED").unwrap(),
            OrderStatus::MarketAccepted
        );
        assert_eq!(
            OrderStatus::try_from("market_accepted").unwrap(),
            OrderStatus::MarketAccepted
        );
    }

    #[test]
    fn test_str_to_order_status_failed() {
        assert!(OrderStatus::try_from("invalid").is_err());
    }

    #[test]
    fn test_order_status_description() {
        assert_eq!(
            OrderStatus::to_str(&OrderStatus::Assigned),
            "ASSIGNED"
        );
    }

    #[test]
    fn test_order_status_is_terminal() {
        assert!(OrderStatus::is_terminal(&OrderStatus::Completed));
        assert!(OrderStatus::is_terminal(&OrderStatus::Cancelled));
        assert!(OrderStatus::is_terminal(&OrderStatus::ExchangeCompleted));

        // false
        assert_eq!(OrderStatus::is_terminal(&OrderStatus::Assigned), false);
    }
}
