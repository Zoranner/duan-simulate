use duan_package::{DisplayMetadata, Schema};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticBody;

impl StaticBody {
    pub const ITEM_ID: &str = concat!(env!("CARGO_PKG_NAME"), "/static-body");

    pub fn schema() -> Schema {
        Schema::new().field(
            "enabled",
            duan_package::FieldSchema::new(duan_package::PrimitiveKind::Bool)
                .default(duan_package::PrimitiveValue::Bool(true))
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
