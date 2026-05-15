# DUAN Platform Long-Term Design

This document is the long-term design source for the DUAN platform layer. It consolidates the current package, scenario, macro authoring, crate naming, runner, editor, and delivery decisions.

Older design notes under `packages/duan-core/docs/**` remain useful historical context, but they may still describe dotted item ids, hand-written package manifests, and old crate names. When those notes conflict with this document and the outer `docs/**` files, this document is authoritative for platform design.

The repository root is a product and documentation collection. It does not need to be a Cargo package or a root Cargo workspace by default. Individual framework packages own their own Cargo boundaries; a root workspace can be introduced later only when multi-crate build, test, or release workflows justify it.

## Position

DUAN is a Rust-first simulation package platform. It is not a simulation DSL, a visual programming language, or a dynamic field-table runtime.

The stable split is:

- Rust packages define simulation capability, Rust types, schemas, factories, metadata, and real behavior.
- Scenario manifests assemble existing package items, entity instances, component values, domains, reactions, run options, and outputs.
- The editor installs packages, reads schemas, edits scenarios, triggers runner generation/build/run, and inspects results.
- Generated runners statically link selected packages and keep `World::step` on the compiled Rust hot path.
- Customer delivery defaults to a prebuilt runner plus scenario project files, assets, generated schemas, license material, and writable run outputs.

## Runtime Boundary

The hot runtime crate target name is `duan-runtime`. The current implementation still lives at `packages/duan-core` with Cargo package name `duan`; this is a migration state, not the target naming model.

The runtime keeps these concepts as plain Rust:

- `World`, storage, snapshots, scheduler, events, and command commit.
- `Belief`, `Intent`, and `Reality` component semantics.
- `Entity::tick`.
- `Domain::compute`.
- `Reaction::react`.
- `Observer::observe`.

Platform code may add package-facing assembly hooks around the runtime, but it must not turn `duan-runtime` into a string-driven ECS or make `World::step` depend on package item id lookup.

## Package Identity

A DUAN package is an ordinary Cargo package. The Cargo `package.name` is the DUAN package id.

```toml
[package]
name = "example-naval-core"
version = "0.1.0"
edition = "2021"
```

Package item ids use:

```text
<package-id>/<id>
```

Example:

```text
example-naval-core/health
```

The package id is the Cargo package name. The `id` segment is the stable item id inside that package. It is not display text; display text belongs in `label`.

Item ids do not include mechanical type path segments such as `component`, `entity`, `domain`, `event`, or `reaction`. Item kind comes from the Rust item annotation and generated descriptor.

## Package Source Of Truth

Rust source is the authoring source of truth.

Package authors define components, entities, domains, events, reactions, observers, schema metadata, display metadata, and factories in `src/**`. Generated files such as `duan.toml`, `schemas/**`, and editor metadata caches are install, publish, validation, or delivery artifacts. They are not the hand-authored facts for normal packages.

This matters because duplicate sources of truth make package authoring fragile. A user who adds a component in Rust should not also remember to add the same item to a separate manifest before the framework can see it.

## Authoring Surface

The long-term authoring surface is macro-assisted Rust:

- Data items use derive macros: `#[derive(Component)]` and `#[derive(Event)]`.
- Behavior items keep explicit trait implementations and use attributes on the impl block: `#[entity(...)]`, `#[domain(...)]`, `#[reaction(...)]`, and `#[observer(...)]`.
- Field schema metadata stays near component fields with `#[field(...)]`.
- Business logic remains normal Rust in `tick`, `compute`, `react`, and `observe`.
- Package collection is automatic through the current Cargo package.

The intended package entry point is:

```rust
pub fn package() -> Package {
    duan::collect_package()
}
```

Explicit builder and descriptor APIs remain as an escape hatch for generated code, tests, and advanced users. They are not the preferred user-facing authoring model.

## Macro Rules

Macros are for metadata, schema, registration, and boilerplate reduction. They must not hide or reinterpret business behavior.

The rules are:

- Use `Component` and `Event`, not `DuanComponent` or `DuanEvent`.
- Use `id` for the package-local stable item id segment.
- Use `label` for display text.
- Do not use `local_name`; it is less clear than `id`.
- Keep component `kind` explicit because it maps to runtime semantics: `belief`, `intent`, or `reality`.
- Keep domain read/write/dependency declarations visible on the domain impl.
- Generate diagnostics that point at user-owned ids, fields, and impl blocks.
- Validate package collection behavior before relying on it for release builds, generated runners, Windows builds, and linker dead-code elimination.

## Scenario Manifests

Scenario manifests are structured DUAN manifests. The current parser may use YAML internally, but the scenario file is not "YAML as product design" and not a DUAN DSL.

A scenario may describe:

- package dependencies and versions;
- domains, reactions, and observers to install;
- entity instances to spawn;
- component values to attach or override;
- run options;
- output paths;
- experiment batches.

A scenario must not describe:

- entity decision algorithms;
- domain computation algorithms;
- reaction side-effect algorithms;
- event emission logic;
- search, planning, physics solving, or external model logic.

If a scenario needs new behavior, the behavior belongs in a Rust package first. The scenario then references the exposed package item id.

