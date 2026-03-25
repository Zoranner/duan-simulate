//! 域模块
//!
//! 导出所有域规则类型。

pub mod collision;
pub mod motion;

pub use collision::CollisionRules;
pub use motion::MotionRules;
