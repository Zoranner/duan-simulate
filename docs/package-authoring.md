# DUAN Package Authoring

This document describes package authoring details. The platform-wide rules are consolidated in [DUAN Platform Long-Term Design](platform-design.md), and scenario project structure is defined in [DUAN Scenario Projects](scenario-project.md).

DUAN package authoring is Rust-first. A DUAN package is an ordinary Cargo package. Runtime behavior, Rust types, factories, item registration, schema shape, and display metadata are authored in `src/**`.

Scenario manifests are not a DSL. They select package items and provide initial values. Algorithms, scheduling behavior, domain computation, reaction handling, spawning, event emission, and event reaction stay in Rust.

This document describes the authoring surface for package authors. The example packages use this macro-assisted shape directly through the `duan` facade: metadata stays beside the Rust declaration, macros are imported from `duan`, and package entry points collect the current Cargo package through `duan::catalog::collect_package!()`.

## Package Identity

The DUAN package id is the Cargo `package.name`. Do not duplicate it in `[package.metadata.duan]`, `duan.toml`, `build.rs`, or a generated environment variable.

```toml
[package]
name = "example-naval-core"
version = "0.1.0"
edition = "2021"
```

Package item ids use `<package-id>/<id>`.

```text
example-naval-core/health
```

The `id` value in Rust attributes is the stable item id segment inside the current package. It is not a display name. Display text belongs in `label`.

Item ids do not include type segments such as `component`, `entity`, `domain`, `event`, or `reaction`. Item kind comes from the Rust item annotation and generated registration descriptor.

## Authoring Model

Package authors should not maintain a central item list by hand. Each annotated item registers itself with the package metadata collector. The package entry point only collects the current Cargo package:

```rust
pub fn package() -> Package {
    duan::catalog::collect_package!()
}
```

The package boundary still exists. Cargo package name, version, dependencies, and generated install cache all remain package-scoped. Users should not separately list events, reactions, domains, and entities in `package.rs`; annotated items are collected from the package. What disappears is the manual registry chain that is easy to forget:

```rust
Package::builder(...)
    .component(...)
    .domain(...)
    .entity(...)
    .build()
```

Advanced users and generated code may still use explicit builder and descriptor APIs. Macros are the authoring surface, not the only framework contract.

## Components

Components are data, so they use derive and field attributes. The derive generates the DUAN runtime component implementation, package item metadata, and schema descriptor.

```rust
#[derive(Component, Debug, Clone, Default, PartialEq)]
#[component(id = "position-2", kind = "reality", label = "Position")]
pub struct Position2 {
    #[field(label = "X", default = 0.0, range = -10000.0..=10000.0, unit = "m")]
    pub x: f64,

    #[field(label = "Y", default = 0.0, range = -10000.0..=10000.0, unit = "m")]
    pub y: f64,
}
```

`kind` maps to DUAN runtime component semantics:

- `belief`: entity-private state.
- `intent`: entity-authored public intent.
- `reality`: domain-authored world fact.

Schema metadata is intentionally close to fields. Field type, default, range, unit, and label are facts about the component, not facts about the scenario file.

Complex components may opt out of generated field schema and provide explicit schema code when the attribute form would become unclear.

## Entities

Entities can have behavior and spawn bundles, so entity annotation belongs on the explicit `impl Entity` block. The attribute supplies package metadata and declares which component types this entity supports for scenario assembly.

```rust
pub struct Ball;

#[entity(id = "ball", label = "Ball", components(Position2, Velocity2))]
impl Entity for Ball {
    fn bundle() -> impl ComponentBundle + Send + 'static {
        (Position2::default(), Velocity2::default())
    }
}
```

If an entity has no behavior or default bundle, the impl can stay minimal:

```rust
pub struct Ground;

#[entity(id = "ground", components(Position2, StaticBody, Collider))]
impl Entity for Ground {}
```

