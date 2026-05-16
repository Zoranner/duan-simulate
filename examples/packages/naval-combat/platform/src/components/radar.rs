use duan::Component;

#[derive(Component, Debug, Clone, PartialEq)]
#[component(id = "radar", kind = "reality", label = "Radar")]
pub struct Radar {
    #[field(label = "Range", default = 260.0, range = 0.0..=10000.0, unit = "m", control = "number", order = 1)]
    pub range: f64,
}

impl Radar {
    pub fn new(range: f64) -> Self {
        Self { range }
    }
}
