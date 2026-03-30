//! 事件系统
//!
//! 事件用于域向仿真系统其他部分传递信息。
//! 域通过 [`EntityContext::emit`](crate::EntityContext::emit) 和
//! [`DomainContext::emit`](crate::DomainContext::emit) 发出事件，
//! 在本帧所有计算完成后统一处理。

use crate::entity::id::EntityId;
use std::any::Any;
use std::fmt;
use std::sync::Arc;

// ──── CustomEvent ────────────────────────────────────────────────────────

/// 自定义事件 trait
///
/// 用户实现此 trait 来定义仿真特有的事件类型。
/// 不要求 Clone：框架通过 Arc 共享事件数据，克隆 Event 时只增加引用计数。
///
/// # 示例
///
/// ```rust,ignore
/// use duan::CustomEvent;
/// use std::any::Any;
///
/// pub struct HitEvent { pub target_id: EntityId, pub damage: f64 }
///
/// impl CustomEvent for HitEvent {
///     fn as_any(&self) -> &dyn Any { self }
///     fn event_name(&self) -> &str { "hit" }
/// }
/// ```
pub trait CustomEvent: Send + Sync {
    /// 用于 downcast 到具体类型
    fn as_any(&self) -> &dyn Any;

    /// 事件名称（用于调试和日志）
    fn event_name(&self) -> &str;
}

impl dyn CustomEvent {
    /// Downcast 到具体类型
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// world.step_with(dt, |event, _world| {
    ///     if let Some(hit) = event.downcast::<HitEvent>() {
    ///         println!("命中伤害: {}", hit.damage);
    ///     }
    /// });
    /// ```
    pub fn downcast<T: 'static>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }
}

impl fmt::Debug for dyn CustomEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CustomEvent({})", self.event_name())
    }
}

// ──── FrameworkEvent ────────────────────────────────────────────────────

/// 框架内置事件
///
/// 框架生命周期相关的内部事件。用户通过 `step_with` 的回调处理自定义事件，
/// 框架内置事件由 World 内部自动处理。
#[derive(Clone, Debug)]
pub enum FrameworkEvent {
    /// 实体已销毁
    EntityDestroyed { entity_id: EntityId },
    /// 定时器触发
    Timer {
        entity_id: EntityId,
        timer_id: String,
        callback: TimerCallback,
    },
    /// 用户自定义事件（Arc 共享，克隆廉价）
    Custom(Arc<dyn CustomEvent>),
}

impl FrameworkEvent {
    /// 构造实体销毁事件
    pub fn destroyed(entity_id: EntityId) -> Self {
        Self::EntityDestroyed { entity_id }
    }

    /// 构造定时器事件
    pub fn timer(
        entity_id: EntityId,
        timer_id: impl Into<String>,
        callback: TimerCallback,
    ) -> Self {
        Self::Timer {
            entity_id,
            timer_id: timer_id.into(),
            callback,
        }
    }

    /// 包装自定义事件
    pub fn custom<E: CustomEvent + 'static>(event: E) -> Self {
        Self::Custom(Arc::new(event))
    }
}

// ──── TimerCallback ──────────────────────────────────────────────────────

/// 定时器回调
#[derive(Clone, Debug)]
pub enum TimerCallback {
    /// 使实体进入已销毁状态（自毁定时器）
    SelfDestruct,
    /// 触发嵌套事件
    Event(Box<FrameworkEvent>),
}

// ──── EventBuffer ────────────────────────────────────────────────────────

/// 帧内事件缓冲区
///
/// 收集一帧内产生的所有事件，在帧末统一分发处理。
pub struct EventBuffer {
    events: Vec<FrameworkEvent>,
}

impl EventBuffer {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn push(&mut self, event: FrameworkEvent) {
        self.events.push(event);
    }

    pub fn push_custom<E: CustomEvent + 'static>(&mut self, event: E) {
        self.events.push(FrameworkEvent::custom(event));
    }

    pub fn drain(&mut self) -> Vec<FrameworkEvent> {
        std::mem::take(&mut self.events)
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

impl Default for EventBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;

    struct PingEvent {
        value: u32,
    }

    impl CustomEvent for PingEvent {
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn event_name(&self) -> &str {
            "ping"
        }
    }

    #[test]
    fn test_event_buffer() {
        let mut buf = EventBuffer::new();
        buf.push_custom(PingEvent { value: 42 });
        assert_eq!(buf.len(), 1);

        let events = buf.drain();
        assert_eq!(events.len(), 1);
        assert!(buf.is_empty());

        if let FrameworkEvent::Custom(arc) = &events[0] {
            let ping = arc.downcast::<PingEvent>().unwrap();
            assert_eq!(ping.value, 42);
        } else {
            panic!("expected Custom event");
        }
    }

    #[test]
    fn test_custom_event_no_clone_required() {
        let event = FrameworkEvent::custom(PingEvent { value: 99 });
        let cloned = event.clone();
        if let FrameworkEvent::Custom(arc) = cloned {
            assert_eq!(arc.event_name(), "ping");
        }
    }
}
