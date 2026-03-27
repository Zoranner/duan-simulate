use duan::impl_component;

/// 武器组件
#[derive(Debug, Clone, Copy)]
pub struct Weapon {
    /// 武器射程
    pub range: f64,
    /// 导弹命中伤害
    pub damage: f64,
    /// 开火冷却时间（秒）
    pub fire_cooldown: f64,
    /// 导弹速度（m/s）
    pub missile_speed: f64,
}

impl Weapon {
    pub fn new(range: f64, damage: f64, fire_cooldown: f64, missile_speed: f64) -> Self {
        Self {
            range,
            damage,
            fire_cooldown,
            missile_speed,
        }
    }
}

impl_component!(Weapon, "weapon");
