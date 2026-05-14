# DUAN Platform Completion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete the DUAN visual world authoring platform design in staged, verifiable increments.

**Architecture:** Keep `duan-core` as the Rust-first hot runtime and build platform capabilities around it: `duan-package` defines package metadata, schemas, registry, and factories; `duan-scenario` parses and validates manifests; `duan-runner-generator` writes thin static runner crates; `duan-cli` exposes the stable automation surface; `duan-editor` is introduced only after the package/scenario/runner contract is executable. Work must preserve the design rule that scenario manifests assemble Rust capabilities and never become a DSL.

**Tech Stack:** Rust 2021/2024, `duan-core`, Serde, TOML/YAML parsing, Cargo workspaces, Criterion benchmarks, Bun for any future editor frontend.

---

## Current Constraints

- `packages/duan-core` is a git submodule / independent crate and currently has pre-existing `.claude/**` deletions. Do not touch or stage those deletions as part of platform work.
- Outer repository currently has no root `Cargo.toml`; only placeholder package directories exist for platform crates.
- Rust verification after code edits must include `cargo fmt --all` and `cargo clippy --all-targets --all-features -- -D warnings`.
- Cargo commands that build or write `target/` must be run directly in this environment. Do not set `CARGO_TARGET_DIR`.
- Package item ids use lowercase dotted segments and kebab names, for example `duan.kinematics.position-2`.
- Static generated runner is the highest-performance path. Native ABI and external processes remain future work.

## Phase Map

Current baseline status: Phases 0 through 6 have an initial verified implementation in this repository, plus first-pass example metadata, delivery packaging, and a static editor shell. Remaining unchecked items in this plan still represent hardening, deeper example migration, live runner execution, full editor integration, cache invalidation, and future binary package work.

### Phase 0: Workspace And Contract Baseline

Goal: make the outer platform repository buildable without changing `duan-core` semantics.

Owned files:
- `Cargo.toml`
- `packages/duan-package/**`
- `packages/duan-scenario/**`
- `packages/duan-runner-generator/**`
- `packages/duan-cli/**`
- `docs/architecture.md`
- `docs/registry.md`

Tasks:

- [ ] Create root Cargo workspace with members `packages/duan-package`, `packages/duan-scenario`, `packages/duan-runner-generator`, and `packages/duan-cli`.
- [ ] Keep `packages/duan-editor` out of the Rust workspace until it has a concrete frontend stack.
- [ ] Create minimal library/bin crates for the four Rust platform packages.
- [ ] Add internal path dependencies where needed.
- [ ] Add smoke tests proving each crate compiles and exposes a tiny public API.
- [ ] Run `cargo fmt --all`.
- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings`.

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
- [ ] Implement schema data types for fields, primitive values, defaults, ranges, units, and editor metadata.
- [ ] Implement `Package` builder with component/entity/domain/event/reaction/observer registration records.
- [ ] Implement `Registry` with install, duplicate item detection, package lookup, and item lookup.
- [ ] Add tests for valid ids, invalid ids, duplicate items, dependency declarations, and schema metadata round-trip.
- [ ] Document that registry lookup is an assembly-time mechanism and not part of `World::step`.

### Phase 2: Scenario Manifest Parser And Validator

Goal: load scenario manifests that assemble package dependencies, domains, reactions, entities, component values, run options, outputs, and experiments without expressing algorithms.

Owned files:
- `packages/duan-scenario/src/lib.rs`
- `packages/duan-scenario/src/manifest.rs`
- `packages/duan-scenario/src/value.rs`
- `packages/duan-scenario/src/validation.rs`
- `packages/duan-scenario/src/error.rs`
- `packages/duan-scenario/tests/scenario_manifest.rs`
- `examples/free-fall/scenario.yaml`
- `examples/naval-combat/scenario.yaml`

Tasks:

- [ ] Define strongly typed manifest structs for `scenario`, `packages`, `domains`, `reactions`, `entities`, `run`, `outputs`, and `experiments`.
- [ ] Support YAML loading first because existing examples already use `scenario.yaml`.
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

### Phase 5: Runner Generator

Goal: generate a thin Rust runner crate that statically links selected DUAN packages and runs a scenario.

Owned files:
- `packages/duan-runner-generator/src/lib.rs`
- `packages/duan-runner-generator/src/model.rs`
- `packages/duan-runner-generator/src/writer.rs`
- `packages/duan-runner-generator/tests/generate_runner.rs`

Tasks:

- [ ] Generate `Cargo.toml` with registry dependencies, `duan`, `duan-package`, `duan-scenario`, and scenario package crates.
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
- [ ] Implement `duan scenario validate <scenario.yaml>`.
- [ ] Implement `duan runner generate <scenario.yaml>`.
- [ ] Implement `duan runner build <scenario.yaml> --release` as a Cargo invocation wrapper.
- [ ] Implement `duan run <scenario.yaml>`.
- [ ] Implement `duan deliver <scenario.yaml> --target <target>` as a directory packager after runner generation exists.
- [ ] Ensure all errors mention package id, crate name, crate version, registry, scenario path, or generated file path where applicable.

### Phase 7: Example Migration

Goal: prove the package/scenario/runner model with the existing free-fall and naval-combat examples.

Owned files:
- `packages/duan-core/examples/free_fall/**`
- `packages/duan-core/examples/naval_combat/**`
- `examples/free-fall/scenario.yaml`
- `examples/naval-combat/scenario.yaml`
- new example package folders if needed

Tasks:

- [ ] Split reusable kinematics components from example-specific entities.
- [ ] Add package metadata and schema files for free-fall.
- [ ] Add package metadata and schema files for naval-combat.
- [ ] Generate runner fixtures for both examples.
- [ ] Run each example through CLI scenario validation and runner generation.
- [ ] Keep handwritten Rust examples working.

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
- [ ] Include `runner.exe` or target runner binary, scenario files, assets, schemas, license, README, and writable `runs/`.
- [ ] Enforce customer-editable boundaries: scenario parameters yes, Rust logic/package set no.
- [ ] Add build cache keys based on `Cargo.lock`, package set, feature set, target triple, and profile.
- [ ] Add regression tests for cache invalidation.

## First Parallel Wave

Run these in parallel because their write sets are disjoint:

- Worker A: Phase 0 workspace skeleton and minimal crate setup.
- Worker B: Phase 1 `duan-package` id/schema/registry types.
- Worker C: Phase 2 `duan-scenario` manifest model and parser.
- Worker D: Phase 5 `duan-runner-generator` deterministic writer model.

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
