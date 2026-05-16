use duan::Component;

#[derive(Component, Debug, Clone, PartialEq)]
#[component(id = "health", kind = "reality", label = "Health")]
pub struct Health {
    #[field(label = "Current", default = 100.0, range = 0.0..=10000.0, control = "number", order = 1)]
    pub current: f64,
    #[field(label = "Max", default = 100.0, range = 0.0..=10000.0, control = "number", order = 2)]
    pub max: f64,
}

impl Health {
    pub fn new(max: f64) -> Self {
        Self { current: max, max }
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }
}
