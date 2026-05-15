use std::fs;

use duan_runner_generator::model::{
    CrateDependency, PackageInstall, RegistryConfig, RunnerProject,
};
use duan_runner_generator::writer::write_runner_project;

#[test]
fn writes_deterministic_runner_project_files() {
    let tempdir = tempfile::tempdir().expect("create tempdir");
    let project = RunnerProject {
        name: "free-fall-runner".to_string(),
        version: "0.1.0".to_string(),
        registry: RegistryConfig {
            name: "duan-private".to_string(),
            index: "sparse+https://registry.example.test/api/v1/crates/".to_string(),
        },
        scenario_path: "scenario/free-fall.duan".to_string(),
        dependencies: vec![
            CrateDependency::registry("free-fall-package", "0.1.0", "duan-private"),
            CrateDependency::registry("duan-scenario", "0.1.0", "duan-private"),
            CrateDependency::registry("duan", "0.1.0", "duan-private"),
            CrateDependency::registry("duan-package", "0.1.0", "duan-private"),
        ],
        installs: vec![
            PackageInstall::new("examples-free-fall-body"),
            PackageInstall::new("free-fall-package"),
        ],
    };

    write_runner_project(&project, tempdir.path()).expect("write runner project");
    write_runner_project(&project, tempdir.path()).expect("rewrite runner project");

    let cargo_toml =
        fs::read_to_string(tempdir.path().join("Cargo.toml")).expect("read Cargo.toml");
    assert_eq!(
        cargo_toml,
        r#"[package]
name = "free-fall-runner"
version = "0.1.0"
edition = "2021"

[dependencies]
duan = { version = "0.1.0", registry = "duan-private" }
duan-package = { version = "0.1.0", registry = "duan-private" }
duan-runner = { version = "0.1.0", registry = "duan-private" }
duan-scenario = { version = "0.1.0", registry = "duan-private" }
free-fall-package = { version = "0.1.0", registry = "duan-private" }
"#
    );

    let config_toml =
        fs::read_to_string(tempdir.path().join(".cargo/config.toml")).expect("read config");
    assert_eq!(
        config_toml,
        r#"[registries.duan-private]
index = "sparse+https://registry.example.test/api/v1/crates/"
"#
    );

    let main_rs = fs::read_to_string(tempdir.path().join("src/main.rs")).expect("read main.rs");
    assert_eq!(
        main_rs,
        r#"use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scenario_path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("scenario/free-fall.duan"));

    let scenario = duan_scenario::load_from_path(&scenario_path)?;
    let registry = duan_package::Registry::new()
        .install(examples_free_fall_body::package())?
        .install(free_fall_package::package())?;

    let report = duan_runner::Runner::new(registry).run(&scenario)?;
    println!("{report:?}");
    Ok(())
}

"#
    );
}
