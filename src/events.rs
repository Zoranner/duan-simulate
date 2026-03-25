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
    /// 发送事件
    Event(Box<DomainEvent>),
    /// 自定义回调（通过 ID 识别）
    Custom(String),
}

/// 自定义事件 trait
///
/// 用户可以实现此 trait 来定义自己的事件类型。
pub trait CustomEvent: Send + Sync {
    /// 类型转换
    fn as_any(&self) -> &dyn Any;

    /// 类型转换（可变）
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// 克隆事件
    fn clone_event(&self) -> Box<dyn CustomEvent>;

    /// 事件名称
    fn event_name(&self) -> &str;
}

impl Clone for Box<dyn CustomEvent> {
    fn clone(&self) -> Self {
        self.clone_event()
    }
}

impl Debug for dyn CustomEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CustomEvent({})", self.event_name())
    }
}

/// 域事件
///
/// 框架核心的事件类型。用户可以扩展或使用自定义事件。
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

    /// 自定义事件（用户扩展）
    Custom(Box<dyn CustomEvent>),
}

// 实现一些便捷方法
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
    pub fn custom<E: CustomEvent + 'static>(event: E) -> Self {
        Self::Custom(Box::new(event))
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
}
