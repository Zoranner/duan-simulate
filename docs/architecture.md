# DUAN Platform Architecture

The long-term platform design is consolidated in [DUAN Platform Long-Term Design](platform-design.md). This architecture note summarizes the main runtime and platform boundaries.

The repository root is a product collection and documentation entry. It is not required to be a Cargo package or root Cargo workspace. Framework packages should remain independently understandable, with a root workspace introduced only when shared build and release workflows require it.

## Runtime Boundary

The core runtime package is `duan-runtime`. The current implementation still lives in `packages/duan-core`; that directory has not been renamed yet. User packages import the `duan` facade crate, which re-exports runtime APIs, authoring macros, and catalog APIs while keeping `duan-runtime` as the hot runtime package.

`duan-runtime` remains the hot runtime path. It keeps `World::step`, `Belief / Intent / Reality`, `Entity::tick`, `Domain::compute`, `Reaction::react`, storage, snapshots, scheduling, events, and command commit as Rust-first runtime behavior.

The platform layer adds package-facing APIs around the core:

- Package registry.
- Package item metadata collection from Rust annotations.
- Component schema and decode.
- Entity, domain, reaction, and observer factories.
- Scenario manifest parsing and validation.
- Generated runner creation.

Current status: `packages/duan` exists as the user-facing facade. Examples import macros through `duan::{Component, Event, domain, entity, reaction, ...}` and collect package metadata through `duan::catalog::collect_package!()`. `packages/duan-macros` and `packages/duan-catalog` remain internal framework packages used by the facade and generated code.

## Distribution

Package distribution is based on a private Cargo registry. DUAN does not need a separate default package service.

The target editor flow downloads `.crate` packages and reads generated install caches derived from Rust package registration and schema APIs. Source packages do not hand-author `duan.toml` or `schemas/` as facts; those files are cache artifacts for inspection, validation, and delivery packaging. This install/cache/lock/editor flow is not closed in the current implementation.

Generated runners statically link selected Cargo packages. Current runner execution is still planned-only: `duan-runner` builds an assembly plan and returns `RunStatus::PlannedOnly` rather than executing an assembled `World`. Runtime behavior should remain compiled Rust when the execution library is completed.

## Authoring Surface

The intended package authoring surface is macro-assisted Rust, not a scenario or package DSL. Data items such as components and events use derive macros. Behavior items such as entities, domains, reactions, and observers keep explicit Rust trait impls and use attributes only for item ids, read/write sets, event bindings, and package metadata.

Package authors should not maintain a hand-written item list for every package. Annotated items are collected into the current Cargo package metadata, while explicit builder APIs remain available for advanced or generated code.

## Package Naming

Framework crate names should follow the role-based naming system in [DUAN Package Naming](package-naming.md). In particular, the core runtime uses the role suffix `duan-runtime`, while the short `duan` name belongs to the user-facing facade.
