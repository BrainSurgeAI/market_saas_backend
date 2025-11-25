use super::{order_action::OrderAction, order_status::OrderStatus, tenant_type::TenantType};

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
    // #[error("未知的租户类型")]
    // UnknownTenantType,
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
            (
                OrderStatus::SupplierPreparing,
                OrderAction::DeliverToMarket,
                TenantType::Provider,
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
            (
                OrderStatus::ExchangeInProgress,
                OrderAction::DeliverToMarket,
                TenantType::Provider,
            ) => OrderStatus::ExchangeDelivering,

            (OrderStatus::ExchangeDelivering, OrderAction::MarketInspect, TenantType::Market) => {
                OrderStatus::ExchangeInspecting
            }
            (OrderStatus::ExchangeInspecting, OrderAction::MarketAccept, TenantType::Market) => {
                OrderStatus::ExchangeNewDelivering
            }
            (OrderStatus::ExchangeNewDelivering, OrderAction::CustomerInspect, TenantType::Customer) => {
                OrderStatus::CustomerInspecting
            }

            // 客户验收异常流程
            // 客户要求换货
            (
                OrderStatus::CustomerInspecting,
                OrderAction::CustomerExchange,
                TenantType::Customer,
            ) => OrderStatus::ExchangeRequested,
            (
                OrderStatus::CustomerInspecting,
                OrderAction::CustomerReturn,
                TenantType::Customer,
            ) => OrderStatus::ReturnRequested,
            (
                OrderStatus::CustomerInspecting,
                OrderAction::CustomerInspect,
                TenantType::Customer,
            ) => OrderStatus::ExchangeCompleted,
            (OrderStatus::ReturnRequested, OrderAction::MarketReturn, TenantType::Market) => {
                OrderStatus::Returned
            }
            (OrderStatus::ReturnRequested, OrderAction::CustomerReturn, TenantType::Customer) => {
                OrderStatus::Returned // 退货直接退款，无需后续服务流程
            }

            // 取消订单（只有市场和客户可在非终态都可以取消）
            (_, OrderAction::Cancel, TenantType::Customer | TenantType::Market)
                if current == OrderStatus::Pending || current == OrderStatus::Assigned =>
            {
                OrderStatus::Cancelled
            }

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
}

#[cfg(test)]
mod test {
    use super::*;

