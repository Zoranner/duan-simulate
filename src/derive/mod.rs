//! derive-first 入口（设计预留，尚未实现 proc-macro）
//!
//! # 当前状态
//!
//! 本模块是 derive-first API 的预留占位入口。目标形态（评估拆出独立 `duan-macros` crate 后实现）：
//!
//! ```rust,ignore
//! #[derive(Component)]
//! #[duan(belief)]
//! pub struct SoldierBelief { pub path_index: usize }
//!
//! #[derive(Component)]
//! #[duan(intent)]
//! pub struct MovementOrder { pub target_x: f64 }
//!
//! #[derive(Component)]
//! #[duan(reality)]
//! pub struct Position { pub x: f64, pub y: f64 }
//! ```
//!
//! # 现阶段推荐用法
//!
//! 请使用 [`belief!`](crate::belief)、[`intent!`](crate::intent)、[`reality!`](crate::reality)
//! 便捷宏声明语义，等价于手写 `impl Component + impl EntityWritable + impl Belief/Intent/Reality`：
//!
//! ```rust,ignore
//! duan::belief!(SoldierBelief);
//! duan::intent!(MovementOrder);
//! duan::reality!(Position, Velocity, Health);
//! ```
//!
//! # 演进路线
//!
//! 1. 当前：宏（`belief!` / `intent!` / `reality!`）
//! 2. 目标：proc-macro derive（`#[derive(Component)]` + `#[duan(belief|intent|reality)]`）
//! 3. 宏保留作为简单场景的快捷写法
