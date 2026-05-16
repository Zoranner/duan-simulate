# Naval Combat Fleet Objects

Rust DUAN package for naval combat entity templates.

Use explicit DUAN imports for entity authoring. Attribute macros stay on the
crate root path, for example `#[duan::entity(...)]`; runtime and catalog types
are imported by name from `duan::{...}` or `duan::catalog::{...}`.
