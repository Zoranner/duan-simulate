//! DUAN 仿真体系核心框架
//!
//! DUAN 采用域驱动设计，以域（Domain）为核心计算单元，构建可扩展的仿真体系。
//!
//! # 核心概念
//!
//! - **实体（Entity）**：仿真对象的基本单元，是组件的容器
//! - **组件（Component）**：实体的数据组成单元
//! - **域（Domain）**：权威计算单元，负责特定领域的计算和判定
//! - **事件（Event）**：域之间通信的机制
//! - **世界（World）**：仿真的顶层容器
//!
//! # 设计原则
//!
//! - **域是权威**：每个领域有唯一的权威域来裁决
//! - **域运行时定义**：框架不预设域类型，运行时注册
//! - **实体自声明归属**：实体声明自己要加入哪些域
//! - **事件驱动传播**：域的计算结果通过事件系统传播

pub mod component;
pub mod domain;
pub mod entity;
pub mod events;
pub mod time;
pub mod world;

// 重导出核心类型
pub use component::Component;
pub use domain::{Domain, DomainContext, DomainRegistry, DomainRules};
pub use entity::{Entity, EntityId, EntityStore, Lifecycle};
pub use events::{CustomEvent, DestroyCause, DomainEvent, Event, EventChannel, TimerCallback};
pub use time::{TimeClock, Timer, TimerEvent, TimerManager};
pub use world::{World, WorldBuilder};

/// 为结构体自动实现 `Component` trait 所需的样板代码
///
/// 生成 `as_any`、`as_any_mut`、`into_any_boxed` 三个方法，并指定组件类型名称。
///
/// # 用法
///
/// ```rust,ignore
/// use duan::{Component, impl_component};
///
/// pub struct Position { pub x: f64, pub y: f64 }
///
/// impl_component!(Position, "position");
///
/// impl Component for Position {
///     // 只需实现业务方法，样板已由宏生成
///     fn component_type(&self) -> &'static str { "position" }
///     // as_any / as_any_mut / into_any_boxed 已由 impl_component! 生成
/// }
/// ```
///
/// 完整用法（宏生成包括 `component_type` 在内的全部方法）：
///
/// ```rust,ignore
/// impl_component!(Position, "position");
/// ```
#[macro_export]
macro_rules! impl_component {
    ($type:ty, $name:expr) => {
        impl $crate::Component for $type {
            fn component_type(&self) -> &'static str {
                $name
            }
            fn as_any(&self) -> &dyn ::std::any::Any {
                self
            }
            fn as_any_mut(&mut self) -> &mut dyn ::std::any::Any {
                self
            }
            fn into_any_boxed(self: ::std::boxed::Box<Self>) -> ::std::boxed::Box<dyn ::std::any::Any> {
                self
            }
        }
    };
}

/// 为 `DomainRules` 实现类型转换样板（`as_any` / `as_any_mut`）
///
/// 在 `impl DomainRules for MyRules { ... }` 块内调用，
/// 替代手写 `as_any` 和 `as_any_mut` 两个方法。
///
/// # 用法
///
/// ```rust,ignore
/// use duan::{DomainRules, domain_rules_any};
///
/// impl DomainRules for MotionRules {
///     fn compute(&mut self, ctx: &mut DomainContext, dt: f64) { ... }
///     fn try_attach(&mut self, entity: &Entity) -> bool { ... }
///     fn on_detach(&mut self, _entity_id: EntityId) {}
///     domain_rules_any!(MotionRules);
/// }
/// ```
#[macro_export]
macro_rules! domain_rules_any {
    ($type:ty) => {
        fn as_any(&self) -> &dyn ::std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn ::std::any::Any {
            self
        }
    };
}

/// 仿真体系的版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 仿真体系的名称
pub const NAME: &str = "DUAN";
