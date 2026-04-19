//! 域（Domain）
//!
//! 域是**事实**（`Reality`）组件的权威，是仿真的核心计算单元（README：域 / Domain）。
//!
//! # 设计哲学
//!
//! 「域裁定事实。实体表达意图。」（与 README 一致。）
//!
//! - 每种事实（`Reality`）类型只能由一个域独占写入（构建期冲突检测）
//! - 域的计算依赖由类型系统而非字符串声明，Build 时静态分析
//! - 域读取世界快照（上帧值），写入当前帧活跃存储
//!
//! # 示例
//!
//! ```rust,ignore
//! use duan::{Domain, DomainContext};
//!
//! #[derive(Clone, Default)]
//! pub struct Velocity { pub vx: f64, pub vy: f64 }
//! duan::reality!(Velocity);
//!
//! pub struct MotionDomain { pub gravity: f64 }
//!
//! impl Domain for MotionDomain {
//!     type Writes = duan::component_set!(Velocity);
//!     type Reads = duan::component_set!(Velocity);
//!     type After = duan::domain_set!();
//!
//!     fn compute(&mut self, ctx: &mut DomainContext<Self>, delta_time: f64) {
//!         let updates: Vec<_> = ctx
//!             .each::<Velocity>()
//!             .map(|(id, v)| (id, v.clone()))
//!             .collect();
//!         for (id, v) in updates {
//!             if let Some(vel) = ctx.get_mut::<Velocity>(id) {
//!                 vel.vy -= self.gravity * delta_time;
//!             }
//!         }
//!     }
//! }
//! ```

pub mod context;

use crate::type_set::{TypeSet, TypeSetCons, TypeSetEnd};
use crate::ComponentSet;
use std::any::TypeId;

// ──── ComputeResources ────────────────────────────────────────────────────

/// 域计算所需资源（内部传参包）
///
/// 将 `compute_dyn` 所需的多个可变引用打包，避免参数过多。
pub(crate) struct ComputeResources<'a> {
    pub storage: &'a mut crate::storage::Storage,
    pub snapshot: &'a crate::snapshot::Snapshot,
    pub pending_spawns: &'a mut Vec<crate::entity::PendingSpawn>,
    pub pending_destroys: &'a mut Vec<crate::entity::id::EntityId>,
    pub events: &'a mut crate::event::EventBuffer,
    pub clock: &'a crate::runtime::timers::TimeClock,
    pub logger: &'a crate::diagnostics::LoggerHandle,
    pub delta_time: f64,
}

// ──── DomainSet ───────────────────────────────────────────────────────────

/// 类型级域集合
///
/// 用于 [`Domain::After`] 关联类型，声明依赖的前置域。
/// 推荐通过 [`domain_set!`](crate::domain_set) 宏构造，避免 tuple 方案的元素数量上限。
pub trait DomainSet: TypeSet {}

/// 域集合的递归边界约束
pub trait DomainSetBound: TypeSet {}

impl DomainSetBound for TypeSetEnd {}

impl<Head: Domain, Tail> DomainSetBound for TypeSetCons<Head, Tail> where
    Tail: DomainSetBound + TypeSet
{
}

impl<T> DomainSet for T where T: TypeSet + DomainSetBound {}

/// 构造无上限的类型级域集合
///
/// # 用法
///
/// ```rust,ignore
/// type After = duan::domain_set!(MotionDomain, CombatDomain);
/// type NoDeps = duan::domain_set!();
/// ```
#[macro_export]
macro_rules! domain_set {
    () => {
        $crate::type_set::TypeSetEnd
    };
    ($head:ty $(, $tail:ty)* $(,)?) => {
        $crate::type_set::TypeSetCons<$head, $crate::domain_set!($($tail),*)>
    };
}

// ──── Domain trait ────────────────────────────────────────────────────────

/// 域 trait
///
/// 通过三个关联类型声明数据所有权和执行依赖，框架在 `build()` 时静态分析：
///
/// - `Writes`：独占写入的 Reality 类型集合（同一 Reality 只能有一个域写入）
/// - `Reads`：从世界快照读取的 Intent/Reality 类型集合
/// - `After`：必须在本域之前完成计算的域集合
pub trait Domain: Send + Sync + Sized + 'static {
    /// 本域独占写入的 Reality 类型集合
    type Writes: ComponentSet;
    /// 本域从快照读取的 Intent/Reality 类型集合
    type Reads: ComponentSet;
    /// 必须在本域之前完成的前置域集合
    type After: DomainSet;

    /// 每帧计算（Phase 3 执行）
    ///
    /// 通过 [`DomainContext`](context::DomainContext) 读写组件数据和发送事件。
    fn compute(&mut self, ctx: &mut context::DomainContext<Self>, delta_time: f64);

    // ──── 框架内部调度信息接口 ─────────────────────────────────────────

    /// 返回 Writes 集合的 TypeId 列表（框架内部使用）
    fn writes_type_ids(&self) -> Vec<TypeId>
    where
        Self: Sized,
    {
        Self::Writes::type_ids()
    }

    /// 返回 After 集合的 TypeId 列表（框架内部使用）
    fn after_type_ids(&self) -> Vec<TypeId>
    where
        Self: Sized,
    {
        Self::After::type_ids()
    }
}

