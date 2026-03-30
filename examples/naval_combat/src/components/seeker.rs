use duan::EntityId;

/// 导弹寻的参数
#[derive(Debug, Clone)]
pub struct Seeker {
    pub target_id: EntityId,
    pub shooter_id: EntityId,
    pub damage: f64,
    /// 最大飞行距离（超出则自毁）
    pub max_range: f64,
    /// 已飞行距离
    pub traveled: f64,
}

impl Seeker {
    pub fn new(target_id: EntityId, shooter_id: EntityId, damage: f64, max_range: f64) -> Self {
        Self {
            target_id,
            shooter_id,
            damage,
            max_range,
            traveled: 0.0,
        }
    }
}

duan::state!(Seeker);
