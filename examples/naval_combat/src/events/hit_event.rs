use duan::{CustomEvent, EntityId};
use std::any::Any;

/// 命中事件
#[derive(Debug)]
pub struct HitEvent {
    pub missile_id: EntityId,
    pub target_id: EntityId,
    pub damage: f64,
}

impl CustomEvent for HitEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn event_name(&self) -> &str {
        "hit"
    }
}
