#[derive(duan::Component, Debug, Clone, PartialEq, Eq)]
#[component(id = "static-body", kind = "reality", label = "Static Body")]
pub struct StaticBody {
    #[field(label = "Static", default = true, control = "checkbox", order = 1)]
    pub enabled: bool,
}

impl StaticBody {
    pub fn enabled() -> Self {
        Self { enabled: true }
    }
}
