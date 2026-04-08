use duan::EntityId;

/// 导弹寻的配置（固有属性，生成时确定，之后不再变化）
///
/// 属于**事实**（`Reality`）语义：由 `FireEvent` 的 Reaction 在生成导弹时写入，
/// 之后只有 `CollisionDomain` 读取，无域写入。
#[derive(Debug, Clone)]
pub struct SeekerConfig {
    pub target_id: EntityId,
    pub shooter_id: EntityId,
    pub damage: f64,
    /// 最大飞行距离（超出则自毁）
    pub max_range: f64,
}

impl SeekerConfig {
    pub fn new(target_id: EntityId, shooter_id: EntityId, damage: f64, max_range: f64) -> Self {
        Self {
            target_id,
            shooter_id,
            damage,
            max_range,
        }
    }
}

duan::reality!(SeekerConfig);

/// 导弹飞行状态（运动过程中累积，由 MotionDomain 权威写入）
///
/// 属于**事实**（`Reality`）语义：`MotionDomain` 每帧累积飞行距离，
/// `CollisionDomain` 读取以判断是否超出射程。
#[derive(Debug, Clone, Default)]
pub struct SeekerState {
    /// 已飞行距离
    pub traveled: f64,
}

duan::reality!(SeekerState);
