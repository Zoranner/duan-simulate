use duan::{EntityId, Event};

/// 开火事件
#[derive(Debug)]
pub struct FireEvent {
    pub shooter_id: EntityId,
    pub target_id: EntityId,
    pub launch_x: f64,
    pub launch_y: f64,
    pub dir_x: f64,
    pub dir_y: f64,
    pub missile_speed: f64,
    pub missile_range: f64,
    pub damage: f64,
}

impl Event for FireEvent {
    fn event_name(&self) -> &'static str {
        "fire"
    }
}
