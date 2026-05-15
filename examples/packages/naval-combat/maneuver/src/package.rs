use duan_package::{ComponentDescriptor, ItemId, Package, PackageId, Schema};

use crate::{CollisionDomain, Heading, MotionDomain, Position2, Velocity2};

pub const PACKAGE_ID: &str = env!("CARGO_PKG_NAME");

pub fn package() -> Package {
    Package::builder(PackageId::new(PACKAGE_ID).expect("valid package id"))
        .component(component(Position2::ITEM_ID, Position2::schema()))
        .component(component(Velocity2::ITEM_ID, Velocity2::schema()))
        .component(component(Heading::ITEM_ID, Heading::schema()))
        .domain(ItemId::new(MotionDomain::ITEM_ID).unwrap())
        .domain(ItemId::new(CollisionDomain::ITEM_ID).unwrap())
        .build()
}

fn component(id: &str, schema: Schema) -> ComponentDescriptor {
    ComponentDescriptor::new(ItemId::new(id).expect("valid item id"), schema)
}
