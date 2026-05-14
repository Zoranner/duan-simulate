use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::{PackageError, PackageResult};

pub const PACKAGE_METADATA_FILE: &str = "duan-package.toml";

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
        let path = path.as_ref();
        let metadata_path = if path.is_dir() {
            path.join(PACKAGE_METADATA_FILE)
        } else {
            path.to_path_buf()
        };

        let source =
            fs::read_to_string(&metadata_path).map_err(|source| PackageError::ReadMetadata {
                path: metadata_path.clone(),
                source: source.to_string(),
            })?;
        Self::from_toml_str(&source)
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
}
