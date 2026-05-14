use std::path::PathBuf;

use duan_package::PackageMetadata;

fn workspace_path(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

#[test]
fn metadata_deserializes_minimal_duan_package_toml() {
    let metadata = PackageMetadata::from_toml_str(
        r#"
[duan.package]
id = "duan.kinematics"
version = "0.1.0"
name = "DUAN Kinematics Components"

[duan.rust]
crate = "duan-kinematics"

[provides.components]
"duan.kinematics.position-2" = "schemas/components/position-2.json"
"duan.kinematics.velocity-2" = "schemas/components/velocity-2.json"
"#,
    )
    .expect("deserialize package metadata");

    assert_eq!(metadata.package_id(), "duan.kinematics");
    assert_eq!(metadata.package_version(), "0.1.0");
    assert_eq!(metadata.package_name(), "DUAN Kinematics Components");
    assert_eq!(metadata.rust_crate(), "duan-kinematics");
    assert_eq!(metadata.provides_components_count(), 2);
}

#[test]
fn metadata_loads_from_package_file_path() {
    let package_path = workspace_path("examples/free-fall/duan-package.toml");

    let metadata = PackageMetadata::load_from_path(&package_path).expect("load metadata");

    assert_eq!(metadata.package_id(), "examples.free-fall.components");
    assert_eq!(metadata.rust_crate(), "examples-free-fall-components");
    assert_eq!(metadata.provides_components_count(), 2);
}

#[test]
fn metadata_loads_duan_package_toml_from_directory() {
    let package_dir = workspace_path("examples/free-fall");

    let metadata = PackageMetadata::load_from_path(package_dir).expect("load metadata");

    assert_eq!(metadata.package_id(), "examples.free-fall.components");
    assert_eq!(metadata.provides_components_count(), 2);
}
