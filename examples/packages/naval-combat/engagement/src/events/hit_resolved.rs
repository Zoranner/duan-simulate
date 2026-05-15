use duan::{EntityId, Event};

#[derive(Debug)]
pub struct HitResolved {
    pub target_id: EntityId,
    pub damage: f64,
}

impl HitResolved {
    pub const ITEM_ID: &str = concat!(env!("CARGO_PKG_NAME"), "/hit-resolved");
}

impl Event for HitResolved {
    fn event_name(&self) -> &'static str {
        Self::ITEM_ID
    }
}