The `components(...)` list is not a package registration list. It is the entity capability contract used to reject unsupported scenario component overrides.

## Domains

Domains are behavior, so `compute()` stays plain Rust. The domain attribute only supplies item id, package metadata, and the type-level read/write/dependency sets.

```rust
pub struct GravityField {
    acceleration: f64,
}

impl GravityField {
    pub fn earth() -> Self {
        Self { acceleration: 9.8 }
    }
}

#[domain(
    id = "field",
    label = "Gravity Field",
    writes(Position2, Velocity2),
    reads(Collider, StaticBody)
)]
impl Domain for GravityField {
    fn compute(&mut self, ctx: &mut DomainContext<Self>, delta_time: f64) {
        // Simulation logic stays ordinary Rust.
    }
}
```

The macro must not generate, wrap, or reinterpret business logic. It only generates the associated type declarations and package registration descriptor.

## Events And Reactions

Events are data, so they use derive:

```rust
#[derive(Event, Debug)]
#[event(id = "hit-resolved", label = "Hit Resolved")]
pub struct HitResolved {
    pub target_id: EntityId,
    pub damage: f64,
}
```

Reactions are behavior, so the attribute belongs on the explicit `impl Reaction<E>` block:

```rust
pub struct ApplyDamage;

#[reaction(id = "apply-damage", label = "Apply Damage", event = HitResolved)]
impl Reaction<HitResolved> for ApplyDamage {
    fn react(&mut self, event: &HitResolved, world: &mut World) {
        // Event side effects stay ordinary Rust.
    }
}
```

Observers follow the same pattern as reactions:

```rust
pub struct RecordHitMetrics;

#[observer(id = "record-hit-metrics", event = HitResolved)]
impl Observer<HitResolved> for RecordHitMetrics {
    fn observe(&mut self, event: &HitResolved, world: &World) {
        // Read-only event handling stays ordinary Rust.
    }
}
```

## Target Example Shape

The free-fall body package currently reads like this at the package boundary:

```rust
#[derive(Component, Debug, Clone, Default, PartialEq)]
#[component(id = "position-2", kind = "reality", label = "Position")]
pub struct Position2 {
    #[field(label = "X", default = 0.0, range = -10000.0..=10000.0, unit = "m")]
    pub x: f64,

    #[field(label = "Y", default = 0.0, range = -10000.0..=10000.0, unit = "m")]
    pub y: f64,
}

#[derive(Component, Debug, Clone, Default, PartialEq)]
#[component(id = "velocity-2", kind = "reality", label = "Velocity")]
pub struct Velocity2 {
    #[field(label = "VX", default = 0.0, unit = "m/s")]
    pub vx: f64,

    #[field(label = "VY", default = 0.0, unit = "m/s")]
    pub vy: f64,
}

pub fn package() -> Package {
    duan::catalog::collect_package!()
}
```

The package boundary stays deliberately small:

```rust
pub fn package() -> Package {
    duan::catalog::collect_package!()
}
```

The free-fall gravity package should keep the domain logic explicit:

```rust
pub struct GravityField {
    acceleration: f64,
}

impl GravityField {
    pub fn earth() -> Self {
        Self { acceleration: 9.8 }
    }
}

#[domain(
    id = "field",
    label = "Gravity Field",
    writes(Position2, Velocity2),
    reads(Collider, StaticBody)
)]
impl Domain for GravityField {
    fn compute(&mut self, ctx: &mut DomainContext<Self>, delta_time: f64) {
        let ids: Vec<_> = ctx.each_mut::<Velocity2>().map(|(id, _)| id).collect();

        for id in ids {
            let Some((position, velocity)) = ctx.get_pair_mut::<Position2, Velocity2>(id) else {
                continue;
            };

            velocity.vy -= self.acceleration * delta_time;
            position.x += velocity.vx * delta_time;
            position.y += velocity.vy * delta_time;
        }
    }
}
```

The naval engagement package should show the same split between metadata and behavior:

