//! 日常开发导入集（覆盖 80% 仿真开发场景）
//!
//! 使用方式：`use duan::prelude::*;`
//!
//! 包含内容：
//! - 世界构建与运行：[`World`]、[`WorldBuilder`]
//! - 实体：[`Entity`]、[`EntityContext`]、[`EntityId`]、[`ComponentBundle`]
//! - 域：[`Domain`]、[`DomainContext`]
//! - 事件：[`Event`]、[`Reaction`]、[`Observer`]
//! - 组件语义：[`Component`]、[`Memory`]、[`Intent`]、[`State`]、[`EntityWritable`]
//!
//! 宏（`#[macro_export]`，从 crate 根直接可用，无需 prelude）：
//! [`memory!`](crate::memory)、[`intent!`](crate::intent)、[`state!`](crate::state)
//!
//! 高级场景（定时器、快照、日志等）请直接从 `duan::` 导入：
//! [`Timer`](crate::Timer)、[`WorldSnapshot`](crate::WorldSnapshot)、
//! [`LogSink`](crate::LogSink) 等。

// 世界
pub use crate::world::{World, WorldBuilder};

// 实体
pub use crate::entity::context::EntityContext;
pub use crate::entity::id::EntityId;
pub use crate::entity::{ComponentBundle, Entity};

// 域
pub use crate::domain::context::DomainContext;
pub use crate::domain::Domain;

// 事件
pub use crate::event::{Event, Observer, Reaction};

// 组件语义
pub use crate::component::{Component, EntityWritable, Intent, Memory, State};
