#[derive(duan::Component, Debug, Clone, PartialEq)]
#[component(id = "weapon", kind = "reality", label = "Weapon")]
pub struct Weapon {
    #[field(label = "Range", default = 180.0, range = 0.0..=10000.0, unit = "m", control = "number", order = 1)]
    pub range: f64,
    #[field(label = "Damage", default = 25.0, range = 0.0..=10000.0, unit = "hp", control = "number", order = 2)]
    pub damage: f64,
    #[field(label = "Missile Speed", default = 80.0, range = 0.0..=10000.0, unit = "m/s", control = "number", order = 3)]
    pub missile_speed: f64,
    #[field(label = "Cooldown", default = 1.5, range = 0.0..=10000.0, unit = "s", control = "number", order = 4)]
    pub fire_cooldown: f64,
    #[field(label = "Cooldown Remaining", default = 0.0, range = 0.0..=10000.0, unit = "s", control = "number", order = 5)]
    pub cooldown_remaining: f64,
}

impl Weapon {
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
}
