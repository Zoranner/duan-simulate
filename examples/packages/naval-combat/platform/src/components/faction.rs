use duan_catalog::{DisplayMetadata, FieldSchema, PrimitiveKind, PrimitiveValue, Range, Schema};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Faction {
    pub team: u8,
}

impl Faction {
    pub const ITEM_ID: &str = concat!(env!("CARGO_PKG_NAME"), "/faction");

    pub fn red() -> Self {
        Self { team: 0 }
    }

    pub fn blue() -> Self {
        Self { team: 1 }
    }

    pub fn schema() -> Schema {
        Schema::new().field(
            "team",
            FieldSchema::new(PrimitiveKind::Integer)
                .default(PrimitiveValue::Integer(0))
                .range(Range::new(Some(0.0), Some(1.0)))
                .display(
                    DisplayMetadata::new()
                        .label("Team")
                        .order(1)
                        .control("segmented"),
                ),
        )
    }
}

duan::reality!(Faction);
