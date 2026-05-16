pub mod entities;
mod package;

pub use entities::{Ball, Ground};
pub use package::package;

use std::collections::BTreeMap;

use duan::{
    catalog::{EntityFactory, FactoryRegistry, PackageError, PackageResult, Value},
    World,
};

pub fn register_factories(registry: FactoryRegistry) -> PackageResult<FactoryRegistry> {
    registry
        .with_entity(BallFactory)
        .and_then(|registry| registry.with_entity(GroundFactory))
}

struct BallFactory;

impl EntityFactory for BallFactory {
    fn item_id(&self) -> &'static str {
        Ball::ITEM_ID
    }

    fn describe(&self) -> String {
        "free-fall ball".to_owned()
    }

    fn spawn(&self, world: &mut World, components: &BTreeMap<String, Value>) -> PackageResult<()> {
        let position = position_from_components(components, "example-freefall-physics/position-2")?
            .unwrap_or_else(|| example_freefall_physics::Position2::new(0.0, 10.0));
        let velocity = velocity_from_components(components, "example-freefall-physics/velocity-2")?
            .unwrap_or_default();
        world.spawn_with::<Ball>((position, velocity));
        Ok(())
    }
}

struct GroundFactory;

impl EntityFactory for GroundFactory {
    fn item_id(&self) -> &'static str {
        Ground::ITEM_ID
    }

    fn describe(&self) -> String {
        "free-fall ground".to_owned()
    }

    fn spawn(&self, world: &mut World, components: &BTreeMap<String, Value>) -> PackageResult<()> {
        let position = position_from_components(components, "example-freefall-physics/position-2")?
            .unwrap_or_else(|| example_freefall_physics::Position2::new(0.0, 0.0));
        let static_body = example_freefall_physics::StaticBody::enabled();
        let collider = collider_from_components(components, "example-freefall-physics/collider")?
            .unwrap_or_else(|| example_freefall_physics::Collider::new(0.8));
        world.spawn_with::<Ground>((position, static_body, collider));
        Ok(())
    }
}

fn position_from_components(
    components: &BTreeMap<String, Value>,
    id: &str,
) -> PackageResult<Option<example_freefall_physics::Position2>> {
    let Some(value) = components.get(id) else {
        return Ok(None);
    };
    Ok(Some(example_freefall_physics::Position2::new(
        f64_field(value, "x")?,
        f64_field(value, "y")?,
    )))
}

fn velocity_from_components(
    components: &BTreeMap<String, Value>,
    id: &str,
) -> PackageResult<Option<example_freefall_physics::Velocity2>> {
    let Some(value) = components.get(id) else {
        return Ok(None);
    };
    Ok(Some(example_freefall_physics::Velocity2::new(
        f64_field(value, "vx")?,
        f64_field(value, "vy")?,
    )))
}

fn collider_from_components(
    components: &BTreeMap<String, Value>,
    id: &str,
) -> PackageResult<Option<example_freefall_physics::Collider>> {
    let Some(value) = components.get(id) else {
        return Ok(None);
    };
    Ok(Some(example_freefall_physics::Collider::new(f64_field(
        value,
        "restitution",
    )?)))
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
