use duan::{EntityId, Event};

/// 命中事件
#[derive(Debug)]
pub struct HitEvent {
    pub missile_id: EntityId,
    pub target_id: EntityId,
    pub damage: f64,
}

impl Event for HitEvent {
    fn event_name(&self) -> &'static str {
        "hit"
    }
}
