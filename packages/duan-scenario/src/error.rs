use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("failed to read scenario manifest `{path}`: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse scenario manifest: {0}")]
    Parse(#[from] serde_yaml::Error),

    #[error("scenario manifest validation failed")]
    Validation(Vec<ValidationError>),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ValidationError {
    #[error("duplicate entity id `{id}`")]
    DuplicateEntityId { id: String },

    #[error("invalid package id `{id}` at {location}")]
    InvalidPackageId { location: String, id: String },

    #[error("invalid item id `{id}` at {location}")]
    InvalidItemId { location: String, id: String },

    #[error("invalid component reference `{id}` at {location}")]
    InvalidComponentRef { location: String, id: String },

    #[error("missing package dependency for `{item}` at {location}")]
    MissingPackageDependency { location: String, item: String },

    #[error("invalid run/output path `{path}` at {location}")]
    InvalidPath { location: String, path: String },
}
