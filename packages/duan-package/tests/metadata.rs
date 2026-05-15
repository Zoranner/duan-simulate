use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use duan_package::{PackageError, PackageMetadata};

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
    assert_eq!(metadata.rust_entry(), None);
    assert_eq!(metadata.provides_components_count(), 2);
}

#[test]
fn metadata_deserializes_rust_entry() {
    let metadata = PackageMetadata::from_toml_str(
        r#"
[duan.package]
id = "duan.kinematics"
version = "0.1.0"
name = "DUAN Kinematics Components"

[duan.rust]
crate = "duan-kinematics"
entry = "duan_kinematics::install"
"#,
    )
    .expect("deserialize package metadata");

    assert_eq!(metadata.rust_entry(), Some("duan_kinematics::install"));
}

#[test]
fn metadata_loads_from_package_file_path() {
    let package_path = workspace_path("examples/free-fall/components/duan-package.toml");

    let metadata = PackageMetadata::load_from_path(&package_path).expect("load metadata");

    assert_eq!(metadata.package_id(), "examples.free-fall.components");
    assert_eq!(metadata.rust_crate(), "examples-free-fall-components");
    assert_eq!(metadata.provides_components_count(), 2);
}

#[test]
fn metadata_loads_duan_package_toml_from_directory() {
    let package_dir = workspace_path("examples/free-fall/components");

    let metadata = PackageMetadata::load_from_path(package_dir).expect("load metadata");

    assert_eq!(metadata.package_id(), "examples.free-fall.components");
    assert_eq!(metadata.provides_components_count(), 2);
}

#[test]
fn metadata_validates_schema_files_for_all_provides_groups() {
    let package_dir = TempPackageDir::new();
    write_package(
        package_dir.path(),
        r#"
[duan.package]
id = "demo.package"
version = "0.1.0"
name = "Demo Package"

[duan.rust]
crate = "demo-package"

[provides.components]
"demo.component" = "schemas/component.json"

[provides.domains]
"demo.domain" = "schemas/domain.json"

[provides.entities]
"demo.entity" = "schemas/entity.json"

[provides.events]
"demo.event" = "schemas/event.json"

[provides.reactions]
"demo.reaction" = "schemas/reaction.json"

[provides.observers]
"demo.observer" = "schemas/observer.json"
"#,
    );

    write_schema(
        package_dir.path(),
        "schemas/component.json",
        "demo.component",
        "component",
    );
    write_schema(
        package_dir.path(),
        "schemas/domain.json",
        "demo.domain",
        "domain",
    );
    write_schema(
        package_dir.path(),
        "schemas/entity.json",
        "demo.entity",
        "entity",
    );
    write_schema(
        package_dir.path(),
        "schemas/event.json",
        "demo.event",
        "event",
    );
    write_schema(
        package_dir.path(),
        "schemas/reaction.json",
        "demo.reaction",
        "reaction",
    );
    write_schema(
        package_dir.path(),
        "schemas/observer.json",
        "demo.observer",
        "observer",
    );

    let metadata = PackageMetadata::load_from_path(package_dir.path()).expect("load metadata");

    metadata
        .validate_schema_files(package_dir.path())
        .expect("schema files should validate");
}

#[test]
fn metadata_rejects_schema_path_traversal() {
    let package_dir = TempPackageDir::new();
    write_package(
        package_dir.path(),
        r#"
[duan.package]
id = "demo.package"
version = "0.1.0"
name = "Demo Package"

[duan.rust]
crate = "demo-package"

[provides.components]
"demo.component" = "../component.json"
"#,
    );

    let metadata = PackageMetadata::load_from_path(package_dir.path()).expect("load metadata");
    let error = metadata
        .validate_schema_files(package_dir.path())
        .expect_err("schema path should be rejected");

    assert!(matches!(
        error,
        PackageError::InvalidSchemaPath {
            ref item_id,
            ref schema_path,
        } if item_id == "demo.component" && schema_path == "../component.json"
    ));
    assert!(error.to_string().contains("demo.component"));
    assert!(error.to_string().contains("../component.json"));
}

