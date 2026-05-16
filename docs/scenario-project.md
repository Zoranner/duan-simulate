# DUAN Scenario Projects

A scenario project is the user-side working directory for one simulation setup. It contains the scenario manifest, package lock state, assets, outputs, generated caches, and local runner build artifacts.

It is separate from package source. Source packages define capabilities; scenario projects assemble those capabilities.

## Target Shape

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

The same shape applies to product projects outside this repository.

## Files And Directories

`scenario.duan` is the scenario manifest. It lists package dependencies, domains, reactions, entities, component values, run options, outputs, and experiments.

`duan.lock` records resolved packages, versions, registry or source, checksums when available, selected features, and generated cache versions. It belongs to the scenario project because package resolution is project-specific.

`assets/` contains user-managed scenario assets.

`runs/` contains writable run outputs such as frames, events, metrics, logs, and experiment results.

`.duan/packages/` contains installed package cache artifacts. It is tool-managed and must not be confused with source packages.

`.duan/schemas/` contains generated schema and metadata caches for editor, validator, and delivery tooling.

`.duan/build/` contains generated runner projects and build cache material.

## Source Package Boundary

Author-side source packages live under `examples/packages/**` in this repository or in normal Cargo package repositories.

Scenario projects should not vendor source packages by default. Local source packages are referenced from `packages[].path` in `scenario.duan`, and that path is relative to the scenario file directory. A package entry with `path` does not need `version`; registry packages still use `version`. Do not place source crates inside the scenario project just to make them available.

## Manifest Boundary

Scenario manifests assemble existing package items. They may describe:

- package dependencies and versions;
- domains, reactions, and observers to install;
- entity instances to spawn;
- component values to attach or override;
- domain and run parameters;
- output paths and experiment batches.

They do not describe business algorithms. New behavior belongs in Rust packages.

## Delivery Boundary

Delivery packaging can copy a scenario project into a runtime bundle together with a prebuilt runner, read-only schemas, license material, and a writable `runs/` directory.

Customers can edit supported scenario values. They cannot add package items that were not included in the runner unless they are using an SDK or development workflow.
