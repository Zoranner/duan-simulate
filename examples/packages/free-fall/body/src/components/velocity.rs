use duan::Component;

#[derive(Component, Debug, Clone, Default, PartialEq)]
#[component(id = "velocity-2", kind = "reality", label = "Velocity")]
pub struct Velocity2 {
    #[field(label = "VX", default = 0.0, range = -1000.0..=1000.0, unit = "m/s", control = "number", order = 1)]
    pub vx: f64,
    #[field(label = "VY", default = 0.0, range = -1000.0..=1000.0, unit = "m/s", control = "number", order = 2)]
    pub vy: f64,
}

impl Velocity2 {
    pub fn new(vx: f64, vy: f64) -> Self {
        Self { vx, vy }
    }
}
