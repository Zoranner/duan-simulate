use duan::catalog::Package;

pub fn package() -> Package {
    duan::catalog::collect_package!()
}