// ──── 访问约束标记 trait（供文档和高级手动实现使用）────────────────────

/// 写入约束标记（可选的编译期检查）
///
/// 框架通过调度器在构建期验证写入冲突；此 trait 可用于手动约束高级场景。
/// 对于常规域开发，无需关注此 trait。
pub trait InWrites<D: Domain>: crate::Component {}

/// 读取约束标记（可选的编译期检查）
pub trait InReads<D: Domain>: crate::Component {}

// ──── 类型擦除域接口（供 World 内部使用）─────────────────────────────────

/// 类型擦除的域接口（框架内部使用）
pub(crate) trait AnyDomain: Send + Sync {
    fn get_type_id(&self) -> TypeId;
    fn writes_type_ids(&self) -> Vec<TypeId>;
    fn after_type_ids(&self) -> Vec<TypeId>;
    fn compute_dyn(&mut self, res: ComputeResources<'_>);
}

impl<D: Domain> AnyDomain for D {
    fn get_type_id(&self) -> TypeId {
        TypeId::of::<D>()
    }

    fn writes_type_ids(&self) -> Vec<TypeId> {
        D::Writes::type_ids()
    }

    fn after_type_ids(&self) -> Vec<TypeId> {
        D::After::type_ids()
    }

    fn compute_dyn(&mut self, res: ComputeResources<'_>) {
        let delta_time = res.delta_time;
        let mut ctx = context::DomainContext {
            storage: res.storage,
            snapshot: res.snapshot,
            pending_spawns: res.pending_spawns,
            pending_destroys: res.pending_destroys,
            events: res.events,
            clock: res.clock,
            logger: res.logger,
            delta_time,
            _phantom: std::marker::PhantomData,
        };
        self.compute(&mut ctx, delta_time);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! define_component {
        ($name:ident) => {
            #[derive(Clone)]
            struct $name;
            impl crate::Component for $name {}
            impl crate::Reality for $name {}
        };
    }

    define_component!(W1);
    define_component!(W2);
    define_component!(W3);
    define_component!(W4);
    define_component!(W5);
    define_component!(W6);
    define_component!(W7);
    define_component!(W8);
    define_component!(W9);
    define_component!(W10);

    macro_rules! define_domain {
        ($domain:ident, $write:ident, $after:ty) => {
            struct $domain;
            impl Domain for $domain {
                type Writes = crate::component_set!($write);
                type Reads = crate::component_set!();
                type After = $after;

                fn compute(&mut self, _ctx: &mut context::DomainContext<Self>, _delta_time: f64) {}
            }
        };
    }

    define_domain!(D1, W1, crate::domain_set!());
    define_domain!(D2, W2, crate::domain_set!(D1));
    define_domain!(D3, W3, crate::domain_set!(D1, D2));
    define_domain!(D4, W4, crate::domain_set!(D1, D2, D3));
    define_domain!(D5, W5, crate::domain_set!(D1, D2, D3, D4));
    define_domain!(D6, W6, crate::domain_set!(D1, D2, D3, D4, D5));
    define_domain!(D7, W7, crate::domain_set!(D1, D2, D3, D4, D5, D6));
    define_domain!(D8, W8, crate::domain_set!(D1, D2, D3, D4, D5, D6, D7));
    define_domain!(D9, W9, crate::domain_set!(D1, D2, D3, D4, D5, D6, D7, D8));
    define_domain!(
        D10,
        W10,
        crate::domain_set!(D1, D2, D3, D4, D5, D6, D7, D8, D9)
    );

    #[test]
    fn test_domain_set_macro_supports_large_lists() {
        let ids =
            <crate::domain_set!(D1, D2, D3, D4, D5, D6, D7, D8, D9, D10) as crate::type_set::TypeSet>::type_ids();

        assert_eq!(ids.len(), 10);
    }
}
