use duan_package::{
    DisplayMetadata, FieldSchema, PrimitiveKind, PrimitiveValue, Range, Schema, Unit,
};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Heading {
    pub radians: f64,
}

impl Heading {
    pub const ITEM_ID: &str = concat!(env!("CARGO_PKG_NAME"), "/heading");

    pub fn schema() -> Schema {
        Schema::new().field(
            "radians",
            FieldSchema::new(PrimitiveKind::Float)
                .default(PrimitiveValue::Float(0.0))
                .range(Range::new(
                    Some(-std::f64::consts::PI),
                    Some(std::f64::consts::PI),
                ))
                .unit(Unit::new("rad"))
                .display(
                    DisplayMetadata::new()
                        .label("Heading")
                        .order(1)
                        .control("angle"),
                ),
        )
    }
}

duan::intent!(Heading);
