use duan::Component;

#[derive(Component, Debug, Clone, Default, PartialEq)]
#[component(id = "position-2", kind = "reality", label = "Position")]
pub struct Position2 {
    #[field(label = "X", default = 0.0, range = -100000.0..=100000.0, unit = "m", control = "number", order = 1)]
    pub x: f64,
    #[field(label = "Y", default = 0.0, range = -100000.0..=100000.0, unit = "m", control = "number", order = 2)]
    pub y: f64,
}

impl Position2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}
