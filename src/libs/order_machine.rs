// use strum_macros::{Display, EnumString};
// use stateful::{AsStateful, Machine, State, Transition};

// #[derive(Debug, Clone, PartialEq, Display, EnumString)]
// pub enum OrderState {
//     Draft,
//     Submitted,
//     Approved,
//     Rejected,
//     Published,
//     Accepted,
//     InProgress,
//     Completed,
//     Cancelled,
//     Expired,
// }

// #[derive(Debug, Clone, PartialEq)]
// pub enum OrderEvent {
//     Submit,
//     Approve,
//     Reject,
//     Publish,
//     Accept,
//     Start,
//     Complete,
//     Cancel,
//     Expire,
// }

// impl AsStateful for Order {
//     type State = OrderState;
//     type Event = OrderEvent;

//     fn state(&self) -> &Self::State {
//         &self.status
//     }
// }

// #[derive(Debug)]
// pub struct OrderMachine;

// impl Machine for OrderMachine {
//     type State = OrderState;
//     type Event = OrderEvent;

//     fn transition(state: &Self::State, event: &Self::Event) -> Option<Self::State> {
//         use OrderState::*;
//         use OrderEvent::*;

//         match (state, event) {
//             (Draft, Submit) => Some(Submitted),
//             (Submitted, Approve) => Some(Approved),
//             (Submitted, Reject) => Some(Rejected),
//             (Approved, Publish) => Some(Published),
//             (Published, Accept) => Some(Accepted),
//             (Accepted, Start) => Some(InProgress),
//             (InProgress, Complete) => Some(Completed),
//             // 可以从多个状态取消
//             (Submitted | Approved | Published | Accepted | InProgress, Cancel) => Some(Cancelled),
//             (Published, Expire) => Some(Expired),
//             _ => None,
//         }
//     }
// }

// // 使用示例
// impl Order {
//     pub async fn transition<T>(
//         &mut self,
//         event: OrderEvent,
//         context: &RequestContext,
//         repo: &T
//     ) -> Result<(), AppError>
//     where
//         T: OrderRepository + Send + Sync,
//     {
//         // 验证权限
//         let permission = match event {
//             OrderEvent::Submit => "order:submit",
//             OrderEvent::Approve => "order:approve",
//             OrderEvent::Reject => "order:reject",
//             // ... 其他事件对应的权限
//         };

//         if !context.has_permission(permission) {
//             return Err(AppError::Forbidden("Insufficient permissions"));
//         }

//         // 尝试状态转换
//         let next_state = OrderMachine::transition(&self.status, &event)
//             .ok_or_else(|| AppError::InvalidState("Invalid state transition"))?;

//         // 记录状态变更历史
//         repo.record_status_change(
//             self.id,
//             &self.status,
//             &next_state,
//             context.user_id,
//         ).await?;

//         // 更新状态
//         self.status = next_state;
//         repo.update_order_status(self.id, &self.status).await?;

//         // 发送状态变更事件
//         emit_order_status_changed_event(self).await?;

//         Ok(())
//     }
// }

// // 使用方式
// // #[axum::debug_handler]
// // pub async fn approve_order<T>(
// //     Extension(context): Extension<RequestContext>,
// //     Extension(repo): Extension<T>,
// //     Path(order_id): Path<u64>,
// // ) -> Result<impl IntoResponse, AppError>
// // where
// //     T: OrderRepository + Send + Sync,
// // {
// //     let mut order = repo.get_order(order_id).await?;
// //     order.transition(OrderEvent::Approve, &context, &repo).await?;
// //     Ok(Json(ApiResponse::new_with_context(order, context)))
// // }
