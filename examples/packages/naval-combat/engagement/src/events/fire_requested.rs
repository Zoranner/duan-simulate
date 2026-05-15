use duan::{EntityId, Event};

#[derive(Debug)]
pub struct FireRequested {
    pub shooter_id: EntityId,
    pub target_id: EntityId,
    pub launch_x: f64,
    pub launch_y: f64,
    pub dir_x: f64,
    pub dir_y: f64,
    pub missile_speed: f64,
    pub damage: f64,
}

impl FireRequested {
    pub const ITEM_ID: &str = concat!(env!("CARGO_PKG_NAME"), "/fire-requested");
}

impl Event for FireRequested {
    fn event_name(&self) -> &'static str {
        Self::ITEM_ID
    }
}
