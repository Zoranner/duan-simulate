pub mod entities;
mod package;

pub use entities::Ship;
pub use package::package;

use std::collections::BTreeMap;

use duan::{
    catalog::{EntityFactory, FactoryRegistry, PackageError, PackageResult, Value},
    World,
};

pub fn register_factories(registry: FactoryRegistry) -> PackageResult<FactoryRegistry> {
    registry.with_entity(ShipFactory)
}

struct ShipFactory;

impl EntityFactory for ShipFactory {
    fn item_id(&self) -> &'static str {
        Ship::ITEM_ID
    }

    fn describe(&self) -> String {
        "naval ship".to_owned()
    }

    fn spawn(&self, world: &mut World, components: &BTreeMap<String, Value>) -> PackageResult<()> {
        let faction = faction_from_components(components, "example-naval-core/faction")?
            .unwrap_or_else(example_naval_core::Faction::red);
        let position = position_from_components(components, "example-naval-motion/position-2")?
            .unwrap_or_default();
        let velocity = velocity_from_components(components, "example-naval-motion/velocity-2")?
            .unwrap_or_default();
        let health = health_from_components(components, "example-naval-core/health")?
            .unwrap_or_else(|| example_naval_core::Health::new(100.0));
        let radar = radar_from_components(components, "example-naval-core/radar")?
            .unwrap_or_else(|| example_naval_core::Radar::new(260.0));
        let weapon = weapon_from_components(components, "example-naval-combat/weapon")?
            .unwrap_or_else(|| example_naval_combat::Weapon::new(180.0, 25.0, 80.0, 1.5));

        world.spawn_with::<Ship>(Ship::bundle(
            faction, position, velocity, health, radar, weapon,
        ));
        Ok(())
    }
}

fn faction_from_components(
    components: &BTreeMap<String, Value>,
    id: &str,
) -> PackageResult<Option<example_naval_core::Faction>> {
    let Some(value) = components.get(id) else {
        return Ok(None);
    };
    Ok(Some(example_naval_core::Faction {
        team: u8_field(value, "team")?,
    }))
}

fn health_from_components(
    components: &BTreeMap<String, Value>,
    id: &str,
) -> PackageResult<Option<example_naval_core::Health>> {
    let Some(value) = components.get(id) else {
        return Ok(None);
    };
    Ok(Some(example_naval_core::Health {
        current: f64_field(value, "current")?,
        max: f64_field(value, "max")?,
    }))
}

fn radar_from_components(
    components: &BTreeMap<String, Value>,
    id: &str,
) -> PackageResult<Option<example_naval_core::Radar>> {
    let Some(value) = components.get(id) else {
        return Ok(None);
    };
    Ok(Some(example_naval_core::Radar::new(f64_field(
        value, "range",
    )?)))
}

fn position_from_components(
    components: &BTreeMap<String, Value>,
    id: &str,
) -> PackageResult<Option<example_naval_motion::Position2>> {
    let Some(value) = components.get(id) else {
        return Ok(None);
    };
    Ok(Some(example_naval_motion::Position2::new(
        f64_field(value, "x")?,
        f64_field(value, "y")?,
    )))
}

fn velocity_from_components(
    components: &BTreeMap<String, Value>,
    id: &str,
) -> PackageResult<Option<example_naval_motion::Velocity2>> {
    let Some(value) = components.get(id) else {
        return Ok(None);
    };
    Ok(Some(example_naval_motion::Velocity2::new(
        f64_field(value, "vx")?,
        f64_field(value, "vy")?,
    )))
}

fn weapon_from_components(
    components: &BTreeMap<String, Value>,
    id: &str,
) -> PackageResult<Option<example_naval_combat::Weapon>> {
    let Some(value) = components.get(id) else {
        return Ok(None);
    };
    Ok(Some(example_naval_combat::Weapon::new(
        f64_field(value, "range")?,
        f64_field(value, "damage")?,
        f64_field(value, "missile_speed")?,
        f64_field(value, "fire_cooldown")?,
    )))
}

fn f64_field(value: &Value, field: &str) -> PackageResult<f64> {
    value
        .as_mapping()
        .and_then(|mapping| mapping.get(Value::from(field)))
        .and_then(Value::as_f64)
        .ok_or_else(|| PackageError::InvalidScenarioValue {
            location: field.to_owned(),
            expected: "number",
        })
}

fn u8_field(value: &Value, field: &str) -> PackageResult<u8> {
    value
        .as_mapping()
        .and_then(|mapping| mapping.get(Value::from(field)))
        .and_then(Value::as_u64)
        .and_then(|value| u8::try_from(value).ok())
        .ok_or_else(|| PackageError::InvalidScenarioValue {
            location: field.to_owned(),
            expected: "u8",
        })
}
