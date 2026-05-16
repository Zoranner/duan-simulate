pub mod components;
mod package;

pub use components::{Collider, Position2, StaticBody, Velocity2};
pub use package::package;

use duan::catalog::{FactoryRegistry, PackageResult};

pub fn register_factories(registry: FactoryRegistry) -> PackageResult<FactoryRegistry> {
    Ok(registry)
}
