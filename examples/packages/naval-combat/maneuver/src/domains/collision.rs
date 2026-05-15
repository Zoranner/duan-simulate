use duan::{Domain, DomainContext};

use crate::{MotionDomain, Position2};

pub struct CollisionDomain;

impl CollisionDomain {
    pub const ITEM_ID: &str = concat!(env!("CARGO_PKG_NAME"), "/collision");
}

impl Domain for CollisionDomain {
    type Writes = duan::component_set!();
    type Reads = duan::component_set!(Position2);
    type After = duan::domain_set!(MotionDomain);

    fn compute(&mut self, _ctx: &mut DomainContext<Self>, _delta_time: f64) {}
}
