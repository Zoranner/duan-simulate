//! 位置组件
//!
//! 描述实体在三维空间中的位置。

use duan::impl_component;

/// 位置组件
///
/// 存储实体在三维空间中的坐标。
#[derive(Debug, Clone)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Position {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

impl_component!(Position, "position");
