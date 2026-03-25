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
pub use entity::{ComponentBag, Entity, EntityId, EntityStore, Lifecycle};
pub use events::{CustomEvent, DestroyCause, DomainEvent, Event, EventChannel, TimerCallback};
pub use time::{TimeClock, Timer, TimerEvent, TimerManager};
pub use world::{World, WorldBuilder};

/// 仿真体系的版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 仿真体系的名称
pub const NAME: &str = "DUAN";
