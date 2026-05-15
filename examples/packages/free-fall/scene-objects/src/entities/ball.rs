use duan::Entity;
use examples_free_fall_body::{Position2, Velocity2};

pub struct Ball;

impl Ball {
    pub const ITEM_ID: &str = concat!(env!("CARGO_PKG_NAME"), "/ball");

    pub fn bundle_at(x: f64, y: f64) -> (Position2, Velocity2) {
        (Position2::new(x, y), Velocity2::default())
    }
}

impl Entity for Ball {}
