use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum OrderAction {
    Create,
    AssignSupplier,
    StartPreparing,
    DeliverToMarket,
    MarketInspect,
    MarketReturn,
    MarketExchange,
    MarketAccept,
    DeliverToCustomer,
    CustomerInspect,
    CustomerReturn,
    CustomerExchange,
    Complete,
    Cancel,
}

impl OrderAction {
    /// 获取操作的描述信息
    pub(crate) fn description(&self) -> &'static str {
        match self {
            OrderAction::Create => "创建订单",
            OrderAction::AssignSupplier => "分配供应商",
            OrderAction::StartPreparing => "开始备货",
            OrderAction::DeliverToMarket => "配送到市场",
            OrderAction::MarketInspect => "市场检验",
            OrderAction::MarketReturn => "市场申请退货",
            OrderAction::MarketExchange => "市场申请换货",
            OrderAction::MarketAccept => "市场签收",
            OrderAction::DeliverToCustomer => "配送到客户",
            OrderAction::CustomerInspect => "客户检验",
            OrderAction::CustomerReturn => "客户退货",
            OrderAction::CustomerExchange => "客户换货",
            OrderAction::Complete => "完成订单",
            OrderAction::Cancel => "取消订单",
        }
    }
}

impl fmt::Display for OrderAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_order_action_descriptions() {
        assert_eq!(OrderAction::AssignSupplier.description(), "分配供应商");
        assert_eq!(OrderAction::Complete.description(), "完成订单");
    }
}