//! 自由落体小球仿真
//!
//! 展示 DUAN 框架**认知 / 意图 / 状态**三元语义（Rust：`Memory` / `Intent` / `State`）与
//! 5 阶段仿真循环的完整用法：
//!
//! | 概念 | 本示例中的体现 |
//! |------|--------------|
//! | **实体（Entity）** | `Ball`（小球）、`Ground`（地面） |
//! | **组件（Component）** | 按语义分为认知、意图、状态三类（见下表） |
//! | **域（Domain）** | `MotionDomain`（运动积分 + 碰撞响应，Position/Velocity 唯一权威） |
//! | **事件（CustomEvent）** | `GroundCollisionEvent` |
//!
//! # 三元语义：认知、意图、状态
//!
//! | 术语（中文） | Rust | 本示例中的组件 |
//! |-----------|------|---------------|
//! | **认知** | `Memory` | `BounceCount`：Ball 私有弹跳计数，仅 `Ball::tick()` 更新 |
//! | **意图** | `Intent` | （本示例未使用） |
//! | **状态** | `State` | `Position`、`Velocity`、`Collider`、`StaticBody`、`Mass`、`DidBounce` 等；由域权威写入或由初始生成设定 |
//!
//! `Ball::tick()` 从上帧快照读取**状态** `DidBounce`，更新自身**认知** `BounceCount`；运动与碰撞仍由域驱动，体现「域是状态权威、实体是意志主体」。

pub mod components;
pub mod domains;
pub mod entities;
pub mod events;
