use duan::{Reaction, World};
use examples_naval_combat_platform::Health;

use crate::HitResolved;

pub struct ApplyDamage;

impl ApplyDamage {
    pub const ITEM_ID: &str = concat!(env!("CARGO_PKG_NAME"), "/apply-damage");
}

impl Reaction<HitResolved> for ApplyDamage {
    fn react(&mut self, event: &HitResolved, world: &mut World) {
        if let Some(health) = world.inspect_mut::<Health>(event.target_id) {
            health.current = (health.current - event.damage).max(0.0);
        }
    }
}
