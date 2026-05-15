use duan_package::{
    AssemblyPlan, AssemblyStep, ComponentDescriptor, EntityDescriptor, ItemId, Package,
    PackageError, PackageId, Registry, Schema,
};
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

fn tiny_manifest() -> duan_scenario::Manifest {
    load_yaml_str(
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
"#,
    )
    .unwrap()
}

#[test]
fn assembly_plan_emits_ordered_steps_for_tiny_scenario() {
    let registry = tiny_registry();
    let manifest = tiny_manifest();

    let plan = AssemblyPlan::from_manifest(&manifest, &registry).unwrap();

    assert_eq!(
        plan.steps(),
        &[
            AssemblyStep::InstallDomain {
                item_id: id("tiny-motion/kinematics")
            },
            AssemblyStep::InstallReaction {
                item_id: id("tiny-motion/integrate")
            },
            AssemblyStep::BuildWorld,
            AssemblyStep::SpawnEntity {
                entity_id: "ball".to_owned(),
                item_id: id("tiny-motion/body")
            },
            AssemblyStep::ApplyComponentOverride {
                entity_id: "ball".to_owned(),
                component_id: id("tiny-motion/position")
            },
        ]
    );
}

#[test]
fn assembly_plan_reports_missing_package_item() {
    let registry = tiny_registry();
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

    let error = AssemblyPlan::from_manifest(&manifest, &registry).unwrap_err();

    assert_eq!(
        error,
        PackageError::MissingItem {
            location: "domains[0].type".to_owned(),
            item_id: id("tiny-motion/missing-domain"),
        }
    );
}

#[test]
fn assembly_plan_rejects_component_override_not_supported_by_entity_descriptor() {
    let registry = tiny_registry();
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

    let error = AssemblyPlan::from_manifest(&manifest, &registry).unwrap_err();

    assert_eq!(
        error,
        PackageError::UnsupportedComponentOverride {
            entity_id: "ball".to_owned(),
            entity_type: id("tiny-motion/body"),
            component_id: id("tiny-motion/mass"),
        }
    );
}
