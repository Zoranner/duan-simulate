use duan::Entity;
use example_naval_combat::Weapon;
use example_naval_core::{Faction, Health, Radar};
use example_naval_motion::{Position2, Velocity2};

pub struct Ship;

impl Ship {
    pub const ITEM_ID: &str = concat!(env!("CARGO_PKG_NAME"), "/ship");

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

impl Entity for Ship {}
