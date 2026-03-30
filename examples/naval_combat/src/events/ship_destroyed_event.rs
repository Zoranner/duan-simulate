use duan::{CustomEvent, EntityId};
use std::any::Any;

/// 舰船被摧毁事件
#[derive(Debug)]
pub struct ShipDestroyedEvent {
    pub ship_id: EntityId,
}

impl CustomEvent for ShipDestroyedEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn event_name(&self) -> &str {
        "ship_destroyed"
    }
}
