use super::order_action::OrderAction;
use crate::common::AppError;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum TenantType {
    Customer,
    Provider,
    Market,
}

impl TenantType {
    /// 从字符串解析租户类型，支持多种格式
    fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_uppercase().as_str() {
            "CUSTOMER" => Some(Self::Customer),
            "PROVIDER" | "SUPPLIER" => Some(Self::Provider), // 支持 Provider 和 Supplier 别名
            "MARKET" => Some(Self::Market),
            _ => None,
        }
    }

    /// 检查该租户类型是否允许执行特定操作
    pub(crate) fn can_perform_action(&self, action: &OrderAction) -> bool {
        match self {
            TenantType::Customer => matches!(
                action,
                OrderAction::CustomerInspect
                    | OrderAction::CustomerReturn
                    | OrderAction::Complete
                    | OrderAction::CustomerExchange
                    | OrderAction::Cancel
                    | OrderAction::Create
            ),
            TenantType::Provider => matches!(
                action,
                OrderAction::StartPreparing | OrderAction::DeliverToMarket
            ),
            TenantType::Market => matches!(
                action,
                OrderAction::AssignSupplier
                    | OrderAction::MarketInspect
                    | OrderAction::MarketReturn
                    | OrderAction::MarketAccept
                    | OrderAction::MarketExchange
                    | OrderAction::DeliverToCustomer
                    | OrderAction::StartPreparing
                    | OrderAction::Cancel
            ),
        }
    }
}

impl fmt::Display for TenantType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display_str = match self {
            TenantType::Customer => "CUSTOMER",
            TenantType::Provider => "PROVIDER",
            TenantType::Market => "MARKET",
        };
        write!(f, "{}", display_str)
    }
}

impl TryFrom<&str> for TenantType {
    type Error = AppError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::parse(s).ok_or_else(|| AppError::Validation("Unknown tenant type".to_string()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_invalid_str_to_tenant_type() {
        assert!(TenantType::try_from("abc").is_err());
        assert!(TenantType::try_from("supplier").is_ok());
    }

    #[test]
    fn test_valid_str_to_tenant_type() {
        assert!(TenantType::try_from("PROVIDER").is_ok());
        assert!(TenantType::try_from("provider").is_ok());
        assert!(TenantType::try_from("SUPPLIER").is_ok());
        assert!(TenantType::try_from("supplier").is_ok());
    }

    #[test]
    fn test_perform_action_success() {
        assert!(TenantType::can_perform_action(
            &TenantType::Customer,
            &OrderAction::Complete
        ));
    }

    #[test]
    fn test_perform_action_failed() {
        assert!(!TenantType::can_perform_action(
            &TenantType::Market,
            &OrderAction::Complete
        ));
    }
}
