use duan::{CustomEvent, EntityId};
use std::any::Any;

/// 导弹超出射程自毁事件
#[derive(Debug)]
pub struct MissileExpiredEvent {
    pub missile_id: EntityId,
}

impl CustomEvent for MissileExpiredEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn event_name(&self) -> &str {
        "missile_expired"
    }
}
