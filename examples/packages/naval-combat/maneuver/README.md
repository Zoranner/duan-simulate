# Naval Combat Maneuver

Rust DUAN package for movement state and maneuver domains.

Use explicit DUAN imports for domain authoring. Attribute macros stay on the
crate root path, for example `#[duan::domain(...)]`; runtime and catalog types
are imported by name from `duan::{...}` or `duan::catalog::{...}`.
