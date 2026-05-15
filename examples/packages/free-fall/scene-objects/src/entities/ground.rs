use duan::Entity;
use example_freefall_physics::{Collider, Position2, StaticBody};

pub struct Ground;

impl Ground {
    pub const ITEM_ID: &str = concat!(env!("CARGO_PKG_NAME"), "/ground");

    pub fn bundle_at(x: f64, y: f64, restitution: f64) -> (Position2, StaticBody, Collider) {
        (Position2::new(x, y), StaticBody, Collider::new(restitution))
    }
}

impl Entity for Ground {}
