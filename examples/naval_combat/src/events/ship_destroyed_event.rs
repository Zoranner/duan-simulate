use duan::{CustomEvent, EntityId};
use std::any::Any;

/// 舰船销毁事件：舰船生命值归零
pub struct ShipDestroyedEvent {
    pub ship_id: EntityId,
    pub killer_id: EntityId,
}

impl CustomEvent for ShipDestroyedEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn event_name(&self) -> &str {
        "ship_destroyed"
    }
}
