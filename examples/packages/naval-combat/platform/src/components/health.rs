use duan_catalog::{DisplayMetadata, FieldSchema, PrimitiveKind, PrimitiveValue, Range, Schema};

#[derive(Debug, Clone, PartialEq)]
pub struct Health {
    pub current: f64,
    pub max: f64,
}

impl Health {
    pub const ITEM_ID: &str = concat!(env!("CARGO_PKG_NAME"), "/health");

    pub fn new(max: f64) -> Self {
        Self { current: max, max }
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }

    pub fn schema() -> Schema {
        Schema::new()
            .field("current", health_field("Current", 1))
            .field("max", health_field("Max", 2))
    }
}

fn health_field(label: impl Into<String>, order: i32) -> FieldSchema {
    FieldSchema::new(PrimitiveKind::Float)
        .default(PrimitiveValue::Float(100.0))
        .range(Range::new(Some(0.0), Some(10_000.0)))
        .display(
            DisplayMetadata::new()
                .label(label)
                .order(order)
                .control("number"),
        )
}

duan::reality!(Health);
