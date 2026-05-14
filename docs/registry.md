# Private Cargo Registry

DUAN package distribution uses a private Cargo registry.

Generated runner projects should include a local `.cargo/config.toml` like:

```toml
[registries.duan-private]
index = "sparse+https://registry.example.com/api/v1/crates/"
```

The URL is deployment-specific. This repository intentionally keeps it as documentation rather than an active root Cargo configuration.
