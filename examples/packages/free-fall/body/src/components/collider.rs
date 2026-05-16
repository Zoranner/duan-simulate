#[derive(duan::Component, Debug, Clone, PartialEq)]
#[component(id = "collider", kind = "reality", label = "Collider")]
pub struct Collider {
    #[field(label = "Restitution", default = 0.8, range = 0.0..=1.0, unit = "ratio", control = "slider", order = 1)]
    pub restitution: f64,
}

impl Collider {
    pub fn new(restitution: f64) -> Self {
        Self { restitution }
    }
}
