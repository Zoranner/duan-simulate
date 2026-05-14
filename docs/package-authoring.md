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
id = "examples.naval-combat"
version = "0.1.0"
name = "DUAN Naval Combat Example"

[duan.rust]
crate = "examples-naval-combat"
entry = "examples_naval_combat::package"

[provides.components]
"examples.naval-combat.components.health" = "schemas/components/health.json"

[provides.domains]
"examples.naval-combat.domains.combat" = "schemas/domains/combat.json"

[provides.entities]
"examples.naval-combat.entities.ship" = "schemas/entities/ship.json"

[provides.reactions]
"examples.naval-combat.reactions.on-hit" = "schemas/reactions/on-hit.json"
```

The `provides` mappings are an index. They do not define the runtime implementation. Generated runners still link the Rust crate and call the Rust registration entry.

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

- `examples/free-fall/schemas/` documents the components used by `examples/free-fall/scenario.yaml`.
- `examples/naval-combat/duan-package.toml` indexes the naval combat package items.
- `examples/naval-combat/schemas/` documents the naval combat components, domains, entity, and reactions used by `examples/naval-combat/scenario.yaml`.

These files are examples for editor and scenario tooling. They are not a replacement for crate code or generated runner validation.
