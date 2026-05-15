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
| `duan-runtime` | `duan` in `packages/duan-core` | Hot simulation runtime: world, storage, snapshot, entity, domain, event, reaction, scheduler, and command commit. |
| `duan-macros` | not present | Proc macros for component/event derives and item metadata attributes. |
| `duan` | not present | Thin user-facing facade crate. It can live under `packages/duan/` and re-export stable runtime, macro, and authoring APIs. |
| `duan-author` | not present | Optional author-facing facade that re-exports macros and authoring traits when keeping them out of `duan-runtime` is cleaner. Use only if `duan` should stay smaller than the full authoring surface. |
| `duan-catalog` | `duan-package` | Package item metadata, schemas, descriptors, generated cache readers, package item collection, and assembly-time registry. |
| `duan-scenario` | `duan-scenario` | Scenario manifest parser and structural validator. |
| `duan-exec` | `duan-runner` | Scenario execution library. This name should only be used once the crate owns real execution, not just planned-only audit. |
| `duan-build` | `duan-runner-generator` and current empty `packages/duan-build` | Generated runner creation, Cargo build orchestration, build cache keys, and delivery build support. |
| `duan-cli` | `duan-cli` | Command line automation surface. Its binary can still be named `duan`. |
| `duan-editor` | `duan-editor` | Visual authoring and inspection application. |

`duan-runtime` replaces the current short implementation package name `duan` for consistency. The user-facing `duan` name should belong to a thin facade crate, not the runtime implementation. The repository root does not need to be that crate or a Cargo workspace.

## Layering

The target dependency direction is:

```text
duan-runtime
duan-macros
duan -> duan-runtime, duan-macros
duan-catalog -> duan-runtime, duan-scenario
duan-exec -> duan-runtime, duan-catalog, duan-scenario
duan-build -> duan-catalog, duan-scenario
duan-cli -> duan-catalog, duan-scenario, duan-exec, duan-build
duan-editor -> duan-catalog, duan-scenario, duan-build
```

`duan-author` is optional. Use it only if re-exporting author-facing macro support directly from `duan-runtime` would pollute the hot runtime crate or create dependency cycles.

## Current Rename Priorities

Rename in this order when implementation starts:

- `duan-package` to `duan-catalog`: this is the clearest mismatch because the crate owns more than package file handling.
- `duan-runner-generator` and the empty `packages/duan-build` concept into one `duan-build` crate.
- `duan-runner` to `duan-exec` after it becomes a real execution library.
- `duan` / `packages/duan-core` to `duan-runtime` after the outer platform dependency graph is stable enough to absorb the submodule rename.

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
