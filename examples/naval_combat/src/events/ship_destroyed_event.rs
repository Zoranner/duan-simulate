use duan::{EntityId, Event};

/// 舰船被摧毁事件
#[derive(Debug)]
pub struct ShipDestroyedEvent {
    pub ship_id: EntityId,
}

impl Event for ShipDestroyedEvent {
    fn event_name(&self) -> &'static str {
        "ship_destroyed"
    }
}
