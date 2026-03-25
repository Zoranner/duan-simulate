//! 质量组件
//!
//! 描述实体的质量属性。

use duan::Component;
use std::any::Any;

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

impl Component for Mass {
    fn component_type(&self) -> &'static str {
        "mass"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn into_any_boxed(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}
