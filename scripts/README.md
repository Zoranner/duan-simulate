# Scripts

- `verify-rust.ps1`: runs the repository-level Rust verification matrix across the framework crates and example packages covered by the Rust engineering review. The runtime package uses `cargo test --lib --tests --all-features` so regular tests do not execute Criterion bench targets in test mode. Public framework crates also run `cargo doc --all-features --no-deps`.
