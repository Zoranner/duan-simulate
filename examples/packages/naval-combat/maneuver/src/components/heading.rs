use duan_macros::Component;

#[derive(Component, Debug, Clone, Default, PartialEq)]
#[component(id = "heading", kind = "intent", label = "Heading")]
pub struct Heading {
    #[field(label = "Heading", default = 0.0, range = -3.141592653589793..=3.141592653589793, unit = "rad", control = "angle", order = 1)]
    pub radians: f64,
}
