//! 质量组件
//!
//! 描述实体的质量属性。

use duan::impl_component;

/// 质量组件
///
/// 存储实体的质量值。
#[derive(Debug, Clone)]
pub struct Mass {
    pub value: f64,
}

impl Mass {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}

impl_component!(Mass, "mass");
