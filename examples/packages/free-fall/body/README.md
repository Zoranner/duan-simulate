# Free Fall Body

Rust DUAN package for free-fall body state components.

Use explicit DUAN imports for component authoring. Derive macros stay on the
crate root path, for example `#[derive(duan::Component)]`; runtime and catalog
types are imported by name from `duan::{...}` or `duan::catalog::{...}`.
