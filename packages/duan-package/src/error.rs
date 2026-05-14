use std::fmt;
use std::path::PathBuf;

use crate::{ItemId, PackageId};

pub type PackageResult<T> = Result<T, PackageError>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackageError {
    InvalidId(String),
    InvalidVersionReq(String),
    ReadMetadata {
        path: PathBuf,
        source: String,
    },
    ParseMetadata(String),
    InvalidSchemaPath {
        item_id: String,
        schema_path: String,
    },
    ReadSchema {
        item_id: String,
        schema_path: String,
        path: PathBuf,
        source: String,
    },
    ParseSchema {
        item_id: String,
        schema_path: String,
        source: String,
    },
    MissingSchemaField {
        item_id: String,
        schema_path: String,
        field: &'static str,
    },
    SchemaIdMismatch {
        item_id: String,
        schema_path: String,
        actual_id: String,
    },
    SchemaKindMismatch {
        item_id: String,
        schema_path: String,
        expected_kind: &'static str,
        actual_kind: String,
    },
    DuplicatePackage(PackageId),
    DuplicateItem(ItemId),
    MissingItem {
        location: String,
        item_id: ItemId,
    },
    UnexpectedItemKind {
        location: String,
        item_id: ItemId,
        expected_kind: &'static str,
    },
    UnsupportedComponentOverride {
        entity_id: String,
        entity_type: ItemId,
        component_id: ItemId,
    },
}

impl PackageError {
    pub fn duplicate_item_id(&self) -> Option<&ItemId> {
        match self {
            Self::DuplicateItem(id) => Some(id),
            _ => None,
        }
    }
}

impl fmt::Display for PackageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidId(value) => write!(f, "invalid DUAN id `{value}`"),
            Self::InvalidVersionReq(value) => write!(f, "invalid version requirement `{value}`"),
            Self::ReadMetadata { path, source } => {
                write!(
                    f,
                    "failed to read package metadata `{}`: {source}",
                    path.display()
                )
            }
            Self::ParseMetadata(source) => write!(f, "failed to parse package metadata: {source}"),
            Self::InvalidSchemaPath {
                item_id,
                schema_path,
            } => write!(
                f,
                "invalid schema path `{schema_path}` for package item `{item_id}`"
            ),
            Self::ReadSchema {
                item_id,
                schema_path,
                path,
                source,
            } => write!(
                f,
                "failed to read schema `{schema_path}` for package item `{item_id}` at `{}`: {source}",
                path.display()
            ),
            Self::ParseSchema {
                item_id,
                schema_path,
                source,
            } => write!(
                f,
                "failed to parse schema `{schema_path}` for package item `{item_id}`: {source}"
            ),
            Self::MissingSchemaField {
                item_id,
                schema_path,
                field,
            } => write!(
                f,
                "schema `{schema_path}` for package item `{item_id}` is missing `{field}`"
            ),
            Self::SchemaIdMismatch {
                item_id,
                schema_path,
                actual_id,
            } => write!(
                f,
                "schema `{schema_path}` for package item `{item_id}` declares id `{actual_id}`"
            ),
            Self::SchemaKindMismatch {
                item_id,
                schema_path,
                expected_kind,
                actual_kind,
            } => write!(
                f,
                "schema `{schema_path}` for package item `{item_id}` declares kind `{actual_kind}`, expected `{expected_kind}`"
            ),
            Self::DuplicatePackage(id) => write!(f, "duplicate package `{id}`"),
            Self::DuplicateItem(id) => write!(f, "duplicate package item `{id}`"),
            Self::MissingItem { location, item_id } => {
                write!(f, "missing package item `{item_id}` at {location}")
            }
            Self::UnexpectedItemKind {
                location,
                item_id,
                expected_kind,
            } => write!(
                f,
                "package item `{item_id}` at {location} is not a {expected_kind}"
            ),
            Self::UnsupportedComponentOverride {
                entity_id,
                entity_type,
                component_id,
            } => write!(
                f,
                "entity `{entity_id}` of type `{entity_type}` does not support component override `{component_id}`"
            ),
        }
    }
}

impl std::error::Error for PackageError {}
