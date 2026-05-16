pub mod components;
pub mod domains;
mod package;

pub use components::{Heading, Position2, Velocity2};
pub use domains::{CollisionDomain, MotionDomain};
pub use package::package;

pub fn register_factories(
    registry: duan::catalog::FactoryRegistry,
) -> duan::catalog::PackageResult<duan::catalog::FactoryRegistry> {
    registry
        .with_domain(MotionDomainFactory)
        .and_then(|registry| registry.with_domain(CollisionDomainFactory))
}

struct MotionDomainFactory;

impl duan::catalog::DomainFactory for MotionDomainFactory {
    fn item_id(&self) -> &'static str {
        MotionDomain::ITEM_ID
    }

    fn describe(&self) -> String {
        "naval motion domain".to_owned()
    }

    fn install(
        &self,
        builder: duan::WorldBuilder,
        _options: &std::collections::BTreeMap<String, duan::catalog::Value>,
    ) -> duan::catalog::PackageResult<duan::WorldBuilder> {
        Ok(builder.domain(MotionDomain))
    }
}

struct CollisionDomainFactory;

impl duan::catalog::DomainFactory for CollisionDomainFactory {
    fn item_id(&self) -> &'static str {
        CollisionDomain::ITEM_ID
    }

    fn describe(&self) -> String {
        "naval collision domain".to_owned()
    }

    fn install(
        &self,
        builder: duan::WorldBuilder,
        _options: &std::collections::BTreeMap<String, duan::catalog::Value>,
    ) -> duan::catalog::PackageResult<duan::WorldBuilder> {
        Ok(builder.domain(CollisionDomain))
    }
}
