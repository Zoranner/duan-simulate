/// 雷达探测范围
#[derive(Debug, Clone)]
pub struct Radar {
    pub range: f64,
}

impl Radar {
    pub fn new(range: f64) -> Self {
        Self { range }
    }
}

duan::reality!(Radar);
