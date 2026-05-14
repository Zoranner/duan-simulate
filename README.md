# DUAN Platform

DUAN Platform is the aggregation repository for the Rust-first simulation package system.

The repository does not make scenario logic into a DSL. Simulation behavior stays in Rust crates. Scenario files only assemble packages, entities, component values, domains, reactions, run options, and outputs.

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
│   ├── free-fall/
│   └── naval-combat/
├── fixtures/
└── scripts/
```

`packages/duan-core` is tracked as an independent package repository. The other package directories are placeholders for the package registry/schema layer, scenario manifest layer, runner generator, CLI, and editor.

## Package Model

DUAN packages are Rust crates published through a private Cargo registry. Each package includes:

- `duan-package.toml` for DUAN package metadata.
- `schemas/` for editor-readable component, entity, domain, event, and reaction schemas.
- `package()` as the Rust registration entry used by generated runners.

Generated runners use Cargo registry dependencies by default. Local path overrides are only for package development.
