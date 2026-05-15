# DUAN Package Authoring

DUAN package authoring is Rust-first. The Rust crate is the authoritative source for runtime behavior, type definitions, factories, and registration. `duan-package.toml` and `schemas/` are metadata for editors, scenario assembly, validation, and delivery packaging.

Scenario manifests are not a DSL. They select package items and provide initial values. Algorithms, scheduling behavior, domain computation, reaction handling, spawning, and event handling stay in Rust.

## Package Files

A DUAN package should include:

- `Cargo.toml`: the Rust crate identity, dependencies, version, and package include list.
- `src/**`: components, domains, entities, reactions, and the Rust `package()` registration entry.
- `duan-package.toml`: package metadata that maps stable DUAN item ids to schema files.
- `schemas/**`: editor-readable descriptions of component fields and installable package items.

For example:

```toml
[duan.package]
id = "examples.naval-combat.components"
version = "0.1.0"
name = "Naval Combat Components"

[duan.rust]
crate = "examples-naval-combat-components"

[provides.components]
"examples.naval-combat.components.health" = "schemas/health.json"
```

The `provides` mappings are an index. They do not define the runtime implementation. Generated runners still link the Rust crate and call the Rust registration entry.

## Single Package And Split Packages

One `duan-package.toml` describes one DUAN package id. It should not index items that the scenario declares as separate packages. If a scenario lists `examples.free-fall.components`, `examples.free-fall.domains`, and `examples.free-fall.entities`, each package gets its own directory, metadata file, and schema set.

Use a single package only when the crate and runtime registration are intentionally shipped as one package id. Split packages when item ownership is separate, when a scenario depends on only part of a model, or when shared packages are reused by multiple examples. A scenario bundle directory such as `examples/free-fall/` may contain `scenario.yaml` and package subdirectories, but it should not keep a root `duan-package.toml` that suggests the bundle is itself a package.

Shared component identities must come from one package. The baseline 2D motion component schemas live in `packages/duan-kinematics`:

```toml
[duan.package]
id = "duan.kinematics"
version = "0.1.0"
name = "DUAN Kinematics Components"

[duan.rust]
crate = "duan-kinematics"

[provides.components]
"duan.kinematics.position-2" = "schemas/components/position-2.json"
"duan.kinematics.velocity-2" = "schemas/components/velocity-2.json"
```

Example packages should reference `duan.kinematics.position-2` and `duan.kinematics.velocity-2` from scenarios and entity schemas instead of copying those schemas into each example package.

## Component Schema Shape

The first schema version is intentionally small. A component schema records the item id, the item kind, and field metadata that an editor can use to render controls and validate obvious mistakes before a runner is built.

```json
{
  "id": "examples.naval-combat.components.weapon",
  "kind": "component",
  "fields": {
    "range": {
      "type": "float",
      "default": 180.0,
      "unit": "m",
      "range": {
        "min": 0.0,
        "max": 2000.0
      }
    }
  }
}
```

Supported field examples in the initial metadata set are `integer`, `float`, `bool`, and `text`. `default`, `unit`, and `range` are descriptive metadata for authoring tools; Rust code remains responsible for final parsing and behavior.

## Scenario Assembly Boundary

`scenario.yaml` should only assemble existing Rust package capabilities:

- package dependencies and versions;
- domains and reactions to install;
- entity types to spawn;
- component values to attach or override;
- run options and output paths.

If a scenario needs a new algorithm, state transition, event, or command behavior, add it to a Rust package first, expose it through `package()`, then reference the item id from the scenario.

## Example Metadata

The example folders include first-pass metadata:

- `packages/duan-kinematics/duan-package.toml` indexes shared 2D position and velocity component schemas.
- `examples/free-fall/components/`, `examples/free-fall/domains/`, and `examples/free-fall/entities/` each contain one package metadata file and local schemas.
- `examples/naval-combat/components/`, `examples/naval-combat/domains/`, `examples/naval-combat/entities/`, and `examples/naval-combat/reactions/` each contain one package metadata file and local schemas.

These files are examples for editor and scenario tooling. They are not a replacement for crate code or generated runner validation.
