use duan_package::{
    DisplayMetadata, FieldSchema, PrimitiveKind, PrimitiveValue, Range, Schema, Unit,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Collider {
    pub restitution: f64,
}

impl Collider {
    pub const ITEM_ID: &str = concat!(env!("CARGO_PKG_NAME"), "/collider");

    pub fn new(restitution: f64) -> Self {
        Self { restitution }
    }

    pub fn schema() -> Schema {
        Schema::new().field(
            "restitution",
            FieldSchema::new(PrimitiveKind::Float)
                .default(PrimitiveValue::Float(0.8))
                .range(Range::new(Some(0.0), Some(1.0)))
                .unit(Unit::new("ratio"))
                .display(
                    DisplayMetadata::new()
                        .label("Restitution")
                        .order(1)
                        .control("slider"),
                ),
        )
    }
}

duan::reality!(Collider);
