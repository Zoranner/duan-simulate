pub mod components;
mod package;

pub use components::{Faction, Health, Radar};
pub use package::package;

pub fn register_factories(
    registry: duan::catalog::FactoryRegistry,
) -> duan::catalog::PackageResult<duan::catalog::FactoryRegistry> {
    Ok(registry)
}
