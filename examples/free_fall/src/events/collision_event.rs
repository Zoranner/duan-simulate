use duan::CustomEvent;
use std::any::Any;

/// 小球与地面碰撞事件
#[derive(Debug)]
pub struct GroundCollisionEvent {
    /// 碰撞前的 y 方向速度（m/s，向下为负）
    pub impact_velocity: f64,
    /// 使用的弹性系数
    pub restitution: f64,
}

impl CustomEvent for GroundCollisionEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn event_name(&self) -> &str {
        "ground_collision"
    }
}
