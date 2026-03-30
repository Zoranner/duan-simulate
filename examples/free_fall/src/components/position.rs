/// 位置（State：由域写入，实体只读）
#[derive(Debug, Clone, Default)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

impl Position {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

duan::state!(Position);
