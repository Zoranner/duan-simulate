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
//! | **认知** | `Memory` | `BounceCount`：Ball 私有弹跳计数，仅 `Ball::tick()` 更新，域和快照不可见 |
//! | **意图** | `Intent` | `Elasticity`：Ball 每帧声明期望弹性系数，`MotionDomain` 从快照只读 |
//! | **状态** | `State` | `Position`、`Velocity`、`Collider`、`StaticBody`、`Mass`、`DidBounce` 等；由域权威写入 |
//!
//! `Ball::tick()` 感知上帧**状态** `DidBounce`，更新自身**认知** `BounceCount`，
//! 再据此重新声明**意图** `Elasticity`（弹性随弹跳次数递减）；
//! `MotionDomain` 读取意图乘以地面参数得出实际弹性系数，体现「实体意志驱动域行为」的完整闭环。

pub mod components;
pub mod domains;
pub mod entities;
pub mod events;
