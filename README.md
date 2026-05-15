# DUAN

DUAN is a Rust-first simulation platform for complex rule-driven worlds.

It is built for systems where many objects evolve over time, rules need clear authority, and repeated runs must stay explainable and reproducible. Entities express intent, domains decide facts, and events record what happened.

DUAN does not turn simulation logic into a DSL. Business behavior stays in Rust packages. Scenario files assemble existing packages, entities, component values, domains, reactions, run options, and outputs.

## Use DUAN

Runtime users write normal Rust:

```rust
use duan::prelude::*;

#[derive(Clone, Default)]
struct Position {
    y: f64,
}
duan::reality!(Position);

struct Ball;

impl Entity for Ball {
    fn bundle() -> impl ComponentBundle + Send + 'static {
        (Position { y: 10.0 },)
    }
}
```

Package authors expose reusable components, domains, entities, events, and reactions as Cargo packages. Scenario authors assemble those capabilities into `scenario.duan` projects.

## Read More

Start with:

- [Platform design](docs/platform-design.md)
- [Platform philosophy](docs/platform-philosophy.md)
- [Package authoring](docs/package-authoring.md)
- [Scenario projects](docs/scenario-project.md)
- [Package naming](docs/package-naming.md)

## Layout

```text
duan/
├── docs/
├── packages/
│   ├── duan-core/
│   ├── duan-macros/
│   ├── duan-catalog/
│   ├── duan-scenario/
│   ├── duan-runner/
│   ├── duan-runner-generator/
│   ├── duan-build/
│   ├── duan-cli/
│   └── duan-editor/
├── examples/
│   ├── scenarios/
│   │   ├── free-fall/
│   │   │   └── scenario.duan
│   │   └── naval-combat/
│   │       └── scenario.duan
│   └── packages/
│       ├── free-fall/
│       └── naval-combat/
├── fixtures/
└── scripts/
```

The repository root is a product and documentation collection. It does not need to be a Cargo package or a root Cargo workspace by default.

`packages/duan-core` is the current runtime implementation directory, and its runtime package name is now `duan-runtime`. The directory has not been renamed yet. `packages/duan-macros` is the current proc-macro crate. A future separate `duan` facade package can live under `packages/duan/` as the user-facing Cargo entry point; that facade has not been introduced yet.

Example simulation capabilities live under `examples/packages/` as ordinary Cargo packages. Scenario examples should converge on `examples/scenarios/<name>/scenario.duan` project directories, which can later hold package locks, assets, run outputs, and generated `.duan/**` caches.

DUAN packages are Rust Cargo packages. The Cargo `package.name` is the DUAN package id, for example `example-freefall-physics`. Package item ids use `<package-id>/<id>`, for example `example-freefall-physics/position-2`.