## Scenario Projects

A scenario file should live inside a scenario project directory, not as a long-term flat file. The project directory is the unit for package installation, locks, assets, runs, local build cache, and delivery preparation.

Target example shape:

```text
examples/
  packages/
    free-fall/
    naval-combat/

  scenarios/
    free-fall/
      scenario.duan
      duan.lock
      assets/
      runs/
      README.md
      .duan/
        packages/
        schemas/
        build/

    naval-combat/
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

`examples/packages/**` contains author-side source packages. `examples/scenarios/<name>/` contains user-side scenario projects. See [DUAN Scenario Projects](scenario-project.md) for the detailed directory contract.

Installed package caches should live under `.duan/packages/`, not under a top-level `packages/` directory inside the scenario project. That avoids confusing installed artifacts with source packages.

## Generated Runner

Generated runners are thin Rust crates created from a scenario project and its package lock state.

Runner generation owns:

- generated `Cargo.toml`;
- generated `.cargo/config.toml`;
- generated `src/main.rs`;
- dependency selection;
- build cache keys;
- registry configuration;
- development path overrides.

The runner uses registry dependencies by default. Local path overrides are a development mechanism and should not become the normal scenario distribution model.

Only package set changes require runner regeneration and rebuild. Changes to entity counts, component values, domain parameters, run options, and output paths should reuse the existing runner when the selected package set is unchanged.

## Package Installation

DUAN package distribution is based on Cargo registries. DUAN does not need a separate default package download service.

The install flow is:

```text
resolve Cargo package
download or locate crate
collect package metadata
generate or read install cache
update scenario project lock state
make schemas available to editor and validator
regenerate runner when package set changes
```

The lock file belongs to the scenario project. It should record enough information to reproduce package resolution and runner generation: package names, versions, registry/source, checksums when available, selected features, and generated cache versions.

## Editor Boundary

The editor is a package and scenario authoring tool, not an algorithm editor.

It should support:

- package installation and inspection;
- component forms from schema metadata;
- entity instance editing;
- domain/reaction/observer assembly;
- dependency and read/write visibility;
- runner generation/build/run actions;
- event, snapshot, log, metric, and experiment output views.

It should not support:

- editing `Entity::tick`, `Domain::compute`, `Reaction::react`, or `Observer::observe`;
- using visual nodes as the source of business algorithms;
- writing partially edited editor state directly into runtime `World`;
- replacing Rust package behavior with scenario expressions.

## Delivery Model

Development and customer delivery are different modes.

Development mode can use source packages, path overrides, generated runners, and Cargo builds.

Runtime delivery defaults to:

```text
delivery/
  runner.exe
  scenario/
    scenario.duan
    duan.lock
    assets/
  schemas/
  runs/
  license/
  README.md
```

Customers may edit scenario parameters, entity instances, component values, domain parameters, run options, and output paths within supported bounds.

Customers do not need Rust source code for normal runtime delivery. They also cannot add new package items unless the runner already contains those packages or they are using an SDK/development delivery mode.

## Crate Naming

Framework crates use the `duan-*` prefix and role suffixes. The core runtime is not exempt.

The target names are:

- `duan-runtime`: hot runtime.
- `duan-macros`: proc macros.
- `duan-author`: optional author-facing facade.
- `duan-catalog`: package item metadata, schemas, collection, and registry.
- `duan-scenario`: scenario manifest parser and validator.
- `duan-build`: generated runner creation, Cargo build orchestration, cache keys, and delivery build support.
- `duan-exec`: scenario execution library.
- `duan-cli`: command line automation.
- `duan-editor`: visual authoring and inspection application.

The user-facing `duan` Cargo package should be a thin facade crate when it is introduced. It can live under `packages/duan/` and re-export stable runtime, macro, and authoring APIs. The repository root does not need to be that package.

Example packages use `example-*`, not `examples-*`.

## Migration Priorities

The implementation should migrate in coherent, verifiable units:

- Keep current runtime behavior stable while platform APIs mature around it.
- Keep the repository root free of Cargo workspace assumptions until a shared build/release workflow is actually needed.
- Consolidate runner generation and build orchestration under `duan-build`.
- Introduce `duan-exec` when execution has a real library boundary.
- Rename the runtime package to `duan-runtime` once the outer platform dependency graph is ready.
- Move flat example scenarios into scenario project directories before package installation and lock files become central.
- Add macros and automatic package collection only after public metadata traits are stable.

## Design Rules

- Rust package source is the authority for simulation capability.
- Cargo package name is the DUAN package id.
- Item id shape is `<package-id>/<id>`.
- Item kind is metadata, not an id path segment.
- Scenario manifests assemble capabilities and never become a DSL.
- Editors edit schema-backed configuration, not algorithms.
- Generated install caches are artifacts, not package source truth.
- Static Rust generated runner is the primary high-performance path.
- `World::step` stays free of string lookup and dynamic package dispatch.
- Framework crate names are role-based and consistently suffixed.
- Scenario projects are the working unit for install, lock, assets, runs, build cache, and delivery.
- The repository root is the product entry, not necessarily a Cargo package or workspace.
