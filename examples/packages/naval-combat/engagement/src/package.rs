use duan_catalog::{ComponentDescriptor, ItemId, Package, PackageId, Schema};

use crate::{ApplyDamage, CombatDomain, FireRequested, HitResolved, Weapon};

pub const PACKAGE_ID: &str = env!("CARGO_PKG_NAME");

pub fn package() -> Package {
    Package::builder(PackageId::new(PACKAGE_ID).expect("valid package id"))
        .component(component(Weapon::ITEM_ID, Weapon::schema()))
        .domain(ItemId::new(CombatDomain::ITEM_ID).unwrap())
        .event(ItemId::new(FireRequested::ITEM_ID).unwrap())
        .event(ItemId::new(HitResolved::ITEM_ID).unwrap())
        .reaction(ItemId::new(ApplyDamage::ITEM_ID).unwrap())
        .build()
}

fn component(id: &str, schema: Schema) -> ComponentDescriptor {
    ComponentDescriptor::new(ItemId::new(id).expect("valid item id"), schema)
}
