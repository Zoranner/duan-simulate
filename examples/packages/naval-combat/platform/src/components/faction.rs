use duan::Component;

#[derive(Component, Debug, Clone, PartialEq, Eq)]
#[component(id = "faction", kind = "reality", label = "Faction")]
pub struct Faction {
    #[field(label = "Team", default = 0, range = 0.0..=1.0, control = "segmented", order = 1)]
    pub team: u8,
}

impl Faction {
    pub fn red() -> Self {
        Self { team: 0 }
    }

    pub fn blue() -> Self {
        Self { team: 1 }
    }
}
