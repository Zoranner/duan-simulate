pub mod components;
pub mod domains;
mod package;

pub use components::Weapon;
pub use domains::{ApplyDamage, CombatDomain, FireRequested, HitResolved};
pub use package::package;

use std::collections::BTreeMap;

use duan::{
    catalog::{DomainFactory, FactoryRegistry, PackageResult, ReactionFactory, Value},
    WorldBuilder,
};

pub fn register_factories(registry: FactoryRegistry) -> PackageResult<FactoryRegistry> {
    registry
        .with_domain(CombatDomainFactory)
        .and_then(|registry| registry.with_reaction(ApplyDamageFactory))
}

struct CombatDomainFactory;

impl DomainFactory for CombatDomainFactory {
    fn item_id(&self) -> &'static str {
        CombatDomain::ITEM_ID
    }

    fn describe(&self) -> String {
        "naval combat domain".to_owned()
    }

    fn install(
        &self,
        builder: WorldBuilder,
        _options: &BTreeMap<String, Value>,
    ) -> PackageResult<WorldBuilder> {
        Ok(builder.domain(CombatDomain))
    }
}

struct ApplyDamageFactory;

impl ReactionFactory for ApplyDamageFactory {
    fn item_id(&self) -> &'static str {
        ApplyDamage::ITEM_ID
    }

    fn describe(&self) -> String {
        "naval damage reaction".to_owned()
    }

    fn install(
        &self,
        builder: WorldBuilder,
        _options: &BTreeMap<String, Value>,
    ) -> PackageResult<WorldBuilder> {
        Ok(builder.on::<HitResolved>(ApplyDamage))
    }
}
