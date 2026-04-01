//! 实体（Entity）
//!
//! 实体是仿真中的意志主体，是行为逻辑的载体。
//!
//! # 设计哲学
//!
//! 实体是零大小标记类型（ZST），通过 trait 定义行为而非字段存储数据。
//! 数据由框架按组件类型密集存储，实体通过 [`EntityContext`](crate::entity::context::EntityContext) 访问自身数据。
//!
//! # 示例
//!
//! ```rust,ignore
//! use duan::{Entity, EntityContext, Component, Reality};
//!
//! #[derive(Clone, Default)]
//! pub struct Health { pub current: f64, pub max: f64 }
//! duan::reality!(Health);
//!
//! pub struct Soldier;
//!
//! impl Entity for Soldier {
//!     fn tick(ctx: &mut EntityContext) {
//!         // 从快照读取生命值（事实 Reality：上帧值）
//!         if let Some(hp) = ctx.snapshot().get::<Health>(ctx.id()) {
//!             if hp.current <= 0.0 {
//!                 ctx.destroy(ctx.id());
//!             }
//!         }
//!     }
//!
//!     fn bundle() -> impl ComponentBundle {
//!         (Health { current: 100.0, max: 100.0 },)
//!     }
//! }
//! ```

pub mod context;
pub mod id;

use crate::storage::Storage;
use crate::Component;
use id::EntityId;

// ──── Entity trait ───────────────────────────────────────────────────────

/// 实体 trait
///
/// 实体是仿真中的意志主体。通过实现此 trait 定义实体的行为逻辑和初始数据。
///
/// - [`tick`](Entity::tick)：每帧行为逻辑，通过 [`EntityContext`](context::EntityContext) 访问数据
/// - [`bundle`](Entity::bundle)：实体生成时的初始组件包
pub trait Entity: 'static {
    /// 每帧行为逻辑
    ///
    /// 在 Phase 2（Entity Tick）阶段调用，晚于快照冻结、早于域计算。
    /// 默认实现为空操作（无行为实体）。
    fn tick(_ctx: &mut context::EntityContext) {}

    /// 实体初始组件包
    ///
    /// 在 `World::spawn::<E>()` 时调用一次，返回的组件被写入世界存储。
    /// 默认返回空包。
    fn bundle() -> impl ComponentBundle + Send + 'static
    where
        Self: Sized,
    {
    }
}

// ──── ComponentBundle ────────────────────────────────────────────────────

/// 组件包 trait
///
/// 将一组组件应用到世界存储中。支持 `()` 和 1-12 元素的组件元组。
pub trait ComponentBundle {
    fn apply(self, id: EntityId, storage: &mut Storage);
}

impl ComponentBundle for () {
    fn apply(self, _id: EntityId, _storage: &mut Storage) {}
}

macro_rules! impl_component_bundle {
    ($($T:ident: $idx:tt),+) => {
        impl<$($T: Component),+> ComponentBundle for ($($T,)+) {
            fn apply(self, id: EntityId, storage: &mut Storage) {
                $(storage.insert::<$T>(id, self.$idx);)+
            }
        }
    };
}

impl_component_bundle!(A: 0);
impl_component_bundle!(A: 0, B: 1);
impl_component_bundle!(A: 0, B: 1, C: 2);
impl_component_bundle!(A: 0, B: 1, C: 2, D: 3);
impl_component_bundle!(A: 0, B: 1, C: 2, D: 3, E: 4);
impl_component_bundle!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5);
impl_component_bundle!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6);
impl_component_bundle!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7);
impl_component_bundle!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8);
impl_component_bundle!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9);
impl_component_bundle!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10);
impl_component_bundle!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11);

// ──── 实体生命周期状态 ────────────────────────────────────────────────────

/// 实体生命周期状态
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Lifecycle {
    /// 活跃，正常参与仿真
    #[default]
    Active,
    /// 销毁中（已从域脱离，等待清理过渡期结束）
    Destroying,
    /// 已销毁，待从存储移除
    Destroyed,
}

impl Lifecycle {
    pub fn is_active(self) -> bool {
        matches!(self, Lifecycle::Active)
    }

    pub fn is_alive(self) -> bool {
        !matches!(self, Lifecycle::Destroyed)
    }
}

// ──── 内部：实体注册记录 ──────────────────────────────────────────────────

/// 实体注册记录（框架内部使用）
pub(crate) struct EntityRecord {
    pub id: EntityId,
    pub lifecycle: Lifecycle,
    /// 实体类型 tick 函数（通过 dispatch_tick::<E> 保存）
    pub tick_fn: fn(&mut context::EntityContext),
}

/// 泛型 tick 分发函数，将 Entity 类型的 tick 方法适配为函数指针
pub(crate) fn dispatch_tick<E: Entity>(ctx: &mut context::EntityContext) {
    E::tick(ctx);
}

// ──── 待处理操作 ──────────────────────────────────────────────────────────

type ApplyFn = Box<dyn FnOnce(EntityId, &mut Storage) + Send>;

/// 待 spawn 的实体（框架内部）
pub(crate) struct PendingSpawn {
    /// 将组件应用到存储的函数
    pub apply_fn: ApplyFn,
    /// tick 函数
    pub tick_fn: fn(&mut context::EntityContext),
}

impl PendingSpawn {
    pub fn new<E: Entity>(bundle: impl ComponentBundle + Send + 'static) -> Self {
        Self {
            apply_fn: Box::new(move |id, storage| bundle.apply(id, storage)),
            tick_fn: dispatch_tick::<E>,
        }
    }
}
