use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Schema {
    fields: BTreeMap<String, FieldSchema>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldSchema {
    #[serde(rename = "type", alias = "kind")]
    kind: PrimitiveKind,
    default: Option<PrimitiveValue>,
    range: Option<Range>,
    unit: Option<Unit>,
    editor: Option<EditorMetadata>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PrimitiveKind {
    Bool,
    Integer,
    Float,
    Text,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PrimitiveValue {
    Bool(bool),
    Integer(i64),
    Float(f64),
    Text(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Range {
    min: Option<f64>,
    max: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Unit(String);

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorMetadata {
    label: Option<String>,
    description: Option<String>,
    order: Option<i32>,
    group: Option<String>,
    control: Option<String>,
}

impl Schema {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn field(mut self, name: impl Into<String>, schema: FieldSchema) -> Self {
        self.fields.insert(name.into(), schema);
        self
    }

    pub fn fields(&self) -> &BTreeMap<String, FieldSchema> {
        &self.fields
    }

    pub fn field_schema(&self, name: &str) -> Option<&FieldSchema> {
        self.fields.get(name)
    }
}

impl FieldSchema {
    pub fn new(kind: PrimitiveKind) -> Self {
        Self {
            kind,
            default: None,
            range: None,
            unit: None,
            editor: None,
        }
    }

    pub fn default(mut self, value: PrimitiveValue) -> Self {
        self.default = Some(value);
        self
    }

    pub fn range(mut self, range: Range) -> Self {
        self.range = Some(range);
        self
    }

    pub fn unit(mut self, unit: Unit) -> Self {
        self.unit = Some(unit);
        self
    }

    pub fn editor(mut self, metadata: EditorMetadata) -> Self {
        self.editor = Some(metadata);
        self
    }

    pub fn kind(&self) -> &PrimitiveKind {
        &self.kind
    }

    pub fn default_value(&self) -> Option<&PrimitiveValue> {
        self.default.as_ref()
    }

    pub fn range_value(&self) -> Option<&Range> {
        self.range.as_ref()
    }

    pub fn unit_value(&self) -> Option<&Unit> {
        self.unit.as_ref()
    }

    pub fn editor_metadata(&self) -> Option<&EditorMetadata> {
        self.editor.as_ref()
    }
}

impl Range {
    pub fn new(min: Option<f64>, max: Option<f64>) -> Self {
        Self { min, max }
    }

    pub fn min(&self) -> Option<f64> {
        self.min
    }

    pub fn max(&self) -> Option<f64> {
        self.max
    }
}

impl Unit {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl EditorMetadata {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn order(mut self, order: i32) -> Self {
        self.order = Some(order);
        self
    }

    pub fn group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }

    pub fn control(mut self, control: impl Into<String>) -> Self {
        self.control = Some(control.into());
        self
    }

    pub fn label_text(&self) -> Option<&str> {
        self.label.as_deref()
    }

    pub fn description_value(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn order_value(&self) -> Option<i32> {
        self.order
    }

    pub fn group_value(&self) -> Option<&str> {
        self.group.as_deref()
    }

    pub fn control_value(&self) -> Option<&str> {
        self.control.as_deref()
    }
}
