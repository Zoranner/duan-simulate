//! 域（Domain）
//!
//! 域是**状态**（`State`）数据的权威，是仿真的核心计算单元。
//!
//! # 设计哲学
//!
//! 「域是状态数据的权威。实体是意志的主体。」
//!
//! - 每种状态（`State`）类型只能由一个域独占写入（构建期冲突检测）
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
//! duan::state!(Velocity);
//!
//! pub struct MotionDomain { pub gravity: f64 }
//!
//! impl Domain for MotionDomain {
//!     type Writes = (Velocity,);
//!     type Reads = (Velocity,);
//!     type After = ();
//!
//!     fn compute(&mut self, ctx: &mut DomainContext<Self>, dt: f64) {
//!         let updates: Vec<_> = ctx
//!             .each::<Velocity>()
//!             .map(|(id, v)| (id, v.clone()))
//!             .collect();
//!         for (id, v) in updates {
//!             if let Some(vel) = ctx.get_mut::<Velocity>(id) {
//!                 vel.vy -= self.gravity * dt;
//!             }
//!         }
//!     }
//! }
//! ```

pub mod context;

use crate::component::ComponentSet;
use std::any::TypeId;

// ──── ComputeResources ────────────────────────────────────────────────────

/// 域计算所需资源（内部传参包）
///
/// 将 `compute_dyn` 所需的多个可变引用打包，避免参数过多。
pub(crate) struct ComputeResources<'a> {
    pub storage: &'a mut crate::component::storage::WorldStorage,
    pub snapshot: &'a crate::snapshot::WorldSnapshot,
    pub pending_spawns: &'a mut Vec<crate::entity::PendingSpawn>,
    pub pending_destroys: &'a mut Vec<crate::entity::id::EntityId>,
    pub events: &'a mut crate::events::EventBuffer,
    pub clock: &'a crate::time::TimeClock,
    pub logger: &'a crate::logging::LoggerHandle,
    pub dt: f64,
}

// ──── DomainSet ───────────────────────────────────────────────────────────

/// 类型级域集合
///
/// 用于 [`Domain::After`] 关联类型，声明依赖的前置域。
pub trait DomainSet: 'static {
    /// 返回集合中所有域的 TypeId（用于调度器构建 DAG）
    fn type_ids() -> Vec<TypeId>
    where
        Self: Sized;
}

impl DomainSet for () {
    fn type_ids() -> Vec<TypeId> {
        vec![]
    }
}

macro_rules! impl_domain_set {
    ($($D:ident),+) => {
        impl<$($D: Domain),+> DomainSet for ($($D,)+) {
            fn type_ids() -> Vec<TypeId> {
                vec![ $(TypeId::of::<$D>()),+ ]
            }
        }
    };
}

impl_domain_set!(D1);
impl_domain_set!(D1, D2);
impl_domain_set!(D1, D2, D3);
impl_domain_set!(D1, D2, D3, D4);
impl_domain_set!(D1, D2, D3, D4, D5);
impl_domain_set!(D1, D2, D3, D4, D5, D6);
impl_domain_set!(D1, D2, D3, D4, D5, D6, D7);
impl_domain_set!(D1, D2, D3, D4, D5, D6, D7, D8);

// ──── Domain trait ────────────────────────────────────────────────────────

/// 域 trait
///
/// 通过三个关联类型声明数据所有权和执行依赖，框架在 `build()` 时静态分析：
///
/// - `Writes`：独占写入的 State 类型集合（同一 State 只能有一个域写入）
/// - `Reads`：从世界快照读取的 Intent/State 类型集合
/// - `After`：必须在本域之前完成计算的域集合
pub trait Domain: Send + Sync + Sized + 'static {
    /// 本域独占写入的 State 类型集合
    type Writes: ComponentSet;
    /// 本域从快照读取的 Intent/State 类型集合
    type Reads: ComponentSet;
    /// 必须在本域之前完成的前置域集合
    type After: DomainSet;

    /// 每帧计算（Phase 3 执行）
    ///
    /// 通过 [`DomainContext`](context::DomainContext) 读写组件数据和发送事件。
    fn compute(&mut self, ctx: &mut context::DomainContext<Self>, dt: f64);

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
pub trait InWrites<D: Domain>: crate::component::Component {}

/// 读取约束标记（可选的编译期检查）
pub trait InReads<D: Domain>: crate::component::Component {}

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
        let dt = res.dt;
        let mut ctx = context::DomainContext {
            storage: res.storage,
            snapshot: res.snapshot,
            pending_spawns: res.pending_spawns,
            pending_destroys: res.pending_destroys,
            events: res.events,
            clock: res.clock,
            logger: res.logger,
            dt,
            _phantom: std::marker::PhantomData,
        };
        self.compute(&mut ctx, dt);
    }
}
