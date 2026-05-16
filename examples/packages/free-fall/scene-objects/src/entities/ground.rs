use duan::{entity, Entity};
use example_freefall_physics::{Collider, Position2, StaticBody};

pub struct Ground;

impl Ground {
    pub fn bundle_at(x: f64, y: f64, restitution: f64) -> (Position2, StaticBody, Collider) {
        (
            Position2::new(x, y),
            StaticBody::enabled(),
            Collider::new(restitution),
        )
    }
}

#[entity(
    id = "ground",
    label = "Ground",
    components(Position2, StaticBody, Collider)
)]
impl Entity for Ground {}
