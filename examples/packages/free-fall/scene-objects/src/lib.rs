pub mod entities;
mod package;

pub use entities::{Ball, Ground};
pub use package::package;

pub fn register_factories(
    registry: duan::catalog::FactoryRegistry,
) -> duan::catalog::PackageResult<duan::catalog::FactoryRegistry> {
    registry
        .with_entity(BallFactory)
        .and_then(|registry| registry.with_entity(GroundFactory))
}

struct BallFactory;

impl duan::catalog::EntityFactory for BallFactory {
    fn item_id(&self) -> &'static str {
        Ball::ITEM_ID
    }

    fn describe(&self) -> String {
        "free-fall ball".to_owned()
    }

    fn spawn(
        &self,
        world: &mut duan::World,
        components: &std::collections::BTreeMap<String, duan::catalog::Value>,
    ) -> duan::catalog::PackageResult<()> {
        let position = position_from_components(components, "example-freefall-physics/position-2")?
            .unwrap_or_else(|| example_freefall_physics::Position2::new(0.0, 10.0));
        let velocity = velocity_from_components(components, "example-freefall-physics/velocity-2")?
            .unwrap_or_default();
        world.spawn_with::<Ball>((position, velocity));
        Ok(())
    }
}

struct GroundFactory;

impl duan::catalog::EntityFactory for GroundFactory {
    fn item_id(&self) -> &'static str {
        Ground::ITEM_ID
    }

    fn describe(&self) -> String {
        "free-fall ground".to_owned()
    }

    fn spawn(
        &self,
        world: &mut duan::World,
        components: &std::collections::BTreeMap<String, duan::catalog::Value>,
    ) -> duan::catalog::PackageResult<()> {
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
    components: &std::collections::BTreeMap<String, duan::catalog::Value>,
    id: &str,
) -> duan::catalog::PackageResult<Option<example_freefall_physics::Position2>> {
    let Some(value) = components.get(id) else {
        return Ok(None);
    };
    Ok(Some(example_freefall_physics::Position2::new(
        f64_field(value, "x")?,
        f64_field(value, "y")?,
    )))
}

fn velocity_from_components(
    components: &std::collections::BTreeMap<String, duan::catalog::Value>,
    id: &str,
) -> duan::catalog::PackageResult<Option<example_freefall_physics::Velocity2>> {
    let Some(value) = components.get(id) else {
        return Ok(None);
    };
    Ok(Some(example_freefall_physics::Velocity2::new(
        f64_field(value, "vx")?,
        f64_field(value, "vy")?,
    )))
}

fn collider_from_components(
    components: &std::collections::BTreeMap<String, duan::catalog::Value>,
    id: &str,
) -> duan::catalog::PackageResult<Option<example_freefall_physics::Collider>> {
    let Some(value) = components.get(id) else {
        return Ok(None);
    };
    Ok(Some(example_freefall_physics::Collider::new(f64_field(
        value,
        "restitution",
    )?)))
}

fn f64_field(value: &duan::catalog::Value, field: &str) -> duan::catalog::PackageResult<f64> {
    value
        .as_mapping()
        .and_then(|mapping| mapping.get(duan::catalog::Value::from(field)))
        .and_then(duan::catalog::Value::as_f64)
        .ok_or_else(|| duan::catalog::PackageError::InvalidScenarioValue {
            location: field.to_owned(),
            expected: "number",
        })
}
