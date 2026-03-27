//! 碰撞事件
//!
//! 当实体与表面发生碰撞时触发。

use duan::{CustomEvent, EntityId};
use std::any::Any;

/// 地面碰撞事件
///
/// 由碰撞域检测到实体触地时发出。
/// 框架通过 Arc 共享此事件，无需实现 Clone。
#[derive(Debug)]
pub struct GroundCollisionEvent {
    pub entity_id: EntityId,
    pub surface_name: String,
    /// 碰撞时的冲击速度大小（始终为正值）
    pub impact_velocity: f64,
    /// 弹性系数
    pub restitution: f64,
    /// 摩擦系数
    pub friction: f64,
}

impl GroundCollisionEvent {
    pub fn new(
        entity_id: EntityId,
        surface_name: impl Into<String>,
        impact_velocity: f64,
        restitution: f64,
        friction: f64,
    ) -> Self {
        Self {
            entity_id,
            surface_name: surface_name.into(),
            impact_velocity,
            restitution,
            friction,
        }
    }
}

impl CustomEvent for GroundCollisionEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn event_name(&self) -> &str {
        "ground_collision"
    }
}
