//! 舰队对抗仿真示例
//!
//! 展示 DUAN 框架新一代 API 的多域协作、事件驱动、动态实体生命周期。
//!
//! # 域依赖链
//!
//! ```text
//! MotionDomain → CombatDomain → CollisionDomain
//! ```
//!
//! # 三元语义：认知、意图、事实
//!
//! | 术语（中文） | Rust | 本示例中的组件 |
//! |-----------|------|---------------|
//! | **认知** | `Belief` | （本示例未使用） |
//! | **意图** | `Intent` | `Helm`：舰船在 `Ship::tick()` 中写入期望航向，`MotionDomain` 从快照只读 |
//! | **事实** | `Reality` | `Position`、`Velocity`、`Health`、`Faction`、`Radar`、`Weapon`、`SeekerConfig`、`SeekerState` 等；由域或初始生成设定 |
//!
//! `Faction` 和 `Radar` 虽然是固有属性，但因为需要被其他实体（通过快照）和域读取，
//! 所以声明为 `Reality`。`Belief` 仅适用于实体私有、对外完全封闭的数据。

pub mod components;
pub mod domains;
pub mod entities;
pub mod events;
