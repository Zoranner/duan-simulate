use duan::{Domain, DomainContext};

use crate::{Heading, Position2, Velocity2};

pub struct MotionDomain;

impl MotionDomain {
    pub const ITEM_ID: &str = concat!(env!("CARGO_PKG_NAME"), "/motion");
}

impl Domain for MotionDomain {
    type Writes = duan::component_set!(Position2, Velocity2);
    type Reads = duan::component_set!(Heading);
    type After = duan::domain_set!();

    fn compute(&mut self, ctx: &mut DomainContext<Self>, delta_time: f64) {
        let headings: Vec<_> = ctx
            .each::<Heading>()
            .map(|(id, heading)| (id, heading.radians))
            .collect();

        for (id, heading) in headings {
            if let Some(velocity) = ctx.get_mut::<Velocity2>(id) {
                let speed = (velocity.vx * velocity.vx + velocity.vy * velocity.vy).sqrt();
                velocity.vx = heading.cos() * speed;
                velocity.vy = heading.sin() * speed;
            }
        }

        let ids: Vec<_> = ctx.each_mut::<Velocity2>().map(|(id, _)| id).collect();
        for id in ids {
            let Some((position, velocity)) = ctx.get_pair_mut::<Position2, Velocity2>(id) else {
                continue;
            };
            position.x += velocity.vx * delta_time;
            position.y += velocity.vy * delta_time;
        }
    }
}
