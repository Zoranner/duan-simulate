use duan_catalog::{
    DisplayMetadata, FieldSchema, PrimitiveKind, PrimitiveValue, Range, Schema, Unit,
};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Position2 {
    pub x: f64,
    pub y: f64,
}

impl Position2 {
    pub const ITEM_ID: &str = concat!(env!("CARGO_PKG_NAME"), "/position-2");

    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn schema() -> Schema {
        axis_schema("x", 1).field("y", axis_field("Y", 2))
    }
}

fn axis_schema(axis: &str, order: i32) -> Schema {
    Schema::new().field(axis, axis_field(axis.to_ascii_uppercase(), order))
}

fn axis_field(label: impl Into<String>, order: i32) -> FieldSchema {
    FieldSchema::new(PrimitiveKind::Float)
        .default(PrimitiveValue::Float(0.0))
        .range(Range::new(Some(-10_000.0), Some(10_000.0)))
        .unit(Unit::new("m"))
        .display(
            DisplayMetadata::new()
                .label(label)
                .order(order)
                .control("number"),
        )
}

duan::reality!(Position2);
