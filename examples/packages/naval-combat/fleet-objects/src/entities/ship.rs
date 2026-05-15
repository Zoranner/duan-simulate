use duan::Entity;
use duan_macros::entity;
use example_naval_combat::Weapon;
use example_naval_core::{Faction, Health, Radar};
use example_naval_motion::{Position2, Velocity2};

pub struct Ship;

impl Ship {
    pub fn bundle(
        faction: Faction,
        position: Position2,
        velocity: Velocity2,
        health: Health,
        radar: Radar,
        weapon: Weapon,
    ) -> (Faction, Position2, Velocity2, Health, Radar, Weapon) {
        (faction, position, velocity, health, radar, weapon)
    }
}

#[entity(
    id = "ship",
    label = "Ship",
    components(Faction, Position2, Velocity2, Health, Radar, Weapon)
)]
impl Entity for Ship {}
