/// 武器参数（含开火冷却计时）
#[derive(Debug, Clone)]
pub struct Weapon {
    pub range: f64,
    pub damage: f64,
    pub fire_cooldown: f64,
    pub missile_speed: f64,
    /// 剩余冷却时间（秒）
    pub cooldown_remaining: f64,
}

impl Weapon {
    pub fn new(range: f64, damage: f64, fire_cooldown: f64, missile_speed: f64) -> Self {
        Self {
            range,
            damage,
            fire_cooldown,
            missile_speed,
            cooldown_remaining: 0.0,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.cooldown_remaining <= 0.0
    }
}

duan::state!(Weapon);
