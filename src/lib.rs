//! DUAN 仿真体系核心框架
//!
//! DUAN 采用域驱动 ECS 架构，以域（Domain）为核心计算单元，构建可扩展的仿真体系。
//!
//! # 三大编程原语
//!
//! | 原语      | 角色     | 说明                                                           |
//! |---------|--------|--------------------------------------------------------------|
//! | Component | 数据   | 实体附加数据的通用约束，分认知/意图/事实（`Belief`/`Intent`/`Reality`）       |
//! | Entity  | 意志主体  | 零大小标记类型，通过 `tick()` 定义行为                                    |
//! | Domain  | 规则权威  | 独占写入特定**事实**（`Reality`）类型，按拓扑顺序执行                            |
//!
//! # 三元语义：认知、意图、事实
//!
//! 中文术语与 Rust trait 一一对应（与 README 术语表一致）：**认知** → [`Belief`]，**意图** → [`Intent`]，**事实** → [`Reality`]。
//!
//! | 术语（中文） | Rust trait | 实体       | 域        | Snapshot |
//! |-----------|-----------|----------|----------|---------------|
//! | 认知 | Belief     | 读写       | 不可见     | 不可见           |
//! | 意图 | Intent     | 读写       | 只读（快照） | 只读            |
//! | 事实 | Reality    | 只读（快照） | 独占写入   | 只读            |
//!
//! # 事件模型
//!
//! | 角色 | trait | 说明 |
//! |-----|-------|------|
//! | 事件 | [`Event`]        | 已发生的一次变化，纯数据（README：事件 / Event） |
//! | 反应器 | [`Reaction<E>`](Reaction)  | 接收事件并修改世界，用于仿真内副作用 |
//! | 观察器 | [`Observer<E>`](Observer)  | 只读消费事件，用于统计、日志、测试 |
//!
//! # 快速开始
//!
//! ```rust,ignore
//! use duan::prelude::*;
//!
//! #[derive(Clone, Default)]
//! struct Position { pub x: f64, pub y: f64 }
//! reality!(Position);
//!
//! struct Ball;
//! impl Entity for Ball {
//!     fn bundle() -> impl ComponentBundle {
//!         (Position { x: 0.0, y: 10.0 },)
//!     }
//! }
//!
//! struct GravityDomain;
//! impl Domain for GravityDomain {
//!     type Writes = duan::component_set!(Position);
//!     type Reads  = duan::component_set!(Position);
//!     type After  = duan::domain_set!();
//!     fn compute(&mut self, ctx: &mut DomainContext<Self>, delta_time: f64) {
//!         let ids: Vec<_> = ctx.each::<Position>().map(|(id, _)| id).collect();
//!         for id in ids {
//!             if let Some(p) = ctx.get_mut::<Position>(id) {
//!                 p.y -= 9.8 * delta_time;
//!             }
//!         }
//!     }
//! }
//!
//! struct BounceEvent { pub impact_velocity: f64 }
//! impl Event for BounceEvent {
//!     fn event_name(&self) -> &'static str { "bounce" }
//! }
//!
//! /// 观察器：记录弹跳事件（只读，不修改世界）
//! struct LogBounce;
//! impl Observer<BounceEvent> for LogBounce {
//!     fn observe(&mut self, ev: &BounceEvent, _world: &World) {
//!         println!("弹跳！冲击速度 = {:.2}", ev.impact_velocity);
//!     }
//! }
//!
//! let mut world = World::builder()
//!     .domain(GravityDomain)
//!     .observe::<BounceEvent>(LogBounce)
//!     .build();
//!
//! let ball = world.spawn::<Ball>();
//! world.step(0.016);
//! println!("y = {:.3}", world.get::<Position>(ball).unwrap().y);
//! println!("time = {:.3}", world.time());
//! ```
//!
//! # 大型项目模块化装配
//!
//! ```rust,ignore
//! // combat/mod.rs
//! pub fn install(builder: WorldBuilder) -> WorldBuilder {
//!     builder
//!         .domain(CombatDomain)
//!         .on::<HitEvent>(HandleHit)
//! }
//!
//! // main.rs
//! let mut world = World::builder()
//!     .domain(MotionDomain)
//!     .apply(combat::install)
//!     .apply(collision::install)
//!     .build();
//! ```
//!
//! # 导入路径
//!
//! - **日常开发**：`use duan::prelude::*`（覆盖 80% 场景）
//! - **高级场景**：直接从 `duan::` 导入定时器、日志、快照等类型
//! - **专家级**：通过 `duan::diagnostics::*`、`duan::storage::*` 访问内部细节

// ──── 概念模块（pub）────────────────────────────────────────────────────────

pub mod component;
pub mod derive;
pub mod diagnostics;
pub mod domain;
pub mod entity;
pub mod event;
pub mod prelude;
pub mod snapshot;
pub mod storage;
#[doc(hidden)]
pub mod type_set;
pub mod world;

// ──── 执行机制模块（pub，高级用户）──────────────────────────────────────────

pub mod runtime;

// ──── 核心类型重导出（prelude 覆盖的部分同样在此直接可用）──────────────────────

// 编程原语
pub use component::{
    Belief, Component, ComponentKind, ComponentSet, EntityWritable, Intent, Reality,
};
pub use domain::context::DomainContext;
pub use domain::{Domain, DomainSet};
pub use entity::context::EntityContext;
pub use entity::id::EntityId;
pub use entity::{ComponentBundle, Entity, Lifecycle};
pub use event::{Event, Observer, Reaction};
pub use world::{World, WorldBuilder};

// 高级场景
pub use diagnostics::{LogLevel, LogSink, LoggerHandle};
pub use runtime::timers::{TimeClock, Timer, TimerCallback};
pub use snapshot::Snapshot;
pub use storage::Storage;

// ──── 框架常量 ──────────────────────────────────────────────────────────────

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = "DUAN";
