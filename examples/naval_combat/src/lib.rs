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
//! | **事实** | `Reality` | `Position`、`Velocity`、`Health`、`Faction`、`Weapon`、`Seeker` 等；由域或初始生成设定 |

pub mod components;
pub mod domains;
pub mod entities;
pub mod events;
