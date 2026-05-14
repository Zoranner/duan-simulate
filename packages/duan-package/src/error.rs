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
