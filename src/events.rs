//! 事件系统
//!
//! DUAN 事件模型以"事实 → 反应/观察"为核心：
//!
//! - [`Event`]：仿真中已发生的领域事实，纯数据，不承担副作用逻辑。
//! - 反应（`Reaction<E>`）：由 [`WorldBuilder::with_reaction`](crate::WorldBuilder::with_reaction) 注册，
//!   接收特定事件并允许修改世界，处理仿真内副作用。
//! - 观察（`Observer<E>`）：由 [`WorldBuilder::with_observer`](crate::WorldBuilder::with_observer) 注册，
//!   只读消费事件，用于统计、日志、测试采集。
//!
//! # 定义事件
//!
//! ```rust,ignore
//! use duan::Event;
//!
//! #[derive(Debug)]
//! pub struct HitEvent {
//!     pub target_id: duan::EntityId,
//!     pub damage: f64,
//! }
//!
//! impl Event for HitEvent {
//!     fn event_name(&self) -> &'static str { "hit" }
//! }
//! ```
//!
//! # 注册与消费
//!
//! ```rust,ignore
//! World::builder()
//!     .with_reaction::<HitEvent, _>(|e: &HitEvent, world: &mut World| {
//!         world.destroy(e.target_id);
//!     })
//!     .with_observer::<HitEvent, _>(|e: &HitEvent, _world: &World| {
//!         println!("命中！伤害 = {}", e.damage);
//!     })
//!     .build()
//! ```

use std::any::{Any, TypeId};
use std::sync::Arc;

// ──── Event ──────────────────────────────────────────────────────────────

/// 领域事实 trait
///
/// 实现此 trait 的类型表示仿真中已发生的领域事实。
///
/// # 约束
///
/// - 不需要实现 `Clone`
/// - 不需要 `as_any`；框架内部通过 [`TypeId`] 类型化分发，用户无需手动 downcast
///
/// # 示例
///
/// ```rust,ignore
/// use duan::Event;
///
/// #[derive(Debug)]
/// pub struct FireEvent { pub shooter_id: duan::EntityId }
///
/// impl Event for FireEvent {
///     fn event_name(&self) -> &'static str { "fire" }
/// }
/// ```
pub trait Event: Send + Sync + 'static {
    /// 事件名称（用于调试和日志）
    fn event_name(&self) -> &'static str;
}

// ──── 内部：类型擦除事件节点 ──────────────────────────────────────────────

/// 类型擦除的事件节点（框架内部使用）
pub(crate) struct ArcEvent {
    pub(crate) type_id: TypeId,
    pub(crate) inner: Arc<dyn Any + Send + Sync>,
    pub(crate) name: &'static str,
}

// ──── EventBuffer ────────────────────────────────────────────────────────

/// 帧内事件缓冲区
///
/// 收集一帧内产生的所有事实事件，在帧末统一分发到反应器和观察器。
/// 框架内部使用，用户通过 [`EntityContext::emit`](crate::EntityContext::emit) 或
/// [`DomainContext::emit`](crate::DomainContext::emit) 发送事件。
pub struct EventBuffer {
    facts: Vec<ArcEvent>,
}

impl EventBuffer {
    pub fn new() -> Self {
        Self { facts: Vec::new() }
    }

    /// 发出一个领域事实（框架内部接口，供 Context 调用）
    pub(crate) fn emit<E: Event>(&mut self, event: E) {
        let name = event.event_name();
        self.facts.push(ArcEvent {
            type_id: TypeId::of::<E>(),
            inner: Arc::new(event),
            name,
        });
    }

    pub(crate) fn drain(&mut self) -> Vec<ArcEvent> {
        std::mem::take(&mut self.facts)
    }

    pub fn len(&self) -> usize {
        self.facts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.facts.is_empty()
    }
}

impl Default for EventBuffer {
    fn default() -> Self {
        Self::new()
    }
}

// ──── TimerCallback ──────────────────────────────────────────────────────

/// 定时器回调
///
/// 当前唯一支持的行为是让实体在定时器触发时自毁。
/// 若需在特定时间发出事件，推荐在域的 `compute()` 中检查 `ctx.sim_time()` 并主动 `emit`。
#[derive(Clone, Debug)]
pub enum TimerCallback {
    /// 使实体在定时器触发时进入已销毁状态（自毁定时器）
    SelfDestruct,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct PingEvent {
        value: u32,
    }

    impl Event for PingEvent {
        fn event_name(&self) -> &'static str {
            "ping"
        }
    }

    #[test]
    fn test_event_buffer_emit_and_drain() {
        let mut buf = EventBuffer::new();
        buf.emit(PingEvent { value: 42 });
        assert_eq!(buf.len(), 1);

        let events = buf.drain();
        assert_eq!(events.len(), 1);
        assert!(buf.is_empty());

        assert_eq!(events[0].name, "ping");
        assert_eq!(events[0].type_id, TypeId::of::<PingEvent>());

        let ping = events[0].inner.downcast_ref::<PingEvent>().unwrap();
        assert_eq!(ping.value, 42);
    }

    #[test]
    fn test_event_buffer_default_empty() {
        let buf = EventBuffer::default();
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
    }
}
