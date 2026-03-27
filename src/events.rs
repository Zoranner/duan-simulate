//! 事件（Event）是域之间通信的机制
//!
//! 事件是域的计算结果的表达方式。当域完成计算后，通过发出事件来通知仿真系统。
//!
//! # 设计原则
//!
//! - **解耦**：域不需要知道谁会处理它的事件
//! - **可追溯**：所有事件都可以被记录和分析
//! - **一致性**：计算和状态更新分离，确保一致性

use crate::EntityId;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;

/// 事件通道
///
/// 收集和分发事件的容器。
pub type EventChannel = Vec<DomainEvent>;

/// 事件 trait
///
/// 所有事件必须实现此 trait。
pub trait Event: Send + Sync + 'static {
    /// 事件类型名称
    fn event_type(&self) -> &'static str;

    /// 获取事件涉及的实体（可选）
    fn entities(&self) -> Vec<EntityId> {
        vec![]
    }

    /// 获取事件时间戳（可选）
    fn timestamp(&self) -> Option<f64> {
        None
    }
}

/// 销毁原因
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DestroyCause {
    /// 被击中
    Hit,
    /// 超时
    Timeout,
    /// 主动销毁
    Manual,
    /// 其他原因
    Other,
}

/// 定时器回调类型
#[derive(Clone, Debug)]
pub enum TimerCallback {
    /// 自毁
    SelfDestruct,
    /// 发送事件（DomainEvent 通过 Box 持有，可 Clone 因为 DomainEvent: Clone）
    Event(Box<DomainEvent>),
    /// 自定义回调（通过 ID 识别）
    Custom(String),
}

/// 自定义事件 trait
///
/// 用户实现此 trait 来定义自己的事件类型。
/// 不要求 Clone——框架通过 `Arc` 共享自定义事件数据，克隆 `DomainEvent` 时只增加引用计数。
pub trait CustomEvent: Send + Sync {
    /// 类型转换（只读）
    fn as_any(&self) -> &dyn Any;

    /// 类型转换（可变）
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// 事件名称（用于调试和日志）
    fn event_name(&self) -> &str;
}

impl Debug for dyn CustomEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CustomEvent({})", self.event_name())
    }
}

/// 域事件
///
/// 框架核心的事件类型。用户可以通过 `Custom` 变体扩展自己的事件。
///
/// # 克隆语义
///
/// `DomainEvent` 实现了 `Clone`：
/// - 框架内置变体（`EntitySpawned`、`EntityDestroyed`、`Timer`）的克隆是数据深拷贝，开销极低。
/// - `Custom` 变体持有 `Arc<dyn CustomEvent>`，克隆只增加引用计数，不复制事件数据。
///   因此 `CustomEvent` 本身**不要求**实现 `Clone`。
#[derive(Clone, Debug)]
pub enum DomainEvent {
    /// 实体创建事件
    EntitySpawned {
        entity_id: EntityId,
        entity_type: String,
    },

    /// 实体销毁事件
    EntityDestroyed {
        entity_id: EntityId,
        cause: DestroyCause,
    },

    /// 定时器事件
    Timer {
        entity_id: EntityId,
        timer_id: String,
        callback: TimerCallback,
    },

    /// 自定义事件（Arc 共享，克隆廉价）
    Custom(Arc<dyn CustomEvent>),
}

impl DomainEvent {
    /// 创建实体创建事件
    pub fn spawned(entity_id: EntityId, entity_type: impl Into<String>) -> Self {
        Self::EntitySpawned {
            entity_id,
            entity_type: entity_type.into(),
        }
    }

    /// 创建实体销毁事件
    pub fn destroyed(entity_id: EntityId, cause: DestroyCause) -> Self {
        Self::EntityDestroyed { entity_id, cause }
    }

    /// 创建定时器事件
    pub fn timer_event(
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

    /// 创建自定义事件
    ///
    /// 事件数据由 `Arc` 持有，多次克隆 `DomainEvent` 时不会复制事件数据。
    pub fn custom<E: CustomEvent + 'static>(event: E) -> Self {
        Self::Custom(Arc::new(event))
    }

    /// 获取事件涉及的实体
    pub fn entities(&self) -> Vec<EntityId> {
        match self {
            Self::EntitySpawned { entity_id, .. } => vec![*entity_id],
            Self::EntityDestroyed { entity_id, .. } => vec![*entity_id],
            Self::Timer { entity_id, .. } => vec![*entity_id],
            Self::Custom(_) => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_event() {
        let event = DomainEvent::destroyed(EntityId::new(1), DestroyCause::Manual);
        assert_eq!(event.entities(), vec![EntityId::new(1)]);
    }

    #[test]
    fn test_event_channel() {
        let channel: EventChannel = vec![
            DomainEvent::spawned(EntityId::new(1), "ship"),
            DomainEvent::destroyed(EntityId::new(2), DestroyCause::Timeout),
        ];

        assert_eq!(channel.len(), 2);
    }

    #[test]
    fn test_custom_event_no_clone_required() {
        struct MyEvent {
            value: i32,
        }

        impl CustomEvent for MyEvent {
            fn as_any(&self) -> &dyn Any { self }
            fn as_any_mut(&mut self) -> &mut dyn Any { self }
            fn event_name(&self) -> &str { "my_event" }
        }

        let event = DomainEvent::custom(MyEvent { value: 42 });
        // Clone 只增加 Arc 引用计数，不复制 MyEvent 数据
        let cloned = event.clone();

        if let DomainEvent::Custom(arc) = &cloned {
            assert_eq!(arc.event_name(), "my_event");
            let inner = arc.as_any().downcast_ref::<MyEvent>().unwrap();
            assert_eq!(inner.value, 42);
        } else {
            panic!("expected Custom event");
        }
    }
}
