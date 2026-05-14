use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn duan() -> Command {
    Command::new(env!("CARGO_BIN_EXE_duan"))
}

fn workspace_path(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

#[test]
fn scenario_validate_accepts_example_manifest() {
    let scenario = workspace_path("examples/free-fall/scenario.yaml");
    let scenario_arg = scenario.to_string_lossy().into_owned();

    let output = duan()
        .args(["scenario", "validate", &scenario_arg])
        .output()
        .expect("run duan scenario validate");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!("valid: {scenario_arg}\n")
    );
}

#[test]
fn runner_generate_writes_project_without_building_it() {
    let tempdir = tempfile::tempdir().expect("create tempdir");
    let out_dir = tempdir.path().join("runner");
    let out_arg = out_dir.to_string_lossy().into_owned();
    let scenario = workspace_path("examples/free-fall/scenario.yaml");
    let scenario_arg = scenario.to_string_lossy().into_owned();

    let output = duan()
        .args([
            "runner",
            "generate",
            &scenario_arg,
            "--out",
            &out_arg,
            "--registry-name",
            "duan-private",
            "--registry-index",
            "sparse+https://registry.example.test/api/v1/crates/",
        ])
        .output()
        .expect("run duan runner generate");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains(&format!("generated: {out_arg}")));

    let cargo_toml = fs::read_to_string(out_dir.join("Cargo.toml")).expect("read Cargo.toml");
    assert!(cargo_toml.contains("name = \"free-fall-demo-runner\""));
    assert!(cargo_toml.contains(
        "examples-free-fall-components = { version = \"0.1.0\", registry = \"duan-private\" }"
    ));

    let main_rs = fs::read_to_string(out_dir.join("src/main.rs")).expect("read main.rs");
    assert!(main_rs.contains("run_scenario(scenario, registry, \"duan_runner::run_scenario\")?;"));
    assert!(main_rs.contains("runner execution is not implemented in generated runner stub"));
}

#[test]
fn package_inspect_reads_metadata_from_directory() {
    let package_dir = workspace_path("examples/free-fall");
    let package_arg = package_dir.to_string_lossy().into_owned();

    let output = duan()
        .args(["package", "inspect", &package_arg])
        .output()
        .expect("run duan package inspect");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "package: examples.free-fall.components\nversion: 0.1.0\ncrate: examples-free-fall-components\nprovides: 5\n"
    );
}

#[test]
fn package_inspect_reads_metadata_from_file() {
    let package_file = workspace_path("examples/free-fall/duan-package.toml");
    let package_arg = package_file.to_string_lossy().into_owned();

    let output = duan()
        .args(["package", "inspect", &package_arg])
        .output()
        .expect("run duan package inspect");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("crate: examples-free-fall-components\n")
    );
}

#[test]
fn runner_build_requires_runner_manifest() {
    let tempdir = tempfile::tempdir().expect("create tempdir");
    let runner_dir = tempdir.path().join("missing-runner");
    let runner_arg = runner_dir.to_string_lossy().into_owned();

    let output = duan()
        .args(["runner", "build", &runner_arg])
        .output()
        .expect("run duan runner build");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("runner directory"));
    assert!(stderr.contains(&runner_arg));
    assert!(stderr.contains("Cargo.toml"));
}

#[test]
fn runner_build_rejects_missing_path_argument() {
    let output = duan()
        .args(["runner", "build"])
        .output()
        .expect("run duan runner build without path");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("<RUNNER_DIR>"));
}

#[test]
fn deliver_copies_runner_scenario_and_writes_readme() {
    let tempdir = tempfile::tempdir().expect("create tempdir");
    let runner = tempdir.path().join("runner.exe");
    let scenario = tempdir.path().join("scenario.yaml");
    let out_dir = tempdir.path().join("delivery");
    fs::write(&runner, "runner-binary").expect("write runner");
    fs::write(&scenario, "scenario: demo").expect("write scenario");

    let runner_arg = runner.to_string_lossy().into_owned();
    let scenario_arg = scenario.to_string_lossy().into_owned();
    let out_arg = out_dir.to_string_lossy().into_owned();
    let output = duan()
        .args([
            "deliver",
            &scenario_arg,
            "--runner",
            &runner_arg,
            "--out",
            &out_arg,
        ])
        .output()
        .expect("run duan deliver");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains(&format!("delivered: {out_arg}")));
    assert_eq!(
        fs::read_to_string(out_dir.join("runner.exe")).expect("read delivered runner"),
        "runner-binary"
    );
    assert_eq!(
        fs::read_to_string(out_dir.join("scenario/scenario.yaml"))
            .expect("read delivered scenario"),
        "scenario: demo"
    );
    assert!(out_dir.join("runs").is_dir());
    let readme = fs::read_to_string(out_dir.join("README.md")).expect("read delivery README");
    assert!(readme.contains("scenario/scenario.yaml"));
    assert!(readme.contains("客户可以修改 scenario"));
    assert!(readme.contains("不可修改 Rust 逻辑"));
    assert!(readme.contains("不可修改 package set"));
}

#[test]
fn deliver_error_mentions_scenario_runner_and_out_paths() {
    let tempdir = tempfile::tempdir().expect("create tempdir");
    let runner = tempdir.path().join("missing-runner.exe");
    let scenario = tempdir.path().join("scenario.yaml");
    let out_dir = tempdir.path().join("delivery");
    let runner_arg = runner.to_string_lossy().into_owned();
    let scenario_arg = scenario.to_string_lossy().into_owned();
    let out_arg = out_dir.to_string_lossy().into_owned();

    let output = duan()
        .args([
            "deliver",
            &scenario_arg,
            "--runner",
            &runner_arg,
            "--out",
            &out_arg,
        ])
        .output()
        .expect("run duan deliver");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(&scenario_arg));
    assert!(stderr.contains(&runner_arg));
    assert!(stderr.contains(&out_arg));
}

#[test]
fn run_is_explicitly_not_implemented_and_fails() {
    let output = duan().args(["run"]).output().expect("run duan run");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains(
        "not implemented: run runner execution is not implemented yet; use runner build and deliver"
    ));
}
