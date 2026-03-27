use duan::impl_component;

#[derive(Debug, Clone, Copy)]
pub struct Velocity {
    pub vx: f64,
    pub vy: f64,
}

impl Velocity {
    pub fn new(vx: f64, vy: f64) -> Self {
        Self { vx, vy }
    }

    /// 以给定方向和速率构造速度向量
    pub fn towards(dir_x: f64, dir_y: f64, speed: f64) -> Self {
        let len = (dir_x * dir_x + dir_y * dir_y).sqrt();
        if len < 1e-10 {
            return Self::new(0.0, speed);
        }
        Self::new(dir_x / len * speed, dir_y / len * speed)
    }

    pub fn speed(&self) -> f64 {
        (self.vx * self.vx + self.vy * self.vy).sqrt()
    }
}

impl_component!(Velocity, "velocity");
