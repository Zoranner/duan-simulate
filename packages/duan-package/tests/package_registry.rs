use duan_package::{
    ComponentDescriptor, DisplayMetadata, EntityDescriptor, FieldSchema, ItemId, Package,
    PackageDependency, PackageId, PrimitiveKind, PrimitiveValue, Range, Registry, Schema, Unit,
    VersionReqText,
};

#[test]
fn ids_accept_cargo_package_names_and_slash_item_names() {
    assert_eq!(
        PackageId::new("examples-motion").unwrap().as_str(),
        "examples-motion"
    );
    assert_eq!(
        ItemId::new("examples-motion/position-2").unwrap().as_str(),
        "examples-motion/position-2"
    );
}

#[test]
fn package_ids_reject_non_cargo_package_names() {
    for value in [
        "",
        "Examples-motion",
        "duan/kinematics",
        "duan.physics",
        "duan_physics",
        "-duan",
        "duan-",
        "duan--physics",
    ] {
        assert!(PackageId::new(value).is_err(), "{value} should be invalid");
    }
}

#[test]
fn item_ids_reject_invalid_package_or_local_names() {
    for value in [
        "",
        "Examples-motion/position",
        "duan-motion",
        "duan-motion.position",
        "duan-motion//position",
        "/position",
        "duan-motion/",
        "duan-motion/position/x",
        "duan/physics_2",
        "duan/-physics",
        "duan/physics-",
        "duan/physics--body",
    ] {
        assert!(ItemId::new(value).is_err(), "{value} should be invalid");
    }
}

#[test]
fn registry_rejects_duplicate_items_across_packages() {
    let component_id = ItemId::new("examples-motion/position").unwrap();
    let first = Package::builder(PackageId::new("examples-motion").unwrap())
        .version("0.1.0")
        .component(ComponentDescriptor::new(
            component_id.clone(),
            Schema::default(),
        ))
        .build();
    let second = Package::builder(PackageId::new("duan-motion").unwrap())
        .version("0.1.0")
        .component(ComponentDescriptor::new(
            component_id.clone(),
            Schema::default(),
        ))
        .build();

    let mut registry = Registry::default();
    registry.install_package(first).unwrap();

    let error = registry.install_package(second).unwrap_err();
    assert_eq!(error.duplicate_item_id(), Some(&component_id));
}

#[test]
fn registry_rejects_duplicate_items_inside_same_package() {
    let component_id = ItemId::new("examples-motion/position").unwrap();
    let package = Package::builder(PackageId::new("examples-motion").unwrap())
        .version("0.1.0")
        .component(ComponentDescriptor::new(
            component_id.clone(),
            Schema::default(),
        ))
        .component(ComponentDescriptor::new(
            component_id.clone(),
            Schema::default(),
        ))
        .build();

    let mut registry = Registry::default();
    let error = registry.install_package(package).unwrap_err();

    assert_eq!(error.duplicate_item_id(), Some(&component_id));
}

#[test]
fn registry_supports_package_lookup_and_item_lookup() {
    let package_id = PackageId::new("examples-motion").unwrap();
    let component_id = ItemId::new("examples-motion/position").unwrap();
    let entity_id = ItemId::new("examples-motion/body").unwrap();
    let domain_id = ItemId::new("examples-motion/integrator").unwrap();
    let event_id = ItemId::new("examples-motion/collision").unwrap();
    let reaction_id = ItemId::new("examples-motion/bounce").unwrap();
    let observer_id = ItemId::new("examples-motion/trace").unwrap();

    let package = Package::builder(package_id.clone())
        .version("0.1.0")
        .component(ComponentDescriptor::new(
            component_id.clone(),
            Schema::default(),
        ))
        .entity(EntityDescriptor::new(entity_id.clone()))
        .domain(domain_id.clone())
        .event(event_id.clone())
        .reaction(reaction_id.clone())
        .observer(observer_id.clone())
        .build();

    let mut registry = Registry::default();
    registry.install_package(package).unwrap();

    assert_eq!(registry.package(&package_id).unwrap().id(), &package_id);
    assert!(registry.item(&component_id).unwrap().is_component());
    assert!(registry.item(&entity_id).unwrap().is_entity());
    assert!(registry.item(&domain_id).unwrap().is_domain());
    assert!(registry.item(&event_id).unwrap().is_event());
    assert!(registry.item(&reaction_id).unwrap().is_reaction());
    assert!(registry.item(&observer_id).unwrap().is_observer());
}

#[test]
fn package_records_dependency_declarations() {
    let dependency = PackageDependency::new(
        PackageId::new("examples-motion").unwrap(),
        VersionReqText::new("^0.1").unwrap(),
    );

    let package = Package::builder(PackageId::new("duan-motion").unwrap())
        .version("0.1.0")
        .dependency(dependency.clone())
        .build();

    assert_eq!(package.dependencies(), &[dependency]);
}

#[test]
fn schema_metadata_roundtrips_through_component_registration() {
    let schema = Schema::new()
        .field(
            "speed",
            FieldSchema::new(PrimitiveKind::Float)
                .default(PrimitiveValue::Float(12.5))
                .range(Range::new(Some(0.0), Some(100.0)))
                .unit(Unit::new("m/s"))
                .display(
                    DisplayMetadata::new()
                        .label("Speed")
                        .description("Initial speed")
                        .order(1),
                ),
        )
        .field(
            "enabled",
            FieldSchema::new(PrimitiveKind::Bool).default(PrimitiveValue::Bool(true)),
        );
    let component_id = ItemId::new("examples-motion/velocity").unwrap();
    let package = Package::builder(PackageId::new("examples-motion").unwrap())
        .version("0.1.0")
        .component(ComponentDescriptor::new(
            component_id.clone(),
            schema.clone(),
        ))
        .build();
    let mut registry = Registry::default();

    registry.install_package(package).unwrap();

    let component = registry.item(&component_id).unwrap().component().unwrap();
    assert_eq!(component.schema(), &schema);
    assert_eq!(
        component
            .schema()
            .field_schema("speed")
            .unwrap()
            .display_metadata()
            .unwrap()
            .label_text(),
        Some("Speed")
    );
}

#[test]
fn registry_supports_chain_install_for_generated_runners() {
    let package = Package::builder(PackageId::new("examples-motion").unwrap())
        .version("0.1.0")
        .build();

    let registry = Registry::new().install(package).unwrap();

    assert!(registry
        .package(&PackageId::new("examples-motion").unwrap())
        .is_some());
}
