use duan_catalog::{
    DisplayMetadata, FieldSchema, PrimitiveKind, PrimitiveValue, Range, Schema, Unit,
};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Velocity2 {
    pub vx: f64,
    pub vy: f64,
}

impl Velocity2 {
    pub const ITEM_ID: &str = concat!(env!("CARGO_PKG_NAME"), "/velocity-2");

    pub fn new(vx: f64, vy: f64) -> Self {
        Self { vx, vy }
    }

    pub fn schema() -> Schema {
        Schema::new()
            .field("vx", velocity_field("VX", 1))
            .field("vy", velocity_field("VY", 2))
    }
}

fn velocity_field(label: impl Into<String>, order: i32) -> FieldSchema {
    FieldSchema::new(PrimitiveKind::Float)
        .default(PrimitiveValue::Float(0.0))
        .range(Range::new(Some(-1_000.0), Some(1_000.0)))
        .unit(Unit::new("m/s"))
        .display(
            DisplayMetadata::new()
                .label(label)
                .order(order)
                .control("number"),
        )
}

duan::reality!(Velocity2);
