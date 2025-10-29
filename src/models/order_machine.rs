use super::{order_action::OrderAction, order_status::OrderStatus, tenant_type::TenantType};

/// 状态转移结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TransitionResult {
    pub(crate) previous: OrderStatus,
    pub(crate) next: OrderStatus,
    pub(crate) action: OrderAction,
    pub(crate) actor: TenantType,
    pub(crate) success: bool,
}

/// 状态转移错误
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub(crate) enum TransitionError {
    #[error("无效操作: {actor} 不能在 {current_status} 状态下执行 {action}")]
    InvalidAction {
        current_status: OrderStatus,
        action: OrderAction,
        actor: TenantType,
    },

    #[error("状态已经是终态: {status}, 无法执行进一步操作")]
    TerminalState { status: OrderStatus },

    #[error("未知的租户类型")]
    UnknownTenantType,
}

/// 订单状态机
#[derive(Debug)]
pub(crate) struct OrderStateMachine;

impl OrderStateMachine {
    /// 获取下一个状态
    pub(crate) fn next_state(
        current: OrderStatus,
        action: OrderAction,
        actor: TenantType,
    ) -> Result<OrderStatus, TransitionError> {
        // 检查当前状态是否为终态
        if current.is_terminal() {
            return Err(TransitionError::TerminalState { status: current });
        }

        // 检查租户是否有权限执行该操作
        if !actor.can_perform_action(&action) {
            return Err(TransitionError::InvalidAction {
                current_status: current,
                action,
                actor,
            });
        }

        let next_status = match (current, action, actor) {
            // 市场给供应商分配订单
            (OrderStatus::Pending, OrderAction::AssignSupplier, TenantType::Market) => {
                OrderStatus::Assigned
            }

            // 供应商操作流程
            (OrderStatus::Assigned, OrderAction::StartPreparing, TenantType::Provider) => {
                OrderStatus::SupplierPreparing
            }
            (OrderStatus::SupplierPreparing, OrderAction::DeliverToMarket, TenantType::Provider,
            ) => OrderStatus::SupplierDelivering,

            // 市场正常接收流程
            (OrderStatus::SupplierDelivering, OrderAction::MarketInspect, TenantType::Market) => {
                OrderStatus::MarketInspecting
            }
            (OrderStatus::MarketInspecting, OrderAction::MarketAccept, TenantType::Market) => {
                OrderStatus::MarketAccepted
            }
            (OrderStatus::MarketAccepted, OrderAction::DeliverToCustomer, TenantType::Market) => {
                OrderStatus::MarketDelivering
            }
            (OrderStatus::MarketDelivering, OrderAction::CustomerInspect, TenantType::Customer) => {
                OrderStatus::CustomerInspecting
            }
            (OrderStatus::CustomerInspecting, OrderAction::Complete, TenantType::Customer) => {
                OrderStatus::Completed
            }

            // 市场验收请求退换货
            (OrderStatus::MarketInspecting, OrderAction::MarketReturn, TenantType::Market) => {
                OrderStatus::Returned // 市场申请退货场景暂时没有，暂时不做后续处理
            }
            (OrderStatus::MarketInspecting, OrderAction::MarketExchange, TenantType::Market) => {
                OrderStatus::ExchangeRequested
            }
            (OrderStatus::ExchangeRequested, OrderAction::StartPreparing, TenantType::Provider) => {
                OrderStatus::ExchangeInProgress
            }
            (OrderStatus::ExchangeInProgress, OrderAction::DeliverToMarket, TenantType::Provider) => {
                OrderStatus::ExchangeNewDelivering
            }

            (OrderStatus::ExchangeNewDelivering, OrderAction::MarketInspect, TenantType::Market) => {
                OrderStatus::ExchangeInspecting  
            }
            (OrderStatus::ExchangeInspecting, OrderAction::MarketAccept, TenantType::Market) => {
                OrderStatus::ExchangeCompleted  // 只有一次换货，这里直接换货完毕
            }
            (OrderStatus::ExchangeCompleted, OrderAction::DeliverToCustomer, TenantType::Market) => {
                OrderStatus::MarketDelivering
            }
            
           
            // 客户验收异常流程
            // 客户要求换货
            (OrderStatus::CustomerInspecting, OrderAction::CustomerExchange, TenantType::Customer) => {
                OrderStatus::ExchangeRequested
            }
            // (OrderStatus::ExchangeRequested, OrderAction::StartPreparing, TenantType::Market) => {
            //     OrderStatus::ExchangeInProgress
            // }
            (OrderStatus::ReturnRequested, OrderAction::CustomerReturn, TenantType::Customer) => {
                OrderStatus::Returned // 退货直接退款，无需后续服务流程
            }
            // (OrderStatus::ExchangeInProgress, OrderAction::DeliverToCustomer, TenantType::Market) => {
            //     OrderStatus::ExchangeNewDelivering
            // }
            
            // (OrderStatus::ExchangeInProgress, OrderAction::Complete, TenantType::Customer) => {
            //     OrderStatus::Completed // 一般只有一次换货过程，所以这里完成订单
            // }

            // 取消订单（只有市场和客户可在非终态都可以取消）
            (_, OrderAction::Cancel, TenantType::Customer | TenantType::Market) if !current.is_terminal() => OrderStatus::Cancelled,

            // 默认情况：无效的状态转移
            _ => {
                return Err(TransitionError::InvalidAction {
                    current_status: current,
                    action,
                    actor,
                })
            }
        };

        Ok(next_status)
    }

    /// 尝试执行状态转移，返回详细结果
    pub fn try_transition(
        current: OrderStatus,
        action: OrderAction,
        actor: TenantType,
    ) -> TransitionResult {
        match Self::next_state(current, action, actor) {
            Ok(next_status) => TransitionResult {
                previous: current,
                next: next_status,
                action,
                actor,
                success: true,
            },
            Err(_) => TransitionResult {
                previous: current,
                next: current, // 失败时保持原状态
                action,
                actor,
                success: false,
            },
        }
    }

    /// 获取当前状态下允许的所有操作
    pub fn get_available_actions(
        current: OrderStatus,
        actor: TenantType,
    ) -> Vec<(OrderAction, OrderStatus)> {
        use OrderAction::*;

        let all_actions = [
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
        ];

        all_actions
            .into_iter()
            .filter_map(|action| {
                Self::next_state(current, action, actor)
                    .ok()
                    .map(|next_status| (action, next_status))
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_valid_transition() {
        let result = OrderStateMachine::try_transition(
            OrderStatus::Pending,
            OrderAction::AssignSupplier,
            TenantType::Market,
        );
        assert!(result.success);
        assert_eq!(result.next, OrderStatus::Assigned);
    }

    #[test]
    fn test_invalid_actor() {
        let result = OrderStateMachine::try_transition(
            OrderStatus::Pending,
            OrderAction::AssignSupplier,
            TenantType::Customer,
        );
        assert!(!result.success);
    }

    #[test]
    fn test_terminal_state() {
        let result = OrderStateMachine::try_transition(
            OrderStatus::Completed,
            OrderAction::AssignSupplier,
            TenantType::Market,
        );
        assert!(!result.success);
    }
}
