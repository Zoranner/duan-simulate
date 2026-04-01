//! 自由落体小球仿真
//!
//! 展示 DUAN 框架**认知 / 意图 / 事实**三元语义（Rust：`Belief` / `Intent` / `Reality`）与
//! 5 阶段仿真循环的完整用法：
//!
//! | 概念 | 本示例中的体现 |
//! |------|--------------|
//! | **实体（Entity）** | `Ball`（小球）、`Ground`（地面） |
//! | **组件（Component）** | 按语义分为认知、意图、事实三类（见下表） |
//! | **域（Domain）** | `MotionDomain`（运动积分 + 碰撞响应，Position/Velocity 唯一权威） |
//! | **事件（Event）** | `GroundCollisionEvent` |
//!
//! # 三元语义：认知、意图、事实
//!
//! | 术语（中文） | Rust | 本示例中的组件 |
//! |-----------|------|---------------|
//! | **认知** | `Belief` | `BounceCount`：Ball 私有弹跳计数，仅 `Ball::tick()` 更新，域和快照不可见 |
//! | **意图** | `Intent` | `Elasticity`：Ball 每帧声明期望弹性系数，`MotionDomain` 从快照只读 |
//! | **事实** | `Reality` | `Position`、`Velocity`、`Collider`、`StaticBody`、`DidBounce` 等；由域权威写入 |
//!
//! `Ball::tick()` 感知上帧**事实** `DidBounce`，更新自身**认知** `BounceCount`，
//! 再据此重新声明**意图** `Elasticity`（弹性随弹跳次数递减）；
//! `MotionDomain` 读取意图乘以地面参数得出实际弹性系数，体现「实体表达意图，域裁定事实」的完整闭环。

pub mod components;
pub mod domains;
pub mod entities;
pub mod events;