```rust
#[derive(Component, Debug, Clone, PartialEq)]
#[component(id = "weapon", kind = "reality", label = "Weapon")]
pub struct Weapon {
    #[field(label = "Range", default = 180.0, unit = "m")]
    pub range: f64,

    #[field(label = "Damage", default = 25.0)]
    pub damage: f64,

    #[field(label = "Missile Speed", default = 80.0, unit = "m/s")]
    pub missile_speed: f64,

    #[field(label = "Fire Cooldown", default = 1.5, unit = "s")]
    pub fire_cooldown: f64,

    #[field(label = "Cooldown Remaining", default = 0.0, unit = "s")]
    pub cooldown_remaining: f64,
}

#[derive(Event, Debug)]
#[event(id = "fire-requested", label = "Fire Requested")]
pub struct FireRequested {
    pub shooter_id: EntityId,
    pub target_id: EntityId,
    pub launch_x: f64,
    pub launch_y: f64,
    pub dir_x: f64,
    pub dir_y: f64,
    pub missile_speed: f64,
    pub damage: f64,
}

pub struct CombatDomain;

#[domain(
    id = "combat",
    label = "Combat",
    writes(Weapon),
    reads(Position2, Faction, Health, Radar)
)]
impl Domain for CombatDomain {
    fn compute(&mut self, ctx: &mut DomainContext<Self>, delta_time: f64) {
        // Target selection and event emission stay normal Rust.
    }
}
```

This shape is the acceptance sample for example packages. `ITEM_ID`, `schema()`, and `catalog_item()` are generated by the macros; package authors should not write them by hand for normal items.

## Generated Install Cache

After a package is installed or published, tooling may generate a metadata cache:

```toml
[duan.package]
id = "example-naval-core"
version = "0.1.0"
name = "Naval Combat Platform"

[duan.rust]
crate = "example-naval-core"

[provides.components]
"example-naval-core/health" = "schemas/health.json"
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

If a scenario needs a new algorithm, state transition, event, or command behavior, add it to a Rust package first, expose it through package metadata collection, then reference the item id from the scenario.

## Scenario Project Boundary

Scenario manifests should live inside scenario project directories once package installation and delivery features become real:

```text
examples/scenarios/free-fall/
  scenario.duan
  duan.lock
  assets/
  runs/
  README.md
  .duan/
    packages/
    schemas/
    build/
```

The scenario project is the user-side working unit. It owns package locks, installed package caches, generated schema caches, local runner build artifacts, assets, outputs, and delivery preparation.

Author-side source packages stay under `examples/packages/**` or in normal Cargo package repositories. Installed package caches belong under `.duan/packages/` so they are not confused with source packages.

## Framework Work Required

To make the authoring surface above real, the framework needs these changes:

- Macro crate: keep hardening the existing `duan-macros` crate for `component`, `field`, `event`, `entity`, `domain`, `reaction`, and `observer`.
- Facade crate: keep `duan` as the user-facing crate and keep package collection under `duan::catalog::collect_package!()`.
- Public contracts: add stable metadata traits that macros implement, instead of having macros construct private descriptor internals directly.
- Item collection: keep distributed package item collection for the current Cargo package so users do not maintain a package item list manually.
- Explicit escape hatch: keep `Package::builder`, `ComponentDescriptor`, `EntityDescriptor`, and `RegistrationDescriptor` APIs for generated code and advanced users.
- Diagnostics: make macro errors point at user-owned ids, fields, and impl blocks.
- Runtime boundary: keep `Domain::compute`, `Entity::tick`, `Reaction::react`, and `Observer::observe` as ordinary Rust methods.
- Package collection validation: define behavior for tests, generated runners, Windows builds, release builds, and linker dead-code elimination before relying on it for delivery packaging.
- Scenario flows: close package install, scenario lock, generated cache, runner execution, CLI `run`, editor, and delivery packaging before presenting them as implemented workflows.
