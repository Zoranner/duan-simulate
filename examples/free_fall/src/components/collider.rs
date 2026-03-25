//! 碰撞体组件
//!
//! 描述实体的碰撞属性。与 Position 组件配合使用，
//! 可同时挂载在动态物体（如小球）和静态表面（如地面）上。

use duan::Component;
use std::any::Any;

/// 碰撞体组件
///
/// 描述实体的碰撞表面参数。与 Position 组件配合：
/// - 有 Position + Collider + Velocity：动态碰撞体（运动 + 碰撞检测）
/// - 有 Position + Collider（无 Velocity）：静态碰撞体（静止表面）
///
/// 碰撞域通过此组件识别可碰撞的实体。
#[derive(Debug, Clone)]
pub struct Collider {
    /// 碰撞体名称
    pub name: String,
    /// 相对于位置组件的高度偏移（用于地面等水平表面）
    pub offset_y: f64,
    /// 弹性系数（碰撞时的能量保留比例，0=完全非弹性，1=完全弹性）
    pub restitution: f64,
    /// 摩擦系数（影响水平速度的衰减）
    pub friction: f64,
}

impl Collider {
    pub fn new(name: impl Into<String>, offset_y: f64, restitution: f64, friction: f64) -> Self {
        Self {
            name: name.into(),
            offset_y,
            restitution,
            friction,
        }
    }

    /// 创建地面碰撞体
    pub fn ground(restitution: f64, friction: f64) -> Self {
        Self::new("地面", 0.0, restitution, friction)
    }
}

impl Component for Collider {
    fn component_type(&self) -> &'static str {
        "collider"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn into_any_boxed(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}
