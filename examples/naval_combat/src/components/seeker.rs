use duan::{impl_component, EntityId};

/// 追踪组件
///
/// 赋予导弹追踪目标的能力，运动域据此计算转向。
#[derive(Debug, Clone, Copy)]
pub struct Seeker {
    pub target_id: EntityId,
    /// 射手 ID（用于 HitEvent 记录）
    pub shooter_id: EntityId,
    /// 命中伤害
    pub damage: f64,
    /// 最大飞行射程（超出后自毁）
    pub max_range: f64,
    /// 已飞行距离（由运动域每步累加）
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

impl_component!(Seeker, "seeker");
