use duan_catalog::{ComponentDescriptor, ItemId, Package, PackageId, Schema};

use crate::{Collider, Position2, StaticBody, Velocity2};

pub const PACKAGE_ID: &str = env!("CARGO_PKG_NAME");

pub fn package() -> Package {
    Package::builder(PackageId::new(PACKAGE_ID).expect("valid package id"))
        .component(component(Position2::ITEM_ID, Position2::schema()))
        .component(component(Velocity2::ITEM_ID, Velocity2::schema()))
        .component(component(StaticBody::ITEM_ID, StaticBody::schema()))
        .component(component(Collider::ITEM_ID, Collider::schema()))
        .build()
}

fn component(id: &str, schema: Schema) -> ComponentDescriptor {
    ComponentDescriptor::new(ItemId::new(id).expect("valid item id"), schema)
}
