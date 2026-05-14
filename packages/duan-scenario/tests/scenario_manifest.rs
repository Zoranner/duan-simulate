use duan_scenario::{load_yaml_str, Manifest, ManifestError, Value};

#[test]
fn loads_free_fall_example_manifest() {
    let manifest = load_yaml_str(include_str!("../../../examples/free-fall/scenario.yaml"))
        .expect("free-fall scenario should load");

    assert_eq!(manifest.scenario.id, "free_fall_demo");
    assert_eq!(manifest.packages.len(), 4);
    assert_eq!(manifest.domains.len(), 1);
    assert_eq!(manifest.entities.len(), 2);

    let ball = manifest
        .entities
        .iter()
        .find(|entity| entity.id == "ball")
        .expect("ball entity");
    let position = ball
        .components
        .get("duan.kinematics.position-2")
        .expect("position component");
    assert_eq!(
        position.get("y"),
        Some(&Value::Number(serde_yaml::Number::from(10.0)))
    );
}

#[test]
fn loads_naval_combat_example_manifest() {
    let manifest = load_yaml_str(include_str!("../../../examples/naval-combat/scenario.yaml"))
        .expect("naval-combat scenario should load");

    assert_eq!(manifest.scenario.id, "naval_combat_demo");
    assert_eq!(manifest.packages.len(), 5);
    assert_eq!(manifest.domains.len(), 3);
    assert_eq!(manifest.reactions.len(), 4);
    assert_eq!(manifest.entities.len(), 2);
}

#[test]
fn reports_structural_validation_errors() {
    let yaml = r#"
scenario:
  id: invalid_demo
  package: examples.invalid

packages:
  - id: duan.kinematics
    version: 0.1.0

domains:
  - type: examples.invalid.domains.motion
  - type: Bad_Item

entities:
  - id: ball
    type: examples.invalid.entities.ball
    components:
      duan.kinematics.position-2: { x: 0.0, y: 10.0 }
      examples.invalid.components.mass: 1.0
      invalid_component: true
  - id: ball
    type: duan.kinematics.body

run:
  output_dir: ../runs

outputs:
  frames: /tmp/frames.jsonl
"#;

    let err = load_yaml_str(yaml).expect_err("invalid manifest should fail validation");
    let ManifestError::Validation(errors) = err else {
        panic!("expected validation errors, got {err:?}");
    };

    let rendered = errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        rendered.contains("duplicate entity id `ball`"),
        "{rendered}"
    );
    assert!(
        rendered.contains("missing package dependency for `examples.invalid.domains.motion`"),
        "{rendered}"
    );
    assert!(
        rendered.contains("invalid item id `Bad_Item`"),
        "{rendered}"
    );
    assert!(
        rendered.contains("invalid component reference `invalid_component`"),
        "{rendered}"
    );
    assert!(
        rendered.contains("invalid run/output path `../runs`"),
        "{rendered}"
    );
    assert!(
        rendered.contains("invalid run/output path `/tmp/frames.jsonl`"),
        "{rendered}"
    );
}

#[test]
fn manifest_value_keeps_nested_yaml_shape() {
    let manifest: Manifest = load_yaml_str(
        r#"
scenario:
  id: value_demo
  package: examples.value

packages:
  - id: examples.value.components
    version: 0.1.0
  - id: examples.value.entities
    version: 0.1.0

entities:
  - id: actor
    type: examples.value.entities.actor
    components:
      examples.value.components.payload:
        enabled: true
        tags: [alpha, beta]
        nested:
          count: 2
"#,
    )
    .expect("value manifest should load");

    let payload = &manifest.entities[0].components["examples.value.components.payload"];
    assert_eq!(payload.get("enabled"), Some(&Value::Bool(true)));
    assert!(matches!(payload.get("tags"), Some(Value::Sequence(values)) if values.len() == 2));
    let nested_count = payload.get("nested").and_then(|value| value.get("count"));
    assert_eq!(nested_count, Some(&Value::Number(2.into())));
}
