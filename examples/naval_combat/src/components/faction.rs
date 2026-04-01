/// 阵营标记（0=红方，1=蓝方）
#[derive(Debug, Clone)]
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

duan::reality!(Faction);
