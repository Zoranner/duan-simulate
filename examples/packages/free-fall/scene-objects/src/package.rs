use duan_catalog::Package;

pub const PACKAGE_ID: &str = env!("CARGO_PKG_NAME");

pub fn package() -> Package {
    duan_catalog::collect_package!()
}
