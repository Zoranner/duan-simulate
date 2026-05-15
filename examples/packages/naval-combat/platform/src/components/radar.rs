use duan_package::{
    DisplayMetadata, FieldSchema, PrimitiveKind, PrimitiveValue, Range, Schema, Unit,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Radar {
    pub range: f64,
}

impl Radar {
    pub const ITEM_ID: &str = concat!(env!("CARGO_PKG_NAME"), "/radar");

    pub fn new(range: f64) -> Self {
        Self { range }
    }

    pub fn schema() -> Schema {
        Schema::new().field(
            "range",
            FieldSchema::new(PrimitiveKind::Float)
                .default(PrimitiveValue::Float(260.0))
                .range(Range::new(Some(0.0), Some(10_000.0)))
                .unit(Unit::new("m"))
                .display(
                    DisplayMetadata::new()
                        .label("Range")
                        .order(1)
                        .control("number"),
                ),
        )
    }
}

duan::reality!(Radar);
