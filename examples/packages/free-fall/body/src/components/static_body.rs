use duan_catalog::{DisplayMetadata, Schema};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticBody;

impl StaticBody {
    pub const ITEM_ID: &str = concat!(env!("CARGO_PKG_NAME"), "/static-body");

    pub fn schema() -> Schema {
        Schema::new().field(
            "enabled",
            duan_catalog::FieldSchema::new(duan_catalog::PrimitiveKind::Bool)
                .default(duan_catalog::PrimitiveValue::Bool(true))
                .display(
                    DisplayMetadata::new()
                        .label("Static")
                        .order(1)
                        .control("checkbox"),
                ),
        )
    }
}

duan::reality!(StaticBody);
