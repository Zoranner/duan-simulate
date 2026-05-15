use duan_catalog::{ItemId, Package, PackageId};

use crate::GravityField;

pub const PACKAGE_ID: &str = env!("CARGO_PKG_NAME");

pub fn package() -> Package {
    Package::builder(PackageId::new(PACKAGE_ID).expect("valid package id"))
        .domain(ItemId::new(GravityField::ITEM_ID).expect("valid item id"))
        .build()
}
