/// 碰撞体参数（State：弹性系数和摩擦系数）
#[derive(Debug, Clone)]
pub struct Collider {
    /// 弹性系数（0.0 = 完全非弹性，1.0 = 完全弹性）
    pub restitution: f64,
    /// 摩擦系数
    pub friction: f64,
}

impl Collider {
    pub fn new(restitution: f64, friction: f64) -> Self {
        Self {
            restitution,
            friction,
        }
    }
}

duan::state!(Collider);
