pub mod components;
pub mod domains;
mod package;

pub use components::Weapon;
pub use domains::{ApplyDamage, CombatDomain, FireRequested, HitResolved};
pub use package::package;

pub fn register_factories(
    registry: duan::catalog::FactoryRegistry,
) -> duan::catalog::PackageResult<duan::catalog::FactoryRegistry> {
    registry
        .with_domain(CombatDomainFactory)
        .and_then(|registry| registry.with_reaction(ApplyDamageFactory))
}

struct CombatDomainFactory;

impl duan::catalog::DomainFactory for CombatDomainFactory {
    fn item_id(&self) -> &'static str {
        CombatDomain::ITEM_ID
    }

    fn describe(&self) -> String {
        "naval combat domain".to_owned()
    }

    fn install(
        &self,
        builder: duan::WorldBuilder,
        _options: &std::collections::BTreeMap<String, duan::catalog::Value>,
    ) -> duan::catalog::PackageResult<duan::WorldBuilder> {
        Ok(builder.domain(CombatDomain))
    }
}

struct ApplyDamageFactory;

impl duan::catalog::ReactionFactory for ApplyDamageFactory {
    fn item_id(&self) -> &'static str {
        ApplyDamage::ITEM_ID
    }

    fn describe(&self) -> String {
        "naval damage reaction".to_owned()
    }

    fn install(
        &self,
        builder: duan::WorldBuilder,
        _options: &std::collections::BTreeMap<String, duan::catalog::Value>,
    ) -> duan::catalog::PackageResult<duan::WorldBuilder> {
        Ok(builder.on::<HitResolved>(ApplyDamage))
    }
}
