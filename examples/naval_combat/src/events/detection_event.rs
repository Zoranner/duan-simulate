use duan::{CustomEvent, EntityId};
use std::any::Any;

/// 探测事件：观察者发现敌方目标
pub struct DetectionEvent {
    pub observer_id: EntityId,
    pub target_id: EntityId,
    pub distance: f64,
}

impl DetectionEvent {
    pub fn new(observer_id: EntityId, target_id: EntityId, distance: f64) -> Self {
        Self {
            observer_id,
            target_id,
            distance,
        }
    }
}

impl CustomEvent for DetectionEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn event_name(&self) -> &str {
        "detection"
    }
}
