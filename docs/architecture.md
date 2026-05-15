# DUAN Platform Architecture

## Runtime Boundary

`duan-core` remains the hot runtime path. It keeps `World::step`, `Belief / Intent / Reality`, `Entity::tick`, `Domain::compute`, `Reaction::react`, storage, snapshots, scheduling, events, and command commit as Rust-first runtime behavior.

The platform layer adds package-facing APIs around the core:

- Package registry.
- Component schema and decode.
- Entity, domain, reaction, and observer factories.
- Scenario manifest parsing and validation.
- Generated runner creation.

## Distribution

Package distribution is based on a private Cargo registry. DUAN does not need a separate default package service.

The editor can download `.crate` packages and read generated install caches derived from Rust package registration and schema APIs. Source packages do not hand-author `duan.toml` or `schemas/` as facts; those files are cache artifacts for inspection, validation, and delivery packaging.

Generated runners statically link selected Cargo packages. Runtime behavior remains compiled Rust.
