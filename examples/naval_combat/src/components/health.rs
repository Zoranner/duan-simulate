use duan::impl_component;

#[derive(Debug, Clone, Copy)]
pub struct Health {
    pub current: f64,
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

impl_component!(Health, "health");
