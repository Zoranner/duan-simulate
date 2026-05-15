use duan::{Domain, DomainContext};
use duan_macros::domain;

use crate::{MotionDomain, Position2};

pub struct CollisionDomain;

#[domain(
    id = "collision",
    label = "Collision",
    reads(Position2),
    after(MotionDomain)
)]
impl Domain for CollisionDomain {
    fn compute(&mut self, _ctx: &mut DomainContext<Self>, _delta_time: f64) {}
}
