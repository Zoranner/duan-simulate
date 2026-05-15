use duan_catalog::{ComponentDescriptor, ItemId, Package, PackageId, Schema};

use crate::{Faction, Health, Radar};

pub const PACKAGE_ID: &str = env!("CARGO_PKG_NAME");

pub fn package() -> Package {
    Package::builder(PackageId::new(PACKAGE_ID).expect("valid package id"))
        .component(component(Faction::ITEM_ID, Faction::schema()))
        .component(component(Health::ITEM_ID, Health::schema()))
        .component(component(Radar::ITEM_ID, Radar::schema()))
        .build()
}

fn component(id: &str, schema: Schema) -> ComponentDescriptor {
    ComponentDescriptor::new(ItemId::new(id).expect("valid item id"), schema)
}
