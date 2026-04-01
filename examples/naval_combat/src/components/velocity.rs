/// 2D 速度
#[derive(Debug, Clone, Default)]
pub struct Velocity {
    pub vx: f64,
    pub vy: f64,
}

impl Velocity {
    pub fn new(vx: f64, vy: f64) -> Self {
        Self { vx, vy }
    }

    /// 朝指定方向以指定速率生成速度向量
    pub fn towards(dir_x: f64, dir_y: f64, speed: f64) -> Self {
        let len = (dir_x * dir_x + dir_y * dir_y).sqrt();
        if len < 1e-9 {
            return Self::default();
        }
        Self {
            vx: dir_x / len * speed,
            vy: dir_y / len * speed,
        }
    }
}

duan::reality!(Velocity);
