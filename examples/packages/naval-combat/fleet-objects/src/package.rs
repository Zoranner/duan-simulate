use duan_catalog::{EntityDescriptor, ItemId, Package, PackageId};
use example_naval_combat::Weapon;
use example_naval_core::{Faction, Health, Radar};
use example_naval_motion::{Position2, Velocity2};

use crate::Ship;

pub const PACKAGE_ID: &str = env!("CARGO_PKG_NAME");

pub fn package() -> Package {
    Package::builder(PackageId::new(PACKAGE_ID).expect("valid package id"))
        .entity(
            EntityDescriptor::new(ItemId::new(Ship::ITEM_ID).unwrap())
                .component(ItemId::new(Faction::ITEM_ID).unwrap())
                .component(ItemId::new(Position2::ITEM_ID).unwrap())
                .component(ItemId::new(Velocity2::ITEM_ID).unwrap())
                .component(ItemId::new(Health::ITEM_ID).unwrap())
                .component(ItemId::new(Radar::ITEM_ID).unwrap())
                .component(ItemId::new(Weapon::ITEM_ID).unwrap()),
        )
        .build()
}
