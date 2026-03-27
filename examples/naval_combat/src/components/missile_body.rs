use duan::impl_component;

/// 导弹标记组件
///
/// 标识此实体为导弹，碰撞域通过此标记区分导弹和舰船。
#[derive(Debug, Clone, Copy)]
pub struct MissileBody;

impl_component!(MissileBody, "missile_body");
