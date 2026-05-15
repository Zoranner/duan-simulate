# DUAN Package Authoring

DUAN package authoring is Rust-first. A DUAN package is an ordinary Cargo package. Runtime behavior, Rust types, factories, item registration, schema shape, and display metadata are authored in `src/**`.

Scenario manifests are not a DSL. They select package items and provide initial values. Algorithms, scheduling behavior, domain computation, reaction handling, spawning, event emission, and event reaction stay in Rust.

## Package Identity

The DUAN package id is the Cargo `package.name`. Do not duplicate it in `[package.metadata.duan]`, `duan.toml`, `build.rs`, or a generated environment variable.

```toml
[package]
name = "examples-naval-combat-platform"
version = "0.1.0"
edition = "2021"
```

Package item ids use `<package-id>/<local-name>`.

```rust
pub const ITEM_ID: &str = concat!(env!("CARGO_PKG_NAME"), "/health");
```

The item id does not include a type segment such as `component`, `entity`, or `domain`. Item kind comes from the registration API:

```rust
Package::builder(PackageId::new(env!("CARGO_PKG_NAME")).expect("valid package id"))
    .component(ComponentDescriptor::new(ItemId::new(Health::ITEM_ID).unwrap(), Health::schema()))
    .domain(ItemId::new(MotionDomain::ITEM_ID).unwrap())
    .entity(EntityDescriptor::new(ItemId::new(Ship::ITEM_ID).unwrap()))
    .event(ItemId::new(HitResolved::ITEM_ID).unwrap())
    .reaction(ItemId::new(ApplyDamage::ITEM_ID).unwrap())
    .build()
```

## Source Layout

A source package should include:

- `Cargo.toml`: Cargo package identity, version, and dependencies.
- `src/lib.rs`: public exports for generated runners and dependent packages.
- `src/package.rs`: the package registration entry.
- `src/components/**`, `src/entities/**`, `src/domains/**`, `src/events/**`, `src/reactions/**`: named Rust modules when the package owns those capabilities.

Do not hand-author `duan.toml` or `schemas/**` inside the source package. Those are generated cache files produced by install/publish tooling from the Rust registration and schema APIs.

## Schema And Display Metadata

Schemas are authored in Rust next to the type they describe:

```rust
impl Health {
    pub const ITEM_ID: &str = concat!(env!("CARGO_PKG_NAME"), "/health");

    pub fn schema() -> Schema {
        Schema::new()
            .field(
                "current",
                FieldSchema::new(PrimitiveKind::Float)
                    .default(PrimitiveValue::Float(100.0))
                    .range(Range::new(Some(0.0), None))
                    .display(DisplayMetadata::new().label("Current").control("number")),
            )
            .field(
                "max",
                FieldSchema::new(PrimitiveKind::Float)
                    .default(PrimitiveValue::Float(100.0))
                    .range(Range::new(Some(1.0), None))
                    .display(DisplayMetadata::new().label("Max").control("number")),
            )
    }
}
```

The first schema version records item id, item kind, primitive field metadata, defaults, ranges, units, and display metadata for authoring tools. Rust code remains responsible for final parsing and behavior.

## Generated Install Cache

After a package is installed or published, tooling may generate a metadata cache:

```toml
[duan.package]
id = "examples-naval-combat-platform"
version = "0.1.0"
name = "Naval Combat Platform"

[duan.rust]
crate = "examples-naval-combat-platform"

[provides.components]
"examples-naval-combat-platform/health" = "schemas/health.json"
```

That cache exists so editors, validators, delivery packagers, and offline tools can inspect packages without treating Rust source files as the cache format. It is not the source of truth.

## Split Packages

Split packages by capability ownership and release boundary, not by mechanical file type. A package may contain components, entities, domains, events, and reactions together when they are shipped as one capability. Separate packages when scenarios need to depend on them independently or when reuse boundaries differ.

In this repository:

- `examples/packages/free-fall/body` owns body state components.
- `examples/packages/free-fall/gravity` owns the gravity domain.
- `examples/packages/free-fall/scene-objects` owns free-fall entity templates.
- `examples/packages/naval-combat/platform` owns shared platform components.
- `examples/packages/naval-combat/maneuver` owns movement components and domains.
- `examples/packages/naval-combat/engagement` owns weapons, combat events, combat domain, and damage reaction.
- `examples/packages/naval-combat/fleet-objects` owns fleet entity templates.

Framework crates under `packages/` provide platform mechanisms. Example simulation capabilities stay under `examples/packages/` unless they are deliberately extracted into separately versioned product crates.

## Scenario Assembly Boundary

`*.duan` scenario manifests should only assemble existing Rust package capabilities:

- package dependencies and versions;
- domains and reactions to install;
- entity types to spawn;
- component values to attach or override;
- run options and output paths.

If a scenario needs a new algorithm, state transition, event, or command behavior, add it to a Rust package first, expose it through `package()`, then reference the item id from the scenario.
