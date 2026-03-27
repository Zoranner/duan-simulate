use duan::{CustomEvent, EntityId};
use std::any::Any;

/// 开火事件：舰船向目标发射导弹
pub struct FireEvent {
    pub shooter_id: EntityId,
    pub target_id: EntityId,
    /// 发射位置
    pub launch_x: f64,
    pub launch_y: f64,
    /// 朝向目标的方向向量（未归一化）
    pub dir_x: f64,
    pub dir_y: f64,
    pub missile_speed: f64,
    /// 导弹最大飞行距离（超出后自毁）
    pub missile_range: f64,
    pub damage: f64,
}

impl CustomEvent for FireEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn event_name(&self) -> &str {
        "fire"
    }
}
