use duan::{entity, Entity};
use example_freefall_physics::{Position2, Velocity2};

pub struct Ball;

impl Ball {
    pub fn bundle_at(x: f64, y: f64) -> (Position2, Velocity2) {
        (Position2::new(x, y), Velocity2::default())
    }
}

#[entity(id = "ball", label = "Ball", components(Position2, Velocity2))]
impl Entity for Ball {}
