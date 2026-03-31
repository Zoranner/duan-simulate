//! 注册器
//!
//! [`DomainRegistrar`] 是 [`crate::WorldBuilder::domains`] 闭包的入参，
//! 用于批量注册仿真域。

use crate::domain::{AnyDomain, Domain};

/// 域注册器
///
/// 通过 [`crate::WorldBuilder::domains`] 的闭包参数获取，用于批量注册仿真域。
///
/// # 示例
///
/// ```rust,ignore
/// World::builder()
///     .domains(|d| {
///         d.add(MotionDomain);
///         d.add(CollisionDomain);
///     })
///     .build();
/// ```
pub struct DomainRegistrar {
    pub(crate) domains: Vec<Box<dyn AnyDomain>>,
}

impl DomainRegistrar {
    pub(crate) fn new() -> Self {
        Self {
            domains: Vec::new(),
        }
    }

    /// 注册一个域
    pub fn add<D: Domain>(&mut self, domain: D) -> &mut Self {
        self.domains.push(Box::new(domain));
        self
    }
}
