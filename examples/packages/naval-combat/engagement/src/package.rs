use duan_catalog::Package;

pub fn package() -> Package {
    duan_catalog::collect_package!()
}
