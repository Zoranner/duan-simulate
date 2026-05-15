use duan::Entity;
use examples_naval_combat_engagement::Weapon;
use examples_naval_combat_maneuver::{Position2, Velocity2};
use examples_naval_combat_platform::{Faction, Health, Radar};

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
