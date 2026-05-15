use duan_package::{EntityDescriptor, ItemId, Package, PackageId};
use examples_free_fall_body::{Collider, Position2, StaticBody, Velocity2};

use crate::{Ball, Ground};

pub const PACKAGE_ID: &str = env!("CARGO_PKG_NAME");

pub fn package() -> Package {
    Package::builder(PackageId::new(PACKAGE_ID).expect("valid package id"))
        .entity(
            EntityDescriptor::new(ItemId::new(Ground::ITEM_ID).unwrap())
                .component(ItemId::new(Position2::ITEM_ID).unwrap())
                .component(ItemId::new(StaticBody::ITEM_ID).unwrap())
                .component(ItemId::new(Collider::ITEM_ID).unwrap()),
        )
        .entity(
            EntityDescriptor::new(ItemId::new(Ball::ITEM_ID).unwrap())
                .component(ItemId::new(Position2::ITEM_ID).unwrap())
                .component(ItemId::new(Velocity2::ITEM_ID).unwrap()),
        )
        .build()
}
