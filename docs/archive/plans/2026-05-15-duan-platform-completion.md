# DUAN Platform Completion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete the DUAN visual world authoring platform design in staged, verifiable increments.

**Architecture:** Keep the core runtime as the Rust-first hot path and build platform capabilities around it. The authoritative long-term design is `docs/platform-design.md`. Target crate names are role-based: `duan-runtime` for runtime, `duan-catalog` for package metadata and registry, `duan-scenario` for manifests, `duan-build` for runner generation/build support, `duan-exec` for execution, `duan-cli` for automation, and `duan-editor` for visual authoring. Current crate names may lag behind this target while migration is staged. Work must preserve the design rule that scenario manifests assemble Rust capabilities and never become a DSL.

**Tech Stack:** Rust 2021/2024, `duan-core`, Serde, TOML/YAML parsing, independent Cargo packages, Criterion benchmarks, Bun for any future editor frontend.

---

## Current Constraints

- `packages/duan-core` is a git submodule / independent crate and currently has pre-existing `.claude/**` deletions. Do not touch or stage those deletions as part of platform work.
- Outer repository currently has no root `Cargo.toml`, and that is acceptable. The root is a product/documentation collection, not a required Cargo workspace.
- Rust verification after code edits must include `cargo fmt --all` and `cargo clippy --all-targets --all-features -- -D warnings`.
- Cargo commands that build or write `target/` must be run directly in this environment. Do not set `CARGO_TARGET_DIR`.
- Package ids use Cargo package names, and item ids use `<package-id>/<local-name>`, for example `example-freefall-physics/position-2`.
- `duan.toml` and `schemas/**` are generated install/cache artifacts. Rust item annotations and type-local schema/display metadata are the source of truth.
- Package authors should not maintain hand-written package item lists. The intended authoring surface is item-level metadata collection with explicit builder APIs kept as an escape hatch.
- Framework crate names should follow `docs/package-naming.md`; every framework crate, including the core runtime, uses a role suffix.
- Scenario manifests should converge on scenario project directories such as `examples/scenarios/free-fall/scenario.duan`, with `duan.lock`, `assets/`, `runs/`, and generated `.duan/**` caches owned by that project.
- Static generated runner is the highest-performance path. Native ABI and external processes remain future work.

## Phase Map

Current baseline status: Phases 0 through 6 have an initial verified implementation in this repository, plus first-pass example metadata, delivery packaging, and a static editor shell. Remaining unchecked items in this plan still represent hardening, deeper example migration, live runner execution, full editor integration, cache invalidation, and future binary package work.

### Phase 0: Repository And Contract Baseline

Goal: make the outer platform repository boundaries explicit without changing `duan-core` semantics.

Owned files:
- `packages/duan-package/**`
- `packages/duan-scenario/**`
- `packages/duan-build/**`
- `packages/duan-cli/**`
- optional package-local `Cargo.toml` files
- `docs/architecture.md`
- `docs/registry.md`

Tasks:

