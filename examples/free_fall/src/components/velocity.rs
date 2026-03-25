//! 速度组件
//!
//! 描述实体在三维空间中的速度向量。

use duan::Component;
use std::any::Any;

/// 速度组件
///
/// 存储实体在三维空间中的速度分量。
#[derive(Debug, Clone)]
pub struct Velocity {
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
}

impl Velocity {
    pub fn new(vx: f64, vy: f64, vz: f64) -> Self {
        Self { vx, vy, vz }
    }
}

impl Component for Velocity {
    fn component_type(&self) -> &'static str {
        "velocity"
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
