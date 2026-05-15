use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;

use crate::{PackageError, PackageResult};

pub const PACKAGE_METADATA_FILE: &str = "duan.toml";

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct PackageMetadata {
    duan: DuanMetadata,
    #[serde(default)]
    provides: ProvidesMetadata,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct DuanMetadata {
    package: PackageSection,
    rust: RustSection,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct PackageSection {
    id: String,
    version: String,
    name: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct RustSection {
    #[serde(rename = "crate")]
    crate_name: String,
    entry: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
pub struct ProvidesMetadata {
    #[serde(default)]
    components: BTreeMap<String, String>,
    #[serde(default)]
    domains: BTreeMap<String, String>,
    #[serde(default)]
    entities: BTreeMap<String, String>,
    #[serde(default)]
    events: BTreeMap<String, String>,
    #[serde(default)]
    reactions: BTreeMap<String, String>,
    #[serde(default)]
    observers: BTreeMap<String, String>,
}

impl PackageMetadata {
    pub fn from_toml_str(source: &str) -> PackageResult<Self> {
        toml::from_str(source).map_err(|source| PackageError::ParseMetadata(source.to_string()))
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> PackageResult<Self> {
        Self::load_with_base_path(path).map(|(metadata, _base_path)| metadata)
    }

    pub fn load_with_base_path(path: impl AsRef<Path>) -> PackageResult<(Self, PathBuf)> {
        let path = path.as_ref();
        let (metadata_path, base_path) = if path.is_dir() {
            (path.join(PACKAGE_METADATA_FILE), path.to_path_buf())
        } else {
            let base_path = path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf();
            (path.to_path_buf(), base_path)
        };

        let source =
            fs::read_to_string(&metadata_path).map_err(|source| PackageError::ReadMetadata {
                path: metadata_path.clone(),
                source: source.to_string(),
            })?;
        Self::from_toml_str(&source).map(|metadata| (metadata, base_path))
    }

    pub fn package_id(&self) -> &str {
        &self.duan.package.id
    }

    pub fn package_version(&self) -> &str {
        &self.duan.package.version
    }

    pub fn package_name(&self) -> &str {
        &self.duan.package.name
    }

    pub fn rust_crate(&self) -> &str {
        &self.duan.rust.crate_name
    }

    pub fn rust_entry(&self) -> Option<&str> {
        self.duan.rust.entry.as_deref()
    }

    pub fn provides_components(&self) -> &BTreeMap<String, String> {
        &self.provides.components
    }

    pub fn provides_domains(&self) -> &BTreeMap<String, String> {
        &self.provides.domains
    }

    pub fn provides_entities(&self) -> &BTreeMap<String, String> {
        &self.provides.entities
    }

    pub fn provides_events(&self) -> &BTreeMap<String, String> {
        &self.provides.events
    }

    pub fn provides_reactions(&self) -> &BTreeMap<String, String> {
        &self.provides.reactions
    }

    pub fn provides_observers(&self) -> &BTreeMap<String, String> {
        &self.provides.observers
    }

    pub fn provides_components_count(&self) -> usize {
        self.provides.components.len()
    }

    pub fn provides_count(&self) -> usize {
        self.provides.components.len()
            + self.provides.domains.len()
            + self.provides.entities.len()
            + self.provides.events.len()
            + self.provides.reactions.len()
            + self.provides.observers.len()
    }

    pub fn validate_schema_files(&self, base_dir: impl AsRef<Path>) -> PackageResult<()> {
        let base_dir = base_dir.as_ref();
        validate_schema_group(base_dir, &self.provides.components, "component")?;
        validate_schema_group(base_dir, &self.provides.domains, "domain")?;
        validate_schema_group(base_dir, &self.provides.entities, "entity")?;
        validate_schema_group(base_dir, &self.provides.events, "event")?;
        validate_schema_group(base_dir, &self.provides.reactions, "reaction")?;
        validate_schema_group(base_dir, &self.provides.observers, "observer")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct SchemaHeader {
    id: Option<String>,
    kind: Option<String>,
}

fn validate_schema_group(
    base_dir: &Path,
    entries: &BTreeMap<String, String>,
    expected_kind: &'static str,
) -> PackageResult<()> {
    for (item_id, schema_path) in entries {
        validate_schema_path(item_id, schema_path)?;

        let path = base_dir.join(schema_path);
        let source = fs::read_to_string(&path).map_err(|source| PackageError::ReadSchema {
            item_id: item_id.clone(),
            schema_path: schema_path.clone(),
            path: path.clone(),
            source: source.to_string(),
        })?;
        let schema = parse_schema_header(item_id, schema_path, &source)?;
        let actual_id = schema.id.ok_or_else(|| PackageError::MissingSchemaField {
            item_id: item_id.clone(),
            schema_path: schema_path.clone(),
            field: "id",
        })?;
        if actual_id != *item_id {
            return Err(PackageError::SchemaIdMismatch {
                item_id: item_id.clone(),
                schema_path: schema_path.clone(),
                actual_id,
            });
        }

        let actual_kind = schema
            .kind
            .ok_or_else(|| PackageError::MissingSchemaField {
                item_id: item_id.clone(),
                schema_path: schema_path.clone(),
                field: "kind",
            })?;
        if actual_kind != expected_kind {
            return Err(PackageError::SchemaKindMismatch {
                item_id: item_id.clone(),
                schema_path: schema_path.clone(),
                expected_kind,
                actual_kind,
            });
        }
    }

    Ok(())
}

fn validate_schema_path(item_id: &str, schema_path: &str) -> PackageResult<()> {
    let path = Path::new(schema_path);
    let mut components = path.components();
    let normal_relative = !path.as_os_str().is_empty()
        && !path.is_absolute()
        && components.all(|component| matches!(component, Component::Normal(_)));

    if normal_relative {
        Ok(())
    } else {
        Err(PackageError::InvalidSchemaPath {
            item_id: item_id.to_owned(),
            schema_path: schema_path.to_owned(),
        })
    }
}

fn parse_schema_header(
    item_id: &str,
    schema_path: &str,
    source: &str,
) -> PackageResult<SchemaHeader> {
    serde_json::from_str(source).map_err(|source| PackageError::ParseSchema {
        item_id: item_id.to_owned(),
        schema_path: schema_path.to_owned(),
        source: source.to_string(),
    })
}
