/// 质量（State：初始设定，域只读）
#[derive(Debug, Clone)]
pub struct Mass {
    pub kg: f64,
}

impl Mass {
    pub fn new(kg: f64) -> Self {
        Self { kg }
    }
}

duan::state!(Mass);
