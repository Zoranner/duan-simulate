use duan_catalog::{
    DisplayMetadata, FieldSchema, PrimitiveKind, PrimitiveValue, Range, Schema, Unit,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Weapon {
    pub range: f64,
    pub damage: f64,
    pub missile_speed: f64,
    pub fire_cooldown: f64,
    pub cooldown_remaining: f64,
}

impl Weapon {
    pub const ITEM_ID: &str = concat!(env!("CARGO_PKG_NAME"), "/weapon");

    pub fn new(range: f64, damage: f64, missile_speed: f64, fire_cooldown: f64) -> Self {
        Self {
            range,
            damage,
            missile_speed,
            fire_cooldown,
            cooldown_remaining: 0.0,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.cooldown_remaining <= 0.0
    }

    pub fn schema() -> Schema {
        Schema::new()
            .field("range", metric_field("Range", 180.0, "m", 1))
            .field("damage", metric_field("Damage", 25.0, "hp", 2))
            .field(
                "missile_speed",
                metric_field("Missile speed", 80.0, "m/s", 3),
            )
            .field("fire_cooldown", metric_field("Cooldown", 1.5, "s", 4))
    }
}

fn metric_field(label: impl Into<String>, default: f64, unit: &str, order: i32) -> FieldSchema {
    FieldSchema::new(PrimitiveKind::Float)
        .default(PrimitiveValue::Float(default))
        .range(Range::new(Some(0.0), Some(10_000.0)))
        .unit(Unit::new(unit))
        .display(
            DisplayMetadata::new()
                .label(label)
                .order(order)
                .control("number"),
        )
}

duan::reality!(Weapon);
