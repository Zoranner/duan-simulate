# DUAN Package Naming

This document defines the package naming rules referenced by [DUAN Platform Long-Term Design](platform-design.md).

DUAN uses Cargo packages as the distribution and dependency boundary. Package names need to describe long-lived responsibilities, not current implementation details or temporary scaffolding.

This document defines the target naming system. It is a design target and does not imply every current crate has already been renamed.

## Naming Rules

- Every framework crate uses the `duan-*` prefix.
- Every framework crate has a role suffix. The core runtime is not exempt.
- Names describe product architecture roles, not file formats or one command.
- Example crates use `example-*`, not `examples-*`, because each crate is one package.
- Cargo package names remain kebab-case. Rust crate imports follow Cargo's kebab-to-underscore mapping.
- Scenario item ids still use Cargo package names as package ids.

## Framework Crates

| Target package | Current package | Role |
| --- | --- | --- |
| `duan-runtime` | `duan-runtime` in `packages/duan-runtime` | Hot simulation runtime: world, storage, snapshot, entity, domain, event, reaction, scheduler, and command commit. |
| `duan-macros` | `duan-macros` | Proc macros for component/event derives and item metadata attributes. |
| `duan` | `duan` in `packages/duan` | Thin user-facing Rust facade crate that re-exports stable runtime, macro, and authoring APIs. |
| `duan-author` | not present | Optional author-facing facade that re-exports macros and authoring traits when keeping them out of `duan-runtime` is cleaner. Use only if `duan` should stay smaller than the full authoring surface. |
| `duan-catalog` | `duan-catalog` | Package item metadata, schemas, descriptors, generated cache readers, package item collection, and assembly-time registry. |
| `duan-scenario` | `duan-scenario` | Scenario manifest parser and structural validator. |
| `duan-exec` | `duan-runner` | Scenario execution library. Current `duan-runner` has planned-only preflight plus factory-backed execution for generated runners; use the target name once the execution library boundary is ready to rename. |
| `duan-build` | `duan-build` | Generated runner creation, Cargo build orchestration, build cache keys, and delivery build support. Current `duan-build` owns the build-plan facade and generated runner writer; cache keys and delivery build support are not closed yet. |
| `duan-cli` | `duan-cli` | CLI Cargo package for command line automation. Its binary command is named `duan`. |
| `duan-editor` | `duan-editor` | Visual authoring and inspection application. |

The runtime package name is `duan-runtime`, and the source directory is `packages/duan-runtime`. The user-facing `duan` name belongs to the thin Rust facade crate in `packages/duan`. `duan-cli` is the CLI package name, and `duan` is also the CLI binary command name exposed by that package. The repository root does not need to be the facade crate or a Cargo workspace.

## Layering

The target dependency direction is:

```text
duan-runtime
duan-macros -> duan-catalog
duan -> duan-runtime, duan-macros, duan-catalog
duan-catalog -> duan-runtime, duan-scenario
duan-exec -> duan-runtime, duan-catalog, duan-scenario
duan-build -> duan-catalog, duan-scenario
duan-cli -> duan-catalog, duan-scenario, duan-exec, duan-build
duan-editor -> duan-catalog, duan-scenario, duan-build
```

The current macros emit catalog-facing descriptors and package collection hooks, so `duan-macros` depends on the authoring/catalog contract rather than on runtime behavior alone. `duan-author` is optional. Use it only if re-exporting author-facing macro support directly from `duan` would make the facade too broad or create dependency cycles.

## Current Rename Priorities

Rename in this order when implementation starts:

- Keep `duan-macros` as the current proc-macro crate and harden diagnostics and generated metadata contracts.
- Keep the user-facing `duan` facade as the package author entry point; examples should use explicit `duan::{...}` imports for traits, derives, and attributes, and `duan::catalog::collect_package!()` for package collection.
- Keep runner generation and build orchestration behind `duan-build`; the crate owns the build-plan facade and generated runner writer.
- `duan-runner` to `duan-exec` after it becomes a real execution library.
- Keep `packages/duan-runtime` as the runtime implementation directory.

Do not rename all crates in one mechanical commit unless the workspace is already otherwise quiet. Each rename should include dependency updates, docs updates, and verification.

## Example Crates

Example package names should use this form:

```text
example-<scenario-or-domain>-<capability>
```

Recommended target names:

| Current package | Target package | Role |
| --- | --- | --- |
| `examples-free-fall-body` | `example-freefall-physics` | Free-fall physical state components such as position, velocity, collider, and static body marker. |
| `examples-free-fall-gravity` | `example-freefall-gravity` | Free-fall gravity domain. |
| `examples-free-fall-scene-objects` | `example-freefall-objects` | Free-fall entity templates. |
| `examples-naval-combat-platform` | `example-naval-core` | Shared naval state components such as faction, health, and radar. |
| `examples-naval-combat-maneuver` | `example-naval-motion` | Naval movement components and motion/collision domains. |
| `examples-naval-combat-engagement` | `example-naval-combat` | Weapons, combat events, combat domain, and damage reactions. |
| `examples-naval-combat-fleet-objects` | `example-naval-fleet` | Naval fleet entity templates. |

The package id is part of every scenario item id, so example package renames must be coordinated with `.duan` scenario files and tests.

## Non-Goals

- Do not rename runtime concepts such as `Domain`, `Entity`, `Component`, or `Reaction` as part of crate naming.
- Do not introduce a package service crate by default. Distribution remains Cargo registry based.
- Do not make editor or CLI naming drive core crate names.
