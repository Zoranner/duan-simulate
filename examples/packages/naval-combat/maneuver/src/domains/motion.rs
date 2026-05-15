use duan::{Domain, DomainContext};
use duan_macros::domain;

use crate::{Heading, Position2, Velocity2};

pub struct MotionDomain;

#[domain(
    id = "motion",
    label = "Motion",
    writes(Position2, Velocity2),
    reads(Heading)
)]
impl Domain for MotionDomain {
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
