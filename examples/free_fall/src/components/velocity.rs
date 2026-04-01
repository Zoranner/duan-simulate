/// 速度（事实 Reality：由运动域积分）
#[derive(Debug, Clone, Default)]
pub struct Velocity {
    pub vx: f64,
    pub vy: f64,
}

impl Velocity {
    pub fn new(vx: f64, vy: f64) -> Self {
        Self { vx, vy }
    }
}

duan::reality!(Velocity);
