//! DUAN 仿真体系核心框架
//!
//! DUAN 采用域驱动 ECS 架构，以域（Domain）为核心计算单元，构建可扩展的仿真体系。
//!
//! # 三大编程原语
//!
//! | 原语      | 角色          | 说明                              |
//! |---------|-------------|----------------------------------|
//! | Component | 数据        | 实体附加数据的通用约束，分认知/意图/状态（`Memory`/`Intent`/`State`） |
//! | Entity  | 意志主体      | 零大小标记类型，通过 tick() 定义行为         |
//! | Domain  | 状态权威      | 独占写入特定**状态**（`State`）类型，按拓扑顺序执行 |
//!
//! # 三元语义：认知、意图、状态
//!
//! 中文术语与 Rust trait 一一对应：**认知** → [`Memory`]，**意图** → [`Intent`]，**状态** → [`State`]。
//!
//! | 术语（中文） | Rust trait | 实体         | 域          | WorldSnapshot |
//! |-----------|-----------|------------|------------|---------------|
//! | 认知 | Memory | 读写         | 不可见       | 不可见          |
//! | 意图 | Intent | 读写         | 只读（快照）   | 只读            |
//! | 状态 | State  | 只读（快照）   | 独占写入      | 只读            |
//!
//! # 事件模型
//!
//! | 角色 | trait | 说明 |
//! |-----|-------|------|
//! | 事实 | [`Event`] | 领域发出的已发生事实，纯数据 |
//! | 反应 | [`Reaction<E>`](Reaction) | 接收事件并修改世界，用于仿真内副作用 |
//! | 观察 | [`Observer<E>`](Observer) | 只读消费事件，用于统计、日志、测试 |
//!
//! # 快速开始
//!
//! ```rust,ignore
//! use duan::prelude::*;
//!
//! // 定义 State 组件
//! #[derive(Clone, Default)]
//! struct Position { pub x: f64, pub y: f64 }
//! state!(Position);
//!
//! // 定义实体（可无行为）
//! struct Ball;
//! impl Entity for Ball {
//!     fn bundle() -> impl ComponentBundle {
//!         (Position { x: 0.0, y: 10.0 },)
//!     }
//! }
//!
//! // 定义域
//! struct GravityDomain;
//! impl Domain for GravityDomain {
//!     type Writes = (Position,);
//!     type Reads = (Position,);
//!     type After = ();
//!     fn compute(&mut self, ctx: &mut DomainContext<Self>, delta_time: f64) {
//!         for (id, _) in ctx.each::<Position>() {
//!             if let Some(p) = ctx.get_mut::<Position>(id) {
//!                 p.y -= 9.8 * delta_time;
//!             }
//!         }
//!     }
//! }
//!
//! // 定义事件
//! struct BounceEvent { pub impact_velocity: f64 }
//! impl Event for BounceEvent {
//!     fn event_name(&self) -> &'static str { "bounce" }
//! }
//!
//! // 构建并运行
//! let mut world = World::builder()
//!     .domain(GravityDomain)
//!     .observe::<BounceEvent>(|ev: &BounceEvent, _world: &World| {
//!         println!("弹跳！冲击速度 = {:.2}", ev.impact_velocity);
//!     })
//!     .build();
//! let ball = world.spawn::<Ball>();
//! world.step(0.016);
//! println!("y = {:.3}", world.get::<Position>(ball).unwrap().y);
//! ```

pub mod components;
pub mod derive;
pub mod diagnostics;
pub mod domain;
pub mod entity;
pub mod event;
pub mod prelude;
pub mod runtime;
pub mod scheduler;
pub mod snapshot;
pub mod storage;
pub mod world;

// ──── 核心类型重导出 ──────────────────────────────────────────────────────

pub use components::{
    Component, ComponentKind, ComponentSet, Contains, EntityWritable, Intent, Memory, State,
};
pub use diagnostics::{LogLevel, LogRecord, LogSink, Logger, LoggerHandle};
pub use domain::context::DomainContext;
pub use domain::{Domain, DomainSet, InReads, InWrites};
pub use entity::context::EntityContext;
pub use entity::id::EntityId;
pub use entity::{ComponentBundle, Entity, Lifecycle};
pub use event::{Event, Observer, Reaction};
pub use runtime::timers::{TimeClock, Timer, TimerCallback};
pub use snapshot::WorldSnapshot;
pub use world::{World, WorldBuilder};

// ──── 框架常量 ──────────────────────────────────────────────────────────

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = "DUAN";
