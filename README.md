# DUAN Platform

DUAN Platform is the aggregation repository for the Rust-first simulation package system.

The repository does not make scenario logic into a DSL. Simulation behavior stays in Rust crates. Scenario files only assemble packages, entities, component values, domains, reactions, run options, and outputs.

See [DUAN Package Authoring](docs/package-authoring.md) for the current Rust package and generated metadata cache conventions.

## Layout

```text
duan-platform/
├── docs/
├── packages/
│   ├── duan-core/
│   ├── duan-package/
│   ├── duan-scenario/
│   ├── duan-runner-generator/
│   ├── duan-cli/
│   └── duan-editor/
├── examples/
│   ├── scenarios/
│   │   ├── free-fall.duan
│   │   └── naval-combat.duan
│   └── packages/
│       ├── free-fall/
│       └── naval-combat/
├── fixtures/
└── scripts/
```

`packages/duan-core` is tracked as an independent package repository. The other package directories provide the package registry/schema layer, scenario manifest layer, runner generator, and CLI. Example simulation capabilities live under `examples/packages/` as ordinary Cargo packages.

## Package Model

DUAN packages are Rust Cargo packages published through a private Cargo registry. The Cargo `package.name` is the DUAN package id, for example `examples-free-fall-body`. Package item ids use `<package-id>/<local-name>`, for example `examples-free-fall-body/position-2`.

Package source includes:

- `Cargo.toml` for Cargo identity, version, and dependencies.
- `src/**` for components, entities, domains, events, reactions, schemas, display metadata, and behavior.
- `package()` as the Rust registration entry used by generated runners.

`duan.toml` and `schemas/` are generated installation/cache artifacts, not hand-written source facts for example packages. Generated runners use Cargo registry dependencies by default. Local path overrides are only for package development.
