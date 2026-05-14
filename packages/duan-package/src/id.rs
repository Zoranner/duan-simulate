use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{PackageError, PackageResult};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PackageId(String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ItemId(String);

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VersionReqText(String);

impl PackageId {
    pub fn new(value: impl Into<String>) -> PackageResult<Self> {
        let value = value.into();
        validate_dotted_id(&value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ItemId {
    pub fn new(value: impl Into<String>) -> PackageResult<Self> {
        let value = value.into();
        validate_dotted_id(&value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl VersionReqText {
    pub fn new(value: impl Into<String>) -> PackageResult<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(PackageError::InvalidVersionReq(value));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Display for ItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Display for VersionReqText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn validate_dotted_id(value: &str) -> PackageResult<()> {
    if value.is_empty() {
        return Err(PackageError::InvalidId(value.to_owned()));
    }

    for segment in value.split('.') {
        if segment.is_empty() || !valid_segment(segment) {
            return Err(PackageError::InvalidId(value.to_owned()));
        }
    }

    Ok(())
}

fn valid_segment(segment: &str) -> bool {
    if segment.starts_with('-') || segment.ends_with('-') || segment.contains("--") {
        return false;
    }

    segment
        .bytes()
        .all(|byte| matches!(byte, b'a'..=b'z' | b'0'..=b'9' | b'-'))
}
