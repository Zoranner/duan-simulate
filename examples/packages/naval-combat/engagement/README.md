# Naval Combat Engagement

Rust DUAN package for weapon state, combat domain, events, and reactions.

Use explicit DUAN imports for combat authoring. Macros stay on the crate root
path, for example `#[derive(duan::Component)]`, `#[derive(duan::Event)]`,
`#[duan::domain(...)]`, and `#[duan::reaction(...)]`; runtime and catalog types
are imported by name from `duan::{...}` or `duan::catalog::{...}`.
