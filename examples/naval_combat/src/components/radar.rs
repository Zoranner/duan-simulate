use duan::impl_component;

/// 雷达组件
///
/// 赋予实体探测能力，域根据 range 判断探测半径。
#[derive(Debug, Clone, Copy)]
pub struct Radar {
    pub range: f64,
}

impl Radar {
    pub fn new(range: f64) -> Self {
        Self { range }
    }
}

impl_component!(Radar, "radar");
