use duan::impl_component;

/// 舵控组件，赋予舰船转向能力。
///
/// `turn_rate` 为最大偏转角速度（弧度/秒）。
/// 导弹不挂载此组件（导弹通过 Seeker 直接瞄准目标）。
#[derive(Debug, Clone, Copy)]
pub struct Helm {
    pub turn_rate: f64,
}

impl Helm {
    pub fn new(turn_rate: f64) -> Self {
        Self { turn_rate }
    }
}

impl_component!(Helm, "helm");
