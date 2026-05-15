use duan_package::{
    ComponentDescriptor, EntityDescriptor, ItemId, Package, PackageError, PackageId, Registry,
    Schema,
};
use duan_runner::{run_scenario, RunStatus, Runner};
use duan_scenario::load_yaml_str;

fn id(value: &str) -> ItemId {
    ItemId::new(value).unwrap()
}

fn tiny_registry() -> Registry {
    let package = Package::builder(PackageId::new("tiny-motion").unwrap())
        .component(ComponentDescriptor::new(
            id("tiny-motion/position"),
            Schema::default(),
        ))
        .component(ComponentDescriptor::new(
            id("tiny-motion/velocity"),
            Schema::default(),
        ))
        .component(ComponentDescriptor::new(
            id("tiny-motion/mass"),
            Schema::default(),
        ))
        .entity(
            EntityDescriptor::new(id("tiny-motion/body"))
                .component(id("tiny-motion/position"))
                .component(id("tiny-motion/velocity")),
        )
        .domain(id("tiny-motion/kinematics"))
        .reaction(id("tiny-motion/integrate"))
        .build();

    Registry::new().install(package).unwrap()
}

#[test]
fn planned_only_runner_reports_tiny_scenario_audit() {
    let registry = tiny_registry();
    let manifest = load_yaml_str(
        r#"
scenario:
  id: tiny_demo
  package: tiny-motion

packages:
  - id: tiny-motion
    version: 0.1.0

domains:
  - type: tiny-motion/kinematics

reactions:
  - type: tiny-motion/integrate

entities:
  - id: ball
    type: tiny-motion/body
    components:
      tiny-motion/position: { x: 1.0, y: 2.0 }

run:
  delta_time: 0.25
  max_time: 0.6
"#,
    )
    .unwrap();

    let runner = Runner::new(registry.clone());
    let plan = runner.plan(&manifest).unwrap();
    let report = run_scenario(&manifest, registry).unwrap();

    assert_eq!(plan.steps().len(), 5);
    assert_eq!(report.scenario_id, "tiny_demo");
    assert_eq!(report.planned_steps, 5);
    assert_eq!(report.executed_steps, 3);
    assert_eq!(report.simulated_time, 0.75);
    assert_eq!(report.entities_planned, 1);
    assert_eq!(report.status, RunStatus::PlannedOnly);
}

#[test]
fn planned_only_runner_propagates_missing_item_errors() {
    let runner = Runner::new(tiny_registry());
    let manifest = load_yaml_str(
        r#"
scenario:
  id: missing_demo
  package: tiny-motion

packages:
  - id: tiny-motion
    version: 0.1.0

domains:
  - type: tiny-motion/missing-domain
"#,
    )
    .unwrap();

    let error = runner.run(&manifest).unwrap_err();

    assert_eq!(
        error,
        PackageError::MissingItem {
            location: "domains[0].type".to_owned(),
            item_id: id("tiny-motion/missing-domain"),
        }
    );
}

#[test]
fn planned_only_runner_propagates_unsupported_component_override_errors() {
    let runner = Runner::new(tiny_registry());
    let manifest = load_yaml_str(
        r#"
scenario:
  id: unsupported_override_demo
  package: tiny-motion

packages:
  - id: tiny-motion
    version: 0.1.0

entities:
  - id: ball
    type: tiny-motion/body
    components:
      tiny-motion/mass: 3.0
"#,
    )
    .unwrap();

    let error = runner.run(&manifest).unwrap_err();

    assert_eq!(
        error,
        PackageError::UnsupportedComponentOverride {
            entity_id: "ball".to_owned(),
            entity_type: id("tiny-motion/body"),
            component_id: id("tiny-motion/mass"),
        }
    );
}
