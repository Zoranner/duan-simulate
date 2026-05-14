use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub scenario: Scenario,
    #[serde(default)]
    pub packages: Vec<PackageDependency>,
    #[serde(default)]
    pub domains: Vec<DomainInstall>,
    #[serde(default)]
    pub reactions: Vec<ReactionInstall>,
    #[serde(default)]
    pub entities: Vec<EntityInstance>,
    #[serde(default)]
    pub run: Option<RunOptions>,
    #[serde(default)]
    pub outputs: Option<OutputSpec>,
    #[serde(default)]
    pub experiments: Vec<Experiment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub id: String,
    pub package: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackageDependency {
    pub id: String,
    pub version: String,
    #[serde(default)]
    pub registry: Option<String>,
    #[serde(default)]
    pub features: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DomainInstall {
    #[serde(rename = "type")]
    pub type_id: String,
    #[serde(default)]
    pub factory: Option<String>,
    #[serde(default)]
    pub options: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReactionInstall {
    #[serde(rename = "type")]
    pub type_id: String,
    #[serde(default)]
    pub factory: Option<String>,
    #[serde(default)]
    pub options: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityInstance {
    pub id: String,
    #[serde(rename = "type")]
    pub type_id: String,
    #[serde(default)]
    pub components: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunOptions {
    #[serde(default)]
    pub delta_time: Option<f64>,
    #[serde(default)]
    pub max_time: Option<f64>,
    #[serde(default)]
    pub steps: Option<u64>,
    #[serde(default)]
    pub seed: Option<u64>,
    #[serde(default)]
    pub output_dir: Option<String>,
    #[serde(default)]
    pub options: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OutputSpec {
    Map(BTreeMap<String, String>),
    List(Vec<OutputTarget>),
}

impl OutputSpec {
    pub fn paths(&self) -> Vec<(&str, &str)> {
        match self {
            Self::Map(outputs) => outputs
                .iter()
                .map(|(name, path)| (name.as_str(), path.as_str()))
                .collect(),
            Self::List(outputs) => outputs
                .iter()
                .map(|output| (output.id.as_str(), output.path.as_str()))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputTarget {
    pub id: String,
    pub path: String,
    #[serde(default)]
    pub format: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Experiment {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub run: Option<RunOptions>,
    #[serde(default)]
    pub overrides: ExperimentOverrides,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExperimentOverrides {
    #[serde(default)]
    pub components: BTreeMap<String, BTreeMap<String, Value>>,
    #[serde(default)]
    pub parameters: BTreeMap<String, Value>,
}
