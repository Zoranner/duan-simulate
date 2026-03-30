use duan::{EntityId, Event};

/// 导弹超出射程自毁事件
#[derive(Debug)]
pub struct MissileExpiredEvent {
    pub missile_id: EntityId,
}

impl Event for MissileExpiredEvent {
    fn event_name(&self) -> &'static str {
        "missile_expired"
    }
}