#[test]
fn metadata_rejects_missing_schema_file() {
    let package_dir = TempPackageDir::new();
    write_package(
        package_dir.path(),
        r#"
[duan.package]
id = "demo.package"
version = "0.1.0"
name = "Demo Package"

[duan.rust]
crate = "demo-package"

[provides.components]
"demo.component" = "schemas/missing.json"
"#,
    );

    let metadata = PackageMetadata::load_from_path(package_dir.path()).expect("load metadata");
    let error = metadata
        .validate_schema_files(package_dir.path())
        .expect_err("missing schema should be rejected");

    assert!(matches!(
        error,
        PackageError::ReadSchema {
            item_id,
            schema_path,
            ..
        } if item_id == "demo.component" && schema_path == "schemas/missing.json"
    ));
}

#[test]
fn metadata_rejects_schema_id_mismatch() {
    let package_dir = TempPackageDir::new();
    write_package(
        package_dir.path(),
        r#"
[duan.package]
id = "demo.package"
version = "0.1.0"
name = "Demo Package"

[duan.rust]
crate = "demo-package"

[provides.components]
"demo.component" = "schemas/component.json"
"#,
    );
    write_schema(
        package_dir.path(),
        "schemas/component.json",
        "demo.other",
        "component",
    );

    let metadata = PackageMetadata::load_from_path(package_dir.path()).expect("load metadata");
    let error = metadata
        .validate_schema_files(package_dir.path())
        .expect_err("id mismatch should be rejected");

    assert!(matches!(
        error,
        PackageError::SchemaIdMismatch {
            item_id,
            schema_path,
            actual_id,
        } if item_id == "demo.component"
            && schema_path == "schemas/component.json"
            && actual_id == "demo.other"
    ));
}

#[test]
fn metadata_rejects_schema_kind_mismatch() {
    let package_dir = TempPackageDir::new();
    write_package(
        package_dir.path(),
        r#"
[duan.package]
id = "demo.package"
version = "0.1.0"
name = "Demo Package"

[duan.rust]
crate = "demo-package"

[provides.components]
"demo.component" = "schemas/component.json"
"#,
    );
    write_schema(
        package_dir.path(),
        "schemas/component.json",
        "demo.component",
        "entity",
    );

    let metadata = PackageMetadata::load_from_path(package_dir.path()).expect("load metadata");
    let error = metadata
        .validate_schema_files(package_dir.path())
        .expect_err("kind mismatch should be rejected");

    assert!(matches!(
        error,
        PackageError::SchemaKindMismatch {
            item_id,
            schema_path,
            expected_kind: "component",
            actual_kind,
        } if item_id == "demo.component"
            && schema_path == "schemas/component.json"
            && actual_kind == "entity"
    ));
}

fn write_package(package_dir: &std::path::Path, source: &str) {
    std::fs::write(package_dir.join("duan-package.toml"), source).expect("write package metadata");
}

fn write_schema(package_dir: &std::path::Path, relative_path: &str, id: &str, kind: &str) {
    let path = package_dir.join(relative_path);
    std::fs::create_dir_all(path.parent().expect("schema parent")).expect("create schema dir");
    std::fs::write(
        path,
        format!(
            r#"{{
  "id": "{id}",
  "kind": "{kind}",
  "fields": {{
    "value": {{
      "type": "float"
    }}
  }}
}}"#
        ),
    )
    .expect("write schema");
}

struct TempPackageDir {
    path: PathBuf,
}

impl TempPackageDir {
    fn new() -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "duan-package-metadata-test-{}-{nanos}-{id}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).expect("create temp package dir");

        Self { path }
    }

    fn path(&self) -> &std::path::Path {
        &self.path
    }
}

impl Drop for TempPackageDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
