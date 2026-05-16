# Free Fall Gravity

Rust DUAN package for the free-fall gravity domain.

Use explicit DUAN imports for domain authoring. Attribute macros stay on the
crate root path, for example `#[duan::domain(...)]`; runtime and catalog types
are imported by name from `duan::{...}` or `duan::catalog::{...}`.
