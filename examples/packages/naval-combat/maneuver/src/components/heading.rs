use duan::Component;

#[derive(Component, Debug, Clone, Default, PartialEq)]
#[component(id = "heading", kind = "intent", label = "Heading")]
pub struct Heading {
    #[field(label = "Heading", default = 0.0, range = -std::f64::consts::PI..=std::f64::consts::PI, unit = "rad", control = "angle", order = 1)]
    pub radians: f64,
}
