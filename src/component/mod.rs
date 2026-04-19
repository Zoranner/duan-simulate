//! 组件系统
//!
//! # 三元语义：认知、意图、事实
//!
//! 中文术语与 Rust trait 对应：**认知**（[`Belief`]）、**意图**（[`Intent`]）、**事实**（[`Reality`]）。
//! 与 README 术语表一致。所有实体数据按语义归入三类，写入权限由类型本身决定，编译期强制：
//!
//! | 术语（中文） | Rust trait | 实体     | 域       | Snapshot |
//! |-----------|-----------|--------|--------|---------------|
//! | 认知 | Belief | 读写     | 不可见   | 不可见          |
//! | 意图 | Intent | 读写     | 只读     | 只读            |
//! | 事实 | Reality | 只读快照 | 独占写入 | 只读            |
//!
//! # 用法
//!
//! 推荐使用便捷宏声明语义，宏会同时设置正确的 `ComponentKind`：
//!
//! ```rust,ignore
//! #[derive(Clone, Default)] pub struct SoldierBelief { pub path_index: usize }
//! #[derive(Clone, Default)] pub struct MovementOrder { pub target_x: f64, pub target_y: f64 }
//! #[derive(Clone, Default)] pub struct Position { pub x: f64, pub y: f64 }
//!
//! duan::belief!(SoldierBelief);          // 认知：ComponentKind::Belief，不进入快照
//! duan::intent!(MovementOrder);          // 意图：ComponentKind::Intent，进入快照
//! duan::reality!(Position, Velocity);    // 事实：ComponentKind::Reality，进入快照
//! ```
//!
//! 需要手写 `impl` 时，必须显式覆盖 `const KIND`，否则默认为 `Reality`：
//!
//! ```rust,ignore
//! use duan::{Component, ComponentKind, EntityWritable, Belief};
//!
//! #[derive(Clone, Default)]
//! pub struct SoldierBelief { pub path_index: usize }
//!
//! impl Component for SoldierBelief {
//!     const KIND: ComponentKind = ComponentKind::Belief; // 必须显式指定，否则默认 Reality
//! }
//! impl EntityWritable for SoldierBelief {}
//! impl Belief for SoldierBelief {}
//! ```

use std::any::TypeId;

/// 组件语义分类
///
/// 用于描述组件在仿真中的可见性与写入语义。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComponentKind {
    /// 实体私有认知数据，不进入快照
    Belief,
    /// 实体公开意图，进入快照
    Intent,
    /// 域权威事实，进入快照
    Reality,
}

/// 实体组件统一约束（sealed supertrait）
///
/// 所有实体附加数据的基础约束。用户不直接实现此 trait，
/// 而是通过实现 [`Belief`]、[`Intent`] 或 [`Reality`] 之一来声明语义。
///
/// 框架内部以 Component 为统一泛型约束，用户只需关注三个语义 trait。
pub trait Component: Send + Sync + Clone + 'static {
    /// 组件语义（默认视为事实）
    ///
    /// 默认值为 `Reality`，便于渐进迁移已有手写 `impl Component` 的代码。
    const KIND: ComponentKind = ComponentKind::Reality;
}

/// 实体可写标记
///
/// 只有 [`Belief`] 和 [`Intent`] 类型实现此 trait。
/// [`EntityContext::set`](crate::EntityContext::set) 使用此约束，在编译期阻止实体写入 [`Reality`] 类型。
pub trait EntityWritable: Component {}

/// 认知（`Belief`）
///
/// 实体内部认知数据，对外完全封闭：
/// - 实体 `tick()` 可读写
/// - 域 `compute()` 不能访问
/// - [`Snapshot`](crate::Snapshot) 中不包含
///
/// 适用于实体的内部决策数据，如路径规划缓存、有限状态机内部变量等。
pub trait Belief: EntityWritable {}

/// 意图（`Intent`）
///
/// 实体对外表达的意志与诉求（意图数据）：
/// - 实体 `tick()` 可写（当前帧）
/// - 域 `compute()` 可读（从快照，上帧值，只读）
/// - [`Snapshot`](crate::Snapshot) 中可见（只读）
///
/// 适用于实体希望驱动的行为，如移动命令、攻击意图等。
pub trait Intent: EntityWritable {}

