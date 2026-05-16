pub mod components;
pub mod domains;
mod package;

pub use components::{Heading, Position2, Velocity2};
pub use domains::{CollisionDomain, MotionDomain};
pub use package::package;

use std::collections::BTreeMap;

use duan::{
    catalog::{DomainFactory, FactoryRegistry, PackageResult, Value},
    WorldBuilder,
};

pub fn register_factories(registry: FactoryRegistry) -> PackageResult<FactoryRegistry> {
    registry
        .with_domain(MotionDomainFactory)
        .and_then(|registry| registry.with_domain(CollisionDomainFactory))
}

struct MotionDomainFactory;

impl DomainFactory for MotionDomainFactory {
    fn item_id(&self) -> &'static str {
        MotionDomain::ITEM_ID
    }

    fn describe(&self) -> String {
        "naval motion domain".to_owned()
    }

    fn install(
        &self,
        builder: WorldBuilder,
        _options: &BTreeMap<String, Value>,
    ) -> PackageResult<WorldBuilder> {
        Ok(builder.domain(MotionDomain))
    }
}

struct CollisionDomainFactory;

impl DomainFactory for CollisionDomainFactory {
    fn item_id(&self) -> &'static str {
        CollisionDomain::ITEM_ID
    }

    fn describe(&self) -> String {
        "naval collision domain".to_owned()
    }

    fn install(
        &self,
        builder: WorldBuilder,
        _options: &BTreeMap<String, Value>,
    ) -> PackageResult<WorldBuilder> {
        Ok(builder.domain(CollisionDomain))
    }
}
