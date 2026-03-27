//! 静态碰撞体标记组件
//!
//! 零大小标记组件，显式声明实体是静态碰撞体（不受物理运动影响的表面）。
//! 碰撞域通过此组件识别地面等静止表面，而不依赖"缺少某个组件"的隐式推断。

use duan::impl_component;

/// 静态碰撞体标记
///
/// 挂载此组件的实体将被碰撞域识别为静态表面。
/// 不需要任何字段——标记组件的存在本身就是语义声明。
#[derive(Debug, Clone)]
pub struct StaticBody;

impl_component!(StaticBody, "static_body");