/// 事实（`Reality`）
///
/// 由域权威写入的客观世界内容：
/// - 域 `compute()` 中声明 `Writes` 的域可独占写入
/// - 实体 `tick()` 可读（从快照，上帧值，只读）
/// - [`Snapshot`](crate::Snapshot) 中可见（只读）
///
/// 适用于由物理、战斗等域权威裁定的结果，如位置、速度、生命值等。
pub trait Reality: Component {}

/// 类型级组件集合
///
/// 用于 [`Domain`](crate::Domain) 的 `Writes` 和 `Reads` 关联类型。
/// 支持 `()` 和最多 12 元素的组件元组。
pub trait ComponentSet: 'static {
    /// 返回集合中所有组件的 TypeId（用于调度器分析）
    fn type_ids() -> Vec<TypeId>
    where
        Self: Sized;
}

impl ComponentSet for () {
    fn type_ids() -> Vec<TypeId> {
        vec![]
    }
}

/// 编译期集合成员检测（标记 trait）
///
/// 由于 stable Rust 不支持不重叠 blanket impl，`Contains<T>` 不自动为 tuple 实现。
/// 框架转而通过 [`ComponentSet::type_ids()`] 在运行期（构建时）验证集合成员关系。
///
/// 如需编译期检查，可为特定 tuple 手动实现此 trait（高级用法）。
pub trait Contains<T: Component>: ComponentSet {}

// ──── ComponentSet 的 tuple 展开（最多 12 元素）────────────────────────

macro_rules! impl_component_set {
    ($($T:ident),+) => {
        impl<$($T: Component),+> ComponentSet for ($($T,)+) {
            fn type_ids() -> Vec<TypeId> {
                vec![ $(TypeId::of::<$T>()),+ ]
            }
        }
    };
}

impl_component_set!(A);
impl_component_set!(A, B);
impl_component_set!(A, B, C);
impl_component_set!(A, B, C, D);
impl_component_set!(A, B, C, D, E);
impl_component_set!(A, B, C, D, E, F);
impl_component_set!(A, B, C, D, E, F, G);
impl_component_set!(A, B, C, D, E, F, G, H);
impl_component_set!(A, B, C, D, E, F, G, H, I);
impl_component_set!(A, B, C, D, E, F, G, H, I, J);
impl_component_set!(A, B, C, D, E, F, G, H, I, J, K);
impl_component_set!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_component_set!(A, B, C, D, E, F, G, H, I, J, K, L, M);

// ──── 便捷宏 ──────────────────────────────────────────────────────────────

/// 为类型声明认知语义（`Belief`）
///
/// 等价于依次 `impl Component`, `impl EntityWritable`, `impl Belief`。
///
/// # 用法
///
/// ```rust,ignore
/// duan::belief!(SoldierBelief);
/// duan::belief!(A, B, C);
/// ```
#[macro_export]
macro_rules! belief {
    ($($t:ty),+ $(,)?) => {
        $(impl $crate::Component for $t {
            const KIND: $crate::ComponentKind = $crate::ComponentKind::Belief;
        })*
        $(impl $crate::EntityWritable for $t {})*
        $(impl $crate::Belief for $t {})*
    };
}

/// 为类型声明意图语义（`Intent`）
///
/// 等价于依次 `impl Component`, `impl EntityWritable`, `impl Intent`。
#[macro_export]
macro_rules! intent {
    ($($t:ty),+ $(,)?) => {
        $(impl $crate::Component for $t {
            const KIND: $crate::ComponentKind = $crate::ComponentKind::Intent;
        })*
        $(impl $crate::EntityWritable for $t {})*
        $(impl $crate::Intent for $t {})*
    };
}

/// 为类型声明事实语义（`Reality`）
///
/// 等价于依次 `impl Component`, `impl Reality`。
#[macro_export]
macro_rules! reality {
    ($($t:ty),+ $(,)?) => {
        $(impl $crate::Component for $t {
            const KIND: $crate::ComponentKind = $crate::ComponentKind::Reality;
        })*
        $(impl $crate::Reality for $t {})*
    };
}
