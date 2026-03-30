use duan::Event;

/// 小球与地面碰撞事件
#[derive(Debug)]
pub struct GroundCollisionEvent {
    /// 碰撞前的 y 方向速度（m/s，向下为负）
    pub impact_velocity: f64,
    /// 使用的弹性系数
    pub restitution: f64,
}

impl Event for GroundCollisionEvent {
    fn event_name(&self) -> &'static str {
        "ground_collision"
    }
}
