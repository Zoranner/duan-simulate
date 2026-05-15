use duan::{Reaction, World};
use duan_macros::reaction;
use example_naval_core::Health;

use crate::HitResolved;

pub struct ApplyDamage;

#[reaction(id = "apply-damage", label = "Apply Damage", event = HitResolved)]
impl Reaction<HitResolved> for ApplyDamage {
    fn react(&mut self, event: &HitResolved, world: &mut World) {
        if let Some(health) = world.inspect_mut::<Health>(event.target_id) {
            health.current = (health.current - event.damage).max(0.0);
        }
    }
}
