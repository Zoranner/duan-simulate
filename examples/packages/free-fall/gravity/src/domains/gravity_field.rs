use duan::{domain, Domain, DomainContext};
use example_freefall_physics::{Collider, Position2, StaticBody, Velocity2};

pub struct GravityField {
    acceleration: f64,
}

impl GravityField {
    pub fn earth() -> Self {
        Self { acceleration: 9.8 }
    }
}

impl Default for GravityField {
    fn default() -> Self {
        Self::earth()
    }
}

#[domain(
    id = "field",
    label = "Gravity Field",
    writes(Position2, Velocity2),
    reads(Collider, StaticBody)
)]
impl Domain for GravityField {
    fn compute(&mut self, ctx: &mut DomainContext<Self>, delta_time: f64) {
        let ground_restitution = ctx
            .entities::<StaticBody>()
            .find_map(|id| ctx.get::<Collider>(id))
            .map(|collider| collider.restitution)
            .unwrap_or(1.0);

        let ids: Vec<_> = ctx.each_mut::<Velocity2>().map(|(id, _)| id).collect();
        for id in ids {
            let Some((position, velocity)) = ctx.get_pair_mut::<Position2, Velocity2>(id) else {
                continue;
            };

            velocity.vy -= self.acceleration * delta_time;
            position.x += velocity.vx * delta_time;
            position.y += velocity.vy * delta_time;

            if position.y <= 0.0 && velocity.vy < 0.0 {
                position.y = 0.0;
                velocity.vy = -velocity.vy * ground_restitution;
            }
        }
    }
}