- [ ] Keep the repository root free of Cargo workspace assumptions until shared build and release workflows require one.
- [ ] Keep `packages/duan-editor` independent until it has a concrete frontend stack.
- [ ] Ensure each Rust framework package has its own clear Cargo boundary.
- [ ] Add internal path dependencies where needed.
- [ ] Add smoke tests proving each crate compiles and exposes a tiny public API.
- [ ] Run `cargo fmt --all`.
- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings`.

### Phase 0.5: Crate Naming Alignment

Goal: migrate framework and example crate names to the role-based naming system without changing runtime semantics.

Owned files:
- `Cargo.toml`
- package `Cargo.toml` files
- `Cargo.lock`
- `README.md`
- `docs/package-naming.md`
- `docs/package-authoring.md`
- `docs/architecture.md`
- tests and scenarios that reference package ids

Tasks:

- [ ] Rename `duan-package` to `duan-catalog`.
- [x] Merge the generated runner writer into `duan-build`.
- [ ] Rename `duan-runner` to `duan-exec` when it owns real execution semantics.
- [ ] Rename the core runtime package to `duan-runtime`; add a small `packages/duan` facade for the user-facing `duan` crate when the public surface is stable.
- [ ] Rename example packages from `examples-*` to `example-*` according to `docs/package-naming.md`.
- [ ] Update scenario package ids and item references after each example package rename.
- [ ] Keep each rename as a separately verifiable commit unless the workspace is otherwise quiet.

### Phase 0.6: Scenario Project Layout

Goal: move examples from flat scenario files into scenario project directories before package installation, lock files, and delivery packaging depend on paths.

Owned files:
- `README.md`
- `docs/platform-design.md`
- `docs/package-authoring.md`
- `packages/duan-scenario/tests/scenario_manifest.rs`
- `packages/duan-cli/tests/cli_smoke.rs`
- `packages/duan-build/tests/generated_runner.rs`
- `examples/scenarios/**`

Tasks:

- [ ] Move `examples/scenarios/free-fall.duan` to `examples/scenarios/free-fall/scenario.duan`.
- [ ] Move `examples/scenarios/naval-combat.duan` to `examples/scenarios/naval-combat/scenario.duan`.
- [ ] Add scenario project README files that define `duan.lock`, `assets/`, `runs/`, and `.duan/**` ownership.
- [ ] Update scenario parser tests and CLI smoke tests to load the new paths.
- [ ] Update runner generator fixtures to use project-local `scenario.duan`.
- [ ] Keep installed package caches under `.duan/packages/`, not under project-level `packages/`.

### Phase 1: Package Registry And Schema Foundation

Goal: represent DUAN packages, package item ids, component schemas, entity/domain/reaction descriptors, and runtime registration metadata.

Owned files:
- `packages/duan-package/src/lib.rs`
- `packages/duan-package/src/id.rs`
- `packages/duan-package/src/schema.rs`
- `packages/duan-package/src/package.rs`
- `packages/duan-package/src/registry.rs`
- `packages/duan-package/src/error.rs`
- `packages/duan-package/tests/package_registry.rs`

Tasks:

- [ ] Implement `PackageId`, `ItemId`, and `VersionReqText` as validated string newtypes.
- [ ] Implement schema data types for fields, primitive values, defaults, ranges, units, and display metadata.
- [ ] Implement `Package` builder with component/entity/domain/event/reaction/observer registration records.
- [ ] Implement `Registry` with install, duplicate item detection, package lookup, and item lookup.
- [ ] Add tests for valid ids, invalid ids, duplicate items, dependency declarations, and schema metadata round-trip.
- [ ] Document that registry lookup is an assembly-time mechanism and not part of `World::step`.
- [ ] Define public metadata traits for component, entity, domain, event, reaction, and observer items so future macros do not depend on private descriptor internals.

### Phase 2: Scenario Manifest Parser And Validator

Goal: load scenario manifests that assemble package dependencies, domains, reactions, entities, component values, run options, outputs, and experiments without expressing algorithms.

Owned files:
- `packages/duan-scenario/src/lib.rs`
- `packages/duan-scenario/src/manifest.rs`
- `packages/duan-scenario/src/value.rs`
- `packages/duan-scenario/src/validation.rs`
- `packages/duan-scenario/src/error.rs`
- `packages/duan-scenario/tests/scenario_manifest.rs`
- `examples/scenarios/free-fall/scenario.duan`
- `examples/scenarios/naval-combat/scenario.duan`

Tasks:

- [ ] Define strongly typed manifest structs for `scenario`, `packages`, `domains`, `reactions`, `entities`, `run`, `outputs`, and `experiments`.
- [ ] Support YAML loading first because existing examples already use `scenario.duan`.
- [ ] Store component initial values as structured values, not strings.
- [ ] Validate duplicate entity ids, missing package dependencies, invalid item ids, unknown component references, and invalid run/output paths.
- [ ] Add tests for the free-fall and naval-combat example manifests.
- [ ] Keep validation structural; do not attempt to interpret `Entity::tick`, `Domain::compute`, or `Reaction::react`.

### Phase 3: Fallible Core Assembly Boundary

Goal: add the minimal `duan-core` API needed by generated runners and editors to receive structured build errors while preserving existing `World::builder().build()` behavior.

Owned files:
- `packages/duan-core/src/world/builder.rs`
- `packages/duan-core/src/world/mod.rs`
- `packages/duan-core/src/lib.rs`
- `packages/duan-core/tests/execution_contract.rs`

Tasks:

- [ ] Add `WorldBuildError` with scheduler/build diagnostics.
- [ ] Add `WorldBuilder::try_build() -> Result<World, WorldBuildError>`.
- [ ] Keep `WorldBuilder::build()` as panic-on-error convenience for handwritten Rust.
- [ ] Add tests that duplicate domain writes or dependency cycles return structured errors through `try_build`.
- [ ] Confirm existing execution contract tests still pass.

### Phase 4: Assembly Factories

Goal: let package registries create domains, reactions, observers, and entities from scenario records.

Owned files:
- `packages/duan-package/src/factory.rs`
- `packages/duan-package/src/assembly.rs`
- `packages/duan-package/tests/assembly.rs`
- `packages/duan-scenario/src/manifest.rs`
- `packages/duan-core` only if a small public hook is required

Tasks:

- [ ] Define `ScenarioComponent` decode trait without adding cost to hot `Component`.
- [ ] Define `EntityFactory`, `DomainFactory`, `ReactionFactory`, and `ObserverFactory`.
- [ ] Implement assembly order: install domains, install reactions/observers, `try_build`, spawn entities, apply default bundle, then scenario overrides.
- [ ] Make unsupported component overrides explicit: allow, warn, or error based on entity schema.
- [ ] Add tests with a tiny fake package and fake scenario.

### Phase 5: Runner Generation

Goal: generate a thin Rust runner crate that statically links selected DUAN packages and runs a scenario.

Owned files:
- `packages/duan-build/src/lib.rs`
- `packages/duan-build/src/runner_project.rs`
- `packages/duan-build/tests/generated_runner.rs`

Tasks:

- [ ] Generate `Cargo.toml` with registry dependencies, `duan`, `duan-runner`, `duan-scenario`, and scenario package crates.
- [ ] Generate `.cargo/config.toml` with deployment-specific private registry URL supplied by configuration.
- [ ] Generate `src/main.rs` that loads scenario, installs package functions, and calls the runner path.
- [ ] Use deterministic file output so generated runners are stable under repeated generation.
- [ ] Test generation in a temp directory and compare exact generated files.

### Phase 6: CLI

Goal: expose the stable automation surface outside the editor.

Owned files:
- `packages/duan-cli/src/main.rs`
- `packages/duan-cli/src/commands.rs`
- `packages/duan-cli/src/error.rs`
- `packages/duan-cli/tests/cli_smoke.rs`

Tasks:

- [ ] Implement `duan package inspect <path>`.
- [ ] Implement `duan scenario validate <scenario.duan>`.
- [ ] Implement `duan runner generate <scenario.duan>`.
- [ ] Implement `duan runner build <scenario.duan> --release` as a Cargo invocation wrapper.
- [ ] Implement `duan run <scenario.duan>`.
- [ ] Implement `duan deliver <scenario.duan> --target <target>` as a directory packager after runner generation exists.
- [ ] Ensure all errors mention package id, crate name, crate version, registry, scenario path, or generated file path where applicable.

### Phase 7: Example Migration

Goal: prove the package/scenario/runner model with the existing free-fall and naval-combat examples.

Owned files:
- `packages/duan-core/examples/free_fall/**`
- `packages/duan-core/examples/naval_combat/**`
- `examples/scenarios/free-fall/scenario.duan`
- `examples/scenarios/naval-combat/scenario.duan`
- new example package folders if needed

Tasks:

- [ ] Split reusable kinematics components from example-specific entities.
- [ ] Add Rust package registration and type-local schema/display metadata for free-fall.
- [ ] Add Rust package registration and type-local schema/display metadata for naval-combat.
- [ ] Generate runner fixtures for both examples.
- [ ] Run each example through CLI scenario validation and runner generation.
- [ ] Keep handwritten Rust examples working.

### Phase 7.5: Macro Authoring Surface

Goal: replace package author boilerplate with stable Rust macros while keeping business logic in normal Rust.

Owned files:
- `packages/duan-package/**`
- future proc-macro crate
- `examples/packages/**`
- `docs/package-authoring.md`

Tasks:

- [ ] Add derive macros for data items: `Component` and `Event`.
- [ ] Add attributes for behavior impls: `entity`, `domain`, `reaction`, and `observer`.
- [ ] Add field-level component schema attributes for label, default, range, unit, and control metadata.
- [ ] Add distributed package item collection so users do not hand-maintain package item lists.
- [ ] Keep `id` as the package-local stable item id segment and `label` as display text.
- [ ] Ensure generated item ids never include item kind segments such as `component`, `entity`, or `domain`.
- [ ] Keep explicit builder and descriptor APIs for generated code and advanced users.
- [ ] Verify macro diagnostics on duplicate ids, invalid ids, unsupported field metadata, and package collection failures.

### Phase 8: Editor Foundation

Goal: create the visual authoring foundation only after package schemas and scenarios are real.

Owned files:
- `packages/duan-editor/**`

Tasks:

- [ ] Choose and document frontend stack before implementation. If Node is used, use Bun, never npm.
- [ ] Implement package list and schema reader.
- [ ] Implement scenario entity list and component parameter forms.
- [ ] Implement domain/reaction installation view.
- [ ] Implement runner generate/build/run trigger surface.
- [ ] Implement run output views for events, logs, snapshots, and metrics.
- [ ] Keep UI focused on authoring and inspection; do not expose visual algorithm editing.

### Phase 9: Delivery And Hardening

Goal: package prebuilt runners, scenarios, assets, schemas, license files, and documentation for customer delivery.

Owned files:
- `packages/duan-cli/**`
- `scripts/**`
- `docs/**`

Tasks:

- [ ] Implement `duan deliver`.
- [ ] Include `runner.exe` or target runner binary, scenario files, assets, generated schema cache, license, README, and writable `runs/`.
- [ ] Enforce customer-editable boundaries: scenario parameters yes, Rust logic/package set no.
- [ ] Add build cache keys based on `Cargo.lock`, package set, feature set, target triple, and profile.
- [ ] Add regression tests for cache invalidation.

## First Parallel Wave

Run these in parallel because their write sets are disjoint:

- Worker A: Phase 0 repository boundary and package-local crate setup.
- Worker B: Phase 1 `duan-package` id/schema/registry types.
- Worker C: Phase 2 `duan-scenario` manifest model and parser.
- Worker D: Phase 5 `duan-build` deterministic runner writer model.

Do not dispatch Phase 3 until the main session reviews `duan-core` status and decides how to handle its existing `.claude/**` deletions.

## Verification Gates

After each wave:

- `git status --short`
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`

When `duan-core` is modified:

- Run the same checks inside `packages/duan-core`.
- Run `cargo test --all-targets --all-features`.
- Preserve existing benchmark files and rerun `cargo bench --bench world_step_bench` only when runtime hot-path code changes.

## Commit Strategy

- Commit by logical phase after verification.
- Do not include the pre-existing `packages/duan-core/.claude/**` deletions in platform commits.
- Use repository history style. Current outer repository history is English, so commit subjects should be English.
