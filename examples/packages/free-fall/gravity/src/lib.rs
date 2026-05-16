pub mod domains;
mod package;

pub use domains::GravityField;
pub use package::package;

use std::collections::BTreeMap;

use duan::{
    catalog::{DomainFactory, FactoryRegistry, PackageResult, Value},
    WorldBuilder,
};

pub fn register_factories(registry: FactoryRegistry) -> PackageResult<FactoryRegistry> {
    registry.with_domain(GravityFieldFactory)
}

struct GravityFieldFactory;

impl DomainFactory for GravityFieldFactory {
    fn item_id(&self) -> &'static str {
        "example-freefall-gravity/earth"
    }

    fn describe(&self) -> String {
        "free-fall gravity field".to_owned()
    }

    fn install(
        &self,
        builder: WorldBuilder,
        _options: &BTreeMap<String, Value>,
    ) -> PackageResult<WorldBuilder> {
        Ok(builder.domain(GravityField::earth()))
    }
}
