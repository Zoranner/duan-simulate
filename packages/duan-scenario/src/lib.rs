//! Scenario manifest model, YAML loading, and structural validation.

mod error;
mod manifest;
mod validation;
mod value;

use std::path::Path;

pub use error::{ManifestError, ValidationError};
pub use manifest::*;
pub use validation::validate_manifest;
pub use value::Value;

pub const CRATE_NAME: &str = env!("CARGO_PKG_NAME");

pub fn load_yaml_str(source: &str) -> Result<Manifest, ManifestError> {
    let manifest = serde_yaml::from_str::<Manifest>(source)?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

pub fn load_yaml_file(path: impl AsRef<Path>) -> Result<Manifest, ManifestError> {
    let source = std::fs::read_to_string(path.as_ref()).map_err(|source| ManifestError::Io {
        path: path.as_ref().to_path_buf(),
        source,
    })?;
    load_yaml_str(&source)
}

pub fn load_from_path(path: impl AsRef<Path>) -> Result<Manifest, ManifestError> {
    load_yaml_file(path)
}
