# Naval Combat Platform

Rust DUAN package for ship platform state components.

Use explicit DUAN imports for component authoring. Derive macros stay on the
crate root path, for example `#[derive(duan::Component)]`; runtime and catalog
types are imported by name from `duan::{...}` or `duan::catalog::{...}`.
