use duan_package::{
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
        Schema::new()
            .field("x", axis_field("X", 1))
            .field("y", axis_field("Y", 2))
    }
}

fn axis_field(label: impl Into<String>, order: i32) -> FieldSchema {
    FieldSchema::new(PrimitiveKind::Float)
        .default(PrimitiveValue::Float(0.0))
        .range(Range::new(Some(-100_000.0), Some(100_000.0)))
        .unit(Unit::new("m"))
        .display(
            DisplayMetadata::new()
                .label(label)
                .order(order)
                .control("number"),
        )
}

duan::reality!(Position2);
