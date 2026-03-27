use duan::impl_component;

/// 阵营组件
///
/// 0 = 红方，1 = 蓝方
#[derive(Debug, Clone, Copy)]
pub struct Faction {
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

impl_component!(Faction, "faction");