    /// 状态转移结果
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct TransitionResult {
        pub previous: OrderStatus,
        pub next: OrderStatus,
        pub action: OrderAction,
        pub actor: TenantType,
        pub success: bool,
    }
    // 尝试执行状态转移，返回详细结果
    pub fn try_transition(
        current: OrderStatus,
        action: OrderAction,
        actor: TenantType,
    ) -> TransitionResult {
        match OrderStateMachine::next_state(current, action, actor) {
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
                OrderStateMachine::next_state(current, action, actor)
                    .ok()
                    .map(|next_status| (action, next_status))
            })
            .collect()
    }
    #[test]
    fn test_valid_transition() {
        let result = try_transition(
            OrderStatus::Pending,
            OrderAction::AssignSupplier,
            TenantType::Market,
        );
        assert!(result.success);
        assert_eq!(result.next, OrderStatus::Assigned);
    }

    #[test]
    fn test_invalid_actor() {
        let result = try_transition(
            OrderStatus::Pending,
            OrderAction::AssignSupplier,
            TenantType::Customer,
        );
        assert!(!result.success);
    }

    #[test]
    fn test_terminal_state() {
        let result = try_transition(
            OrderStatus::Completed,
            OrderAction::AssignSupplier,
            TenantType::Market,
        );
        assert!(!result.success);
    }

    // ==================== 基础状态转移测试 ====================

    #[test]
    fn test_order_assign_supplier() {
        let result = try_transition(
            OrderStatus::Pending,
            OrderAction::AssignSupplier,
            TenantType::Market,
        );
        assert!(result.success);
        assert_eq!(result.previous, OrderStatus::Pending);
        assert_eq!(result.next, OrderStatus::Assigned);
        assert_eq!(result.action, OrderAction::AssignSupplier);
        assert_eq!(result.actor, TenantType::Market);
    }

    #[test]
    fn test_provider_start_preparing() {
        let result = try_transition(
            OrderStatus::Assigned,
            OrderAction::StartPreparing,
            TenantType::Provider,
        );
        assert!(result.success);
        assert_eq!(result.next, OrderStatus::SupplierPreparing);
    }

    #[test]
    fn test_provider_deliver_to_market() {
        let result = try_transition(
            OrderStatus::SupplierPreparing,
            OrderAction::DeliverToMarket,
            TenantType::Provider,
        );
        assert!(result.success);
        assert_eq!(result.next, OrderStatus::SupplierDelivering);
    }

    #[test]
    fn test_market_inspect() {
        let result = try_transition(
            OrderStatus::SupplierDelivering,
            OrderAction::MarketInspect,
            TenantType::Market,
        );
        assert!(result.success);
        assert_eq!(result.next, OrderStatus::MarketInspecting);
    }

    #[test]
    fn test_market_accept() {
        let result = try_transition(
            OrderStatus::MarketInspecting,
            OrderAction::MarketAccept,
            TenantType::Market,
        );
        assert!(result.success);
        assert_eq!(result.next, OrderStatus::MarketAccepted);
    }

    #[test]
    fn test_market_deliver_to_customer() {
        let result = try_transition(
            OrderStatus::MarketAccepted,
            OrderAction::DeliverToCustomer,
            TenantType::Market,
        );
        assert!(result.success);
        assert_eq!(result.next, OrderStatus::MarketDelivering);
    }

    #[test]
    fn test_customer_inspect() {
        let result = try_transition(
            OrderStatus::MarketDelivering,
            OrderAction::CustomerInspect,
            TenantType::Customer,
        );
        assert!(result.success);
        assert_eq!(result.next, OrderStatus::CustomerInspecting);
    }

    #[test]
    fn test_customer_complete() {
        let result = try_transition(
            OrderStatus::CustomerInspecting,
            OrderAction::Complete,
            TenantType::Customer,
        );
        assert!(result.success);
        assert_eq!(result.next, OrderStatus::Completed);
    }

    // ==================== 换货流程测试 ====================

    #[test]
    fn test_market_exchange_request() {
        let result = try_transition(
            OrderStatus::MarketInspecting,
            OrderAction::MarketExchange,
            TenantType::Market,
        );
        assert!(result.success);
        assert_eq!(result.next, OrderStatus::ExchangeRequested);
    }

    #[test]
    fn test_provider_start_exchange_preparing() {
        let result = try_transition(
            OrderStatus::ExchangeRequested,
            OrderAction::StartPreparing,
            TenantType::Provider,
        );
        assert!(result.success);
        assert_eq!(result.next, OrderStatus::ExchangeInProgress);
    }

    #[test]
    fn test_provider_deliver_exchange_to_market() {
        let result = try_transition(
            OrderStatus::ExchangeInProgress,
            OrderAction::DeliverToMarket,
            TenantType::Provider,
        );
        assert!(result.success);
        assert_eq!(result.next, OrderStatus::ExchangeDelivering);
    }

    #[test]
    fn test_market_inspect_exchange() {
        let result = try_transition(
            OrderStatus::ExchangeDelivering,
            OrderAction::MarketInspect,
            TenantType::Market,
        );
        assert!(result.success);
        assert_eq!(result.next, OrderStatus::ExchangeInspecting);
    }

    #[test]
    fn test_market_accept_exchange() {
        let result = try_transition(
            OrderStatus::ExchangeInspecting,
            OrderAction::MarketAccept,
            TenantType::Market,
        );
        assert!(result.success);
        assert_eq!(result.next, OrderStatus::ExchangeNewDelivering);
    }

    #[test]
    fn test_market_deliver_exchange_to_customer() {
        let result = try_transition(
            OrderStatus::ExchangeNewDelivering,
            OrderAction::DeliverToCustomer,
            TenantType::Market,
        );
        assert!(result.success);
        assert_eq!(result.next, OrderStatus::CustomerInspecting);
    }

    #[test]
    fn test_customer_inspect_exchange_completion() {
        let result = try_transition(
            OrderStatus::CustomerInspecting,
            OrderAction::CustomerInspect,
            TenantType::Customer,
        );
        assert!(result.success);
        assert_eq!(result.next, OrderStatus::ExchangeCompleted);
    }

    #[test]
    fn test_exchange_completed_is_terminal() {
        // ExchangeCompleted 是终态，不能进一步操作
        let result = try_transition(
            OrderStatus::ExchangeCompleted,
            OrderAction::DeliverToCustomer,
            TenantType::Market,
        );

        // 应该失败，因为 ExchangeCompleted 是终态
        assert!(!result.success);
        assert_eq!(result.next, OrderStatus::ExchangeCompleted); // 状态保持不变
    }

    #[test]
    fn test_customer_exchange_request() {
        let result = try_transition(
            OrderStatus::CustomerInspecting,
            OrderAction::CustomerExchange,
            TenantType::Customer,
        );
        assert!(result.success);
        assert_eq!(result.next, OrderStatus::ExchangeRequested);
    }

    // ==================== 退货流程测试 ====================

    #[test]
    fn test_market_return() {
        let result = try_transition(
            OrderStatus::MarketInspecting,
            OrderAction::MarketReturn,
            TenantType::Market,
        );
        assert!(result.success);
        assert_eq!(result.next, OrderStatus::Returned);
    }

    #[test]
    fn test_customer_return_request_sets_return_requested() {
        let result = try_transition(
            OrderStatus::CustomerInspecting,
            OrderAction::CustomerReturn,
            TenantType::Customer,
        );
        assert!(result.success);
        assert_eq!(result.next, OrderStatus::ReturnRequested);
    }

    #[test]
    fn test_market_return_after_request() {
        let result = try_transition(
            OrderStatus::ReturnRequested,
            OrderAction::MarketReturn,
            TenantType::Market,
        );
        assert!(result.success);
        assert_eq!(result.next, OrderStatus::Returned);
    }

    #[test]
    fn test_customer_finalize_return_after_request() {
        let result = try_transition(
            OrderStatus::ReturnRequested,
            OrderAction::CustomerReturn,
            TenantType::Customer,
        );
        assert!(result.success);
        assert_eq!(result.next, OrderStatus::Returned);
    }

    #[test]
    fn test_customer_return() {
        let result = try_transition(
            OrderStatus::ReturnRequested,
            OrderAction::CustomerReturn,
            TenantType::Customer,
        );
        assert!(result.success);
        assert_eq!(result.next, OrderStatus::Returned);
    }

    // ==================== 取消订单测试 ====================

    #[test]
    fn test_customer_cancel_pending_order() {
        let result = try_transition(
            OrderStatus::Pending,
            OrderAction::Cancel,
            TenantType::Customer,
        );
        assert!(result.success);
        assert_eq!(result.next, OrderStatus::Cancelled);
    }

    #[test]
    fn test_market_can_cancel_assigned_order() {
        // Market 可以在 Assigned 状态下取消订单
        let result = try_transition(
            OrderStatus::Assigned,
            OrderAction::Cancel,
            TenantType::Market,
        );
        assert!(result.success);
        assert_eq!(result.next, OrderStatus::Cancelled);
    }

    #[test]
    fn test_customer_can_cancel_assigned_order() {
        // Customer 可以在 Assigned 状态下取消订单
        let result = try_transition(
            OrderStatus::Assigned,
            OrderAction::Cancel,
            TenantType::Customer,
        );
        assert!(result.success);
        assert_eq!(result.next, OrderStatus::Cancelled);
    }

    // ==================== 无效状态转移测试 ====================

    #[test]
    fn test_invalid_customer_assign_supplier() {
        let result = try_transition(
            OrderStatus::Pending,
            OrderAction::AssignSupplier,
            TenantType::Customer,
        );
        assert!(!result.success);
        assert_eq!(result.next, OrderStatus::Pending); // 状态不变
    }

    #[test]
    fn test_invalid_provider_assign_supplier() {
        let result = try_transition(
            OrderStatus::Pending,
            OrderAction::AssignSupplier,
            TenantType::Provider,
        );
        assert!(!result.success);
        assert_eq!(result.next, OrderStatus::Pending);
    }

    #[test]
    fn test_invalid_market_start_preparing() {
        let result = try_transition(
            OrderStatus::Assigned,
            OrderAction::StartPreparing,
            TenantType::Market,
        );
        assert!(!result.success);
        assert_eq!(result.next, OrderStatus::Assigned);
    }

    #[test]
    fn test_invalid_customer_start_preparing() {
        let result = try_transition(
            OrderStatus::Assigned,
            OrderAction::StartPreparing,
            TenantType::Customer,
        );
        assert!(!result.success);
        assert_eq!(result.next, OrderStatus::Assigned);
    }

    #[test]
    fn test_invalid_wrong_status_transition() {
        let result = try_transition(
            OrderStatus::Pending,
            OrderAction::StartPreparing,
            TenantType::Provider,
        );
        assert!(!result.success);
        assert_eq!(result.next, OrderStatus::Pending);
    }

    // ==================== 终态操作测试 ====================

    #[test]
    fn test_cannot_cancel_completed_order() {
        let result = try_transition(
            OrderStatus::Completed,
            OrderAction::Cancel,
            TenantType::Customer,
        );
        assert!(!result.success);
    }

    #[test]
    fn test_cannot_cancel_cancelled_order() {
        let result = try_transition(
            OrderStatus::Cancelled,
            OrderAction::Cancel,
            TenantType::Market,
        );
        assert!(!result.success);
    }

    #[test]
    fn test_cannot_operate_on_returned_order() {
        let result = try_transition(
            OrderStatus::Returned,
            OrderAction::Complete,
            TenantType::Customer,
        );
        assert!(!result.success);
    }

    // ==================== 错误类型测试 ====================

    #[test]
    fn test_transition_error_invalid_action() {
        let result = OrderStateMachine::next_state(
            OrderStatus::Pending,
            OrderAction::AssignSupplier,
            TenantType::Customer,
        );
        assert!(result.is_err());

        match result.unwrap_err() {
            TransitionError::InvalidAction {
                current_status,
                action,
                actor,
            } => {
                assert_eq!(current_status, OrderStatus::Pending);
                assert_eq!(action, OrderAction::AssignSupplier);
                assert_eq!(actor, TenantType::Customer);
            }
            _ => panic!("Expected InvalidAction error"),
        }
    }

    #[test]
    fn test_transition_error_terminal_state() {
        let result = OrderStateMachine::next_state(
            OrderStatus::Completed,
            OrderAction::Cancel,
            TenantType::Customer,
        );
        assert!(result.is_err());

        match result.unwrap_err() {
            TransitionError::TerminalState { status } => {
                assert_eq!(status, OrderStatus::Completed);
            }
            _ => panic!("Expected TerminalState error"),
        }
    }

    // ==================== 完整业务流程测试 ====================

    #[test]
    fn test_complete_normal_workflow() {
        let mut current_status = OrderStatus::Pending;

        // 市场分配订单
        let result = try_transition(
            current_status,
            OrderAction::AssignSupplier,
            TenantType::Market,
        );
        assert!(result.success);
        current_status = result.next;
        assert_eq!(current_status, OrderStatus::Assigned);

        // 供应商开始准备
        let result = try_transition(
            current_status,
            OrderAction::StartPreparing,
            TenantType::Provider,
        );
        assert!(result.success);
        current_status = result.next;
        assert_eq!(current_status, OrderStatus::SupplierPreparing);

        // 供应商配送至市场
        let result = try_transition(
            current_status,
            OrderAction::DeliverToMarket,
            TenantType::Provider,
        );
        assert!(result.success);
        current_status = result.next;
        assert_eq!(current_status, OrderStatus::SupplierDelivering);

        // 市场验收
        let result = try_transition(
            current_status,
            OrderAction::MarketInspect,
            TenantType::Market,
        );
        assert!(result.success);
        current_status = result.next;
        assert_eq!(current_status, OrderStatus::MarketInspecting);

        // 市场接受
        let result = try_transition(
            current_status,
            OrderAction::MarketAccept,
            TenantType::Market,
        );
        assert!(result.success);
        current_status = result.next;
        assert_eq!(current_status, OrderStatus::MarketAccepted);

        // 配送给客户
        let result = try_transition(
            current_status,
            OrderAction::DeliverToCustomer,
            TenantType::Market,
        );
        assert!(result.success);
        current_status = result.next;
        assert_eq!(current_status, OrderStatus::MarketDelivering);

        // 客户验收
        let result = try_transition(
            current_status,
            OrderAction::CustomerInspect,
            TenantType::Customer,
        );
        assert!(result.success);
        current_status = result.next;
        assert_eq!(current_status, OrderStatus::CustomerInspecting);

        // 订单完成
        let result = try_transition(current_status, OrderAction::Complete, TenantType::Customer);
        assert!(result.success);
        current_status = result.next;
        assert_eq!(current_status, OrderStatus::Completed);
    }

    #[test]
    fn test_complete_exchange_workflow() {
        let mut current_status = OrderStatus::MarketInspecting;

        // 市场申请换货
        let result = try_transition(
            current_status,
            OrderAction::MarketExchange,
            TenantType::Market,
        );
        assert!(result.success);
        current_status = result.next;
        assert_eq!(current_status, OrderStatus::ExchangeRequested);

        // 供应商准备换货
        let result = try_transition(
            current_status,
            OrderAction::StartPreparing,
            TenantType::Provider,
        );
        assert!(result.success);
        current_status = result.next;
        assert_eq!(current_status, OrderStatus::ExchangeInProgress);

        // 供应商配送换货商品
        let result = try_transition(
            current_status,
            OrderAction::DeliverToMarket,
            TenantType::Provider,
        );
        assert!(result.success);
        current_status = result.next;
        assert_eq!(current_status, OrderStatus::ExchangeDelivering);

        // 市场验收换货商品
        let result = try_transition(
            current_status,
            OrderAction::MarketInspect,
            TenantType::Market,
        );
        assert!(result.success);
        current_status = result.next;
        assert_eq!(current_status, OrderStatus::ExchangeInspecting);

        // 市场接受换货
        let result = try_transition(
            current_status,
            OrderAction::MarketAccept,
            TenantType::Market,
        );
        assert!(result.success);
        current_status = result.next;
        assert_eq!(current_status, OrderStatus::ExchangeNewDelivering);

        // 市场将换货商品送达客户
        let result = try_transition(
            current_status,
            OrderAction::DeliverToCustomer,
            TenantType::Market,
        );
        assert!(result.success);
        current_status = result.next;
        assert_eq!(current_status, OrderStatus::CustomerInspecting);

        // 客户验收换货商品
        let result = try_transition(
            current_status,
            OrderAction::CustomerInspect,
            TenantType::Customer,
        );
        assert!(result.success);
        current_status = result.next;
        assert_eq!(current_status, OrderStatus::ExchangeCompleted);

        // ExchangeCompleted 是终态，流程结束
        assert_eq!(current_status, OrderStatus::ExchangeCompleted);
    }

    // ==================== get_available_actions 测试 ====================

    #[test]
    fn test_available_actions_for_pending_state() {
        let actions = get_available_actions(OrderStatus::Pending, TenantType::Market);

        // Market 在 Pending 状态可以分配订单或取消
        assert_eq!(actions.len(), 2);
        assert!(actions
            .iter()
            .any(|(action, _)| *action == OrderAction::AssignSupplier));
        assert!(actions
            .iter()
            .any(|(action, _)| *action == OrderAction::Cancel));
    }

    #[test]
    fn test_available_actions_for_customer_pending() {
        let actions = get_available_actions(OrderStatus::Pending, TenantType::Customer);

        // 客户只能取消订单
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].0, OrderAction::Cancel);
        assert_eq!(actions[0].1, OrderStatus::Cancelled);
    }

    #[test]
    fn test_available_actions_for_market_pending_cancel() {
        let actions = get_available_actions(OrderStatus::Pending, TenantType::Market);

        // Market 可以取消 Pending 订单
        assert!(actions
            .iter()
            .any(|(action, next_status)| *action == OrderAction::Cancel
                && *next_status == OrderStatus::Cancelled));
    }

    #[test]
    fn test_available_actions_for_provider_assigned() {
        let actions = get_available_actions(OrderStatus::Assigned, TenantType::Provider);

        // 供应商可以开始准备
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].0, OrderAction::StartPreparing);
        assert_eq!(actions[0].1, OrderStatus::SupplierPreparing);
    }

    #[test]
    fn test_available_actions_for_market_inspecting() {
        let actions = get_available_actions(OrderStatus::MarketInspecting, TenantType::Market);

        // 市场可以接受、退货、换货（不能取消订单，因为不在 Pending 或 Assigned 状态）
        assert_eq!(actions.len(), 3);
        assert!(actions
            .iter()
            .any(|(action, _)| *action == OrderAction::MarketAccept));
        assert!(actions
            .iter()
            .any(|(action, _)| *action == OrderAction::MarketReturn));
        assert!(actions
            .iter()
            .any(|(action, _)| *action == OrderAction::MarketExchange));
        assert!(!actions
            .iter()
            .any(|(action, _)| *action == OrderAction::Cancel));
    }

    #[test]
    fn test_available_actions_for_customer_inspecting() {
        let actions = get_available_actions(OrderStatus::CustomerInspecting, TenantType::Customer);

        // 客户可以完成、换货、验收换货、退货（不能取消订单，因为不在 Pending 或 Assigned 状态）
        assert_eq!(actions.len(), 4);
        assert!(actions
            .iter()
            .any(|(action, _)| *action == OrderAction::Complete));
        assert!(actions
            .iter()
            .any(|(action, _)| *action == OrderAction::CustomerInspect));
        assert!(actions
            .iter()
            .any(|(action, _)| *action == OrderAction::CustomerExchange));
        assert!(actions
            .iter()
            .any(|(action, _)| *action == OrderAction::CustomerReturn));
        assert!(!actions
            .iter()
            .any(|(action, _)| *action == OrderAction::Cancel));
    }

    #[test]
    fn test_available_actions_for_terminal_states() {
        // 已完成订单
        let actions = get_available_actions(OrderStatus::Completed, TenantType::Customer);
        assert_eq!(actions.len(), 0);

        // 已取消订单
        let actions = get_available_actions(OrderStatus::Cancelled, TenantType::Market);
        assert_eq!(actions.len(), 0);

        // 已退货订单
        let actions = get_available_actions(OrderStatus::Returned, TenantType::Provider);
        assert_eq!(actions.len(), 0);
    }

    // ==================== 边界情况测试 ====================

    #[test]
    fn test_provider_cannot_cancel_order() {
        let actions = get_available_actions(OrderStatus::Assigned, TenantType::Provider);

        // 供应商不能取消订单
        assert!(!actions
            .iter()
            .any(|(action, _)| *action == OrderAction::Cancel));
    }

    #[test]
    fn test_market_cannot_complete_order() {
        let actions = get_available_actions(OrderStatus::CustomerInspecting, TenantType::Market);

        // 市场不能完成订单（只有客户可以）
        assert!(!actions
            .iter()
            .any(|(action, _)| *action == OrderAction::Complete));
    }

    #[test]
    fn test_all_actions_for_exchange_status() {
        let actions = get_available_actions(OrderStatus::ExchangeRequested, TenantType::Provider);

        // 供应商可以开始准备换货
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].0, OrderAction::StartPreparing);
        assert_eq!(actions[0].1, OrderStatus::ExchangeInProgress);
    }

    #[test]
    fn test_cannot_cancel_from_non_allowed_states() {
        // 测试所有不能取消订单的状态
        let non_cancellable_states = [
            OrderStatus::SupplierPreparing,
            OrderStatus::SupplierDelivering,
            OrderStatus::MarketInspecting,
            OrderStatus::MarketAccepted,
            OrderStatus::MarketDelivering,
            OrderStatus::CustomerInspecting,
            OrderStatus::ExchangeRequested,
            OrderStatus::ExchangeInProgress,
            OrderStatus::ExchangeDelivering,
            OrderStatus::ExchangeInspecting,
            OrderStatus::ExchangeNewDelivering,
            OrderStatus::ReturnRequested,
        ];

        for status in &non_cancellable_states {
            // Market 不能取消
            let result = try_transition(*status, OrderAction::Cancel, TenantType::Market);
            assert!(
                !result.success,
                "Market should not be able to cancel order in {:?} state",
                status
            );

            // Customer 不能取消
            let result = try_transition(*status, OrderAction::Cancel, TenantType::Customer);
            assert!(
                !result.success,
                "Customer should not be able to cancel order in {:?} state",
                status
            );
        }
    }

    #[test]
    fn test_can_cancel_from_allowed_states() {
        // 测试所有可以取消订单的状态
        let cancellable_states = [OrderStatus::Pending, OrderStatus::Assigned];

        for status in &cancellable_states {
            // Market 可以取消
            let result = try_transition(*status, OrderAction::Cancel, TenantType::Market);
            assert!(
                result.success,
                "Market should be able to cancel order in {:?} state",
                status
            );
            assert_eq!(result.next, OrderStatus::Cancelled);

            // Customer 可以取消
            let result = try_transition(*status, OrderAction::Cancel, TenantType::Customer);
            assert!(
                result.success,
                "Customer should be able to cancel order in {:?} state",
                status
            );
            assert_eq!(result.next, OrderStatus::Cancelled);
        }
    }

    #[test]
    fn test_market_cannot_cancel_from_supplier_preparing() {
        // Market 不能在 SupplierPreparing 状态下取消订单
        let result = try_transition(
            OrderStatus::SupplierPreparing,
            OrderAction::Cancel,
            TenantType::Market,
        );
        assert!(!result.success);
    }

    #[test]
    fn test_market_cannot_cancel_from_supplier_delivering() {
        // Market 不能在 SupplierDelivering 状态下取消订单
        let result = try_transition(
            OrderStatus::SupplierDelivering,
            OrderAction::Cancel,
            TenantType::Market,
        );
        assert!(!result.success);
    }

    #[test]
    fn test_customer_cannot_cancel_from_market_delivering() {
        // Customer 不能在 MarketDelivering 状态下取消订单
        let result = try_transition(
            OrderStatus::MarketDelivering,
            OrderAction::Cancel,
            TenantType::Customer,
        );
        assert!(!result.success);
    }

    #[test]
    fn test_market_cannot_cancel_from_market_inspecting() {
        // Market 不能在 MarketInspecting 状态下取消订单
        let result = try_transition(
            OrderStatus::MarketInspecting,
            OrderAction::Cancel,
            TenantType::Market,
        );
        assert!(!result.success);
    }

    #[test]
    fn test_customer_cannot_cancel_from_customer_inspecting() {
        // Customer 不能在 CustomerInspecting 状态下取消订单
        let result = try_transition(
            OrderStatus::CustomerInspecting,
            OrderAction::Cancel,
            TenantType::Customer,
        );
        assert!(!result.success);
    }

    // ==================== 权限验证测试 ====================

    #[test]
    fn test_customer_cannot_assign_supplier() {
        let result = try_transition(
            OrderStatus::Pending,
            OrderAction::AssignSupplier,
            TenantType::Customer,
        );
        assert!(!result.success);
    }

    #[test]
    fn test_provider_cannot_market_inspect() {
        let result = try_transition(
            OrderStatus::SupplierDelivering,
            OrderAction::MarketInspect,
            TenantType::Provider,
        );
        assert!(!result.success);
    }

    #[test]
    fn test_customer_cannot_market_inspect() {
        let result = try_transition(
            OrderStatus::SupplierDelivering,
            OrderAction::MarketInspect,
            TenantType::Customer,
        );
        assert!(!result.success);
    }

    #[test]
    fn test_provider_cannot_deliver_to_customer() {
        let result = try_transition(
            OrderStatus::MarketAccepted,
            OrderAction::DeliverToCustomer,
            TenantType::Provider,
        );
        assert!(!result.success);
    }

    #[test]
    fn test_market_cannot_customer_inspect() {
        let result = try_transition(
            OrderStatus::MarketDelivering,
            OrderAction::CustomerInspect,
            TenantType::Market,
        );
        assert!(!result.success);
    }

    #[test]
    fn test_market_cannot_customer_exchange() {
        let result = try_transition(
            OrderStatus::CustomerInspecting,
            OrderAction::CustomerExchange,
            TenantType::Market,
        );
        assert!(!result.success);
    }

    #[test]
    fn test_provider_cannot_deliver_to_customer_from_preparing() {
        let result = try_transition(
            OrderStatus::SupplierPreparing,
            OrderAction::DeliverToCustomer,
            TenantType::Provider,
        );
        assert!(!result.success);
    }

    // ==================== 状态转移边界测试 ====================

    #[test]
    fn test_cannot_transition_from_returned_state() {
        let result = try_transition(
            OrderStatus::Returned,
            OrderAction::Complete,
            TenantType::Customer,
        );
        assert!(!result.success); // 终态不能转移
    }

    #[test]
    fn test_cannot_transition_from_cancelled_state() {
        let result = try_transition(
            OrderStatus::Cancelled,
            OrderAction::DeliverToCustomer,
            TenantType::Market,
        );
        assert!(!result.success); // 终态不能转移
    }

    #[test]
    fn test_cannot_transition_from_completed_state() {
        let result = try_transition(
            OrderStatus::Completed,
            OrderAction::CustomerReturn,
            TenantType::Customer,
        );
        assert!(!result.success); // 终态不能转移
    }

    #[test]
    fn test_cannot_transition_from_exchange_completed() {
        let result = try_transition(
            OrderStatus::ExchangeCompleted,
            OrderAction::DeliverToCustomer,
            TenantType::Market,
        );
        assert!(!result.success); // 终态不能转移
    }

    // ==================== 完整退货工作流测试 ====================

    #[test]
    fn test_complete_market_return_workflow() {
        let mut current_status = OrderStatus::SupplierDelivering;

        // 市场验收
        let result = try_transition(
            current_status,
            OrderAction::MarketInspect,
            TenantType::Market,
        );
        assert!(result.success);
        current_status = result.next;
        assert_eq!(current_status, OrderStatus::MarketInspecting);

        // 市场申请退货
        let result = try_transition(
            current_status,
            OrderAction::MarketReturn,
            TenantType::Market,
        );
        assert!(result.success);
        current_status = result.next;
        assert_eq!(current_status, OrderStatus::Returned);

        // 退货是终态，无法继续操作
        let result = try_transition(
            current_status,
            OrderAction::DeliverToCustomer,
            TenantType::Market,
        );
        assert!(!result.success);
    }

    #[test]
    fn test_complete_customer_return_workflow() {
        let mut current_status = OrderStatus::CustomerInspecting;

        // 客户申请退货，进入退货申请状态
        let result = try_transition(
            current_status,
            OrderAction::CustomerReturn,
            TenantType::Customer,
        );
        assert!(result.success);
        current_status = result.next;
        assert_eq!(current_status, OrderStatus::ReturnRequested);

        // 客户确认退货，订单进入退货完成状态
        let result = try_transition(
            current_status,
            OrderAction::CustomerReturn,
            TenantType::Customer,
        );
        assert!(result.success);
        current_status = result.next;
        assert_eq!(current_status, OrderStatus::Returned);
    }

    // ==================== 完整取消工作流测试 ====================

    #[test]
    fn test_cancel_at_various_stages() {
        // Pending 阶段取消（Customer）
        let result = try_transition(
            OrderStatus::Pending,
            OrderAction::Cancel,
            TenantType::Customer,
        );
        assert!(result.success);
        assert_eq!(result.next, OrderStatus::Cancelled);

        // Pending 阶段取消（Market）
        let result = try_transition(
            OrderStatus::Pending,
            OrderAction::Cancel,
            TenantType::Market,
        );
        assert!(result.success);
        assert_eq!(result.next, OrderStatus::Cancelled);

        // Assigned 阶段取消（Market）
        let result = try_transition(
            OrderStatus::Assigned,
            OrderAction::Cancel,
            TenantType::Market,
        );
        assert!(result.success);
        assert_eq!(result.next, OrderStatus::Cancelled);

        // Assigned 阶段取消（Customer）
        let result = try_transition(
            OrderStatus::Assigned,
            OrderAction::Cancel,
            TenantType::Customer,
        );
        assert!(result.success);
        assert_eq!(result.next, OrderStatus::Cancelled);

        // SupplierPreparing 阶段不能取消
        let result = try_transition(
            OrderStatus::SupplierPreparing,
            OrderAction::Cancel,
            TenantType::Market,
        );
        assert!(!result.success);

        // MarketInspecting 阶段不能取消
        let result = try_transition(
            OrderStatus::MarketInspecting,
            OrderAction::Cancel,
            TenantType::Customer,
        );
        assert!(!result.success);
    }

    // ==================== 所有租户操作权限矩阵测试 ====================

    #[test]
    fn test_provider_capabilities() {
        let provider_actions = [
            (OrderStatus::Assigned, OrderAction::StartPreparing),
            (OrderStatus::SupplierPreparing, OrderAction::DeliverToMarket),
            (OrderStatus::ExchangeRequested, OrderAction::StartPreparing),
            (
                OrderStatus::ExchangeInProgress,
                OrderAction::DeliverToMarket,
            ),
        ];

        for (status, action) in &provider_actions {
            let result = try_transition(*status, *action, TenantType::Provider);
            assert!(
                result.success,
                "Provider should be able to perform {:?} in status {:?}",
                action, status
            );
        }
    }

    #[test]
    fn test_market_capabilities() {
        let market_actions = [
            (OrderStatus::Pending, OrderAction::AssignSupplier),
            (OrderStatus::Pending, OrderAction::Cancel),
            (OrderStatus::Assigned, OrderAction::Cancel),
            (OrderStatus::SupplierDelivering, OrderAction::MarketInspect),
            (OrderStatus::MarketInspecting, OrderAction::MarketAccept),
            (OrderStatus::MarketAccepted, OrderAction::DeliverToCustomer),
            (OrderStatus::MarketInspecting, OrderAction::MarketReturn),
            (OrderStatus::MarketInspecting, OrderAction::MarketExchange),
        ];

        for (status, action) in &market_actions {
            let result = try_transition(*status, *action, TenantType::Market);
            assert!(
                result.success,
                "Market should be able to perform {:?} in status {:?}",
                action, status
            );
        }
    }

    #[test]
    fn test_customer_capabilities() {
        let customer_actions = [
            (OrderStatus::MarketDelivering, OrderAction::CustomerInspect),
            (OrderStatus::CustomerInspecting, OrderAction::Complete),
            (
                OrderStatus::CustomerInspecting,
                OrderAction::CustomerExchange,
            ),
            (OrderStatus::Pending, OrderAction::Cancel),
            (OrderStatus::Assigned, OrderAction::Cancel),
        ];

        for (status, action) in &customer_actions {
            let result = try_transition(*status, *action, TenantType::Customer);
            assert!(
                result.success,
                "Customer should be able to perform {:?} in status {:?}",
                action, status
            );
        }
    }

    // ==================== 错误信息验证测试 ====================

    #[test]
    fn test_invalid_action_error_message() {
        let result = OrderStateMachine::next_state(
            OrderStatus::Pending,
            OrderAction::StartPreparing,
            TenantType::Customer,
        );

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("无效操作"));
    }

    #[test]
    fn test_terminal_state_error_message() {
        let result = OrderStateMachine::next_state(
            OrderStatus::Completed,
            OrderAction::Cancel,
            TenantType::Customer,
        );

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("状态已经是终态"));
    }
}
