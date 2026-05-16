pub mod components;
mod package;

pub use components::{Collider, Position2, StaticBody, Velocity2};
pub use package::package;

pub fn register_factories(
    registry: duan::catalog::FactoryRegistry,
) -> duan::catalog::PackageResult<duan::catalog::FactoryRegistry> {
    Ok(registry)
}
