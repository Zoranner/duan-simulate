//! 组件系统
//!
//! # 三元语义：认知、意图、状态
//!
//! 中文术语与 Rust trait 对应：**认知**（[`Memory`]）、**意图**（[`Intent`]）、**状态**（[`State`]）。
//! 所有实体数据按语义归入三类，写入权限由类型本身决定，编译期强制：
//!
//! | 术语（中文） | Rust trait | 实体     | 域       | WorldSnapshot |
//! |-----------|-----------|--------|--------|---------------|
//! | 认知 | Memory | 读写     | 不可见   | 不可见          |
//! | 意图 | Intent | 读写     | 只读     | 只读            |
//! | 状态 | State  | 只读快照 | 独占写入 | 只读            |
//!
//! # 用法
//!
//! ```rust,ignore
//! use duan::{Component, EntityWritable, Memory, Intent, State};
//!
//! #[derive(Clone, Default)]
//! pub struct SoldierMemory { pub path_index: usize }
//! impl Component for SoldierMemory {}
//! impl EntityWritable for SoldierMemory {}
//! impl Memory for SoldierMemory {}
//!
//! #[derive(Clone, Default)]
//! pub struct MovementOrder { pub target_x: f64, pub target_y: f64 }
//! impl Component for MovementOrder {}
//! impl EntityWritable for MovementOrder {}
//! impl Intent for MovementOrder {}
//!
//! #[derive(Clone, Default)]
//! pub struct Position { pub x: f64, pub y: f64 }
//! impl Component for Position {}
//! impl State for Position {}
//! ```
//!
//! 或使用便捷宏简化样板：
//!
//! ```rust,ignore
//! duan::memory!(SoldierMemory);
//! duan::intent!(MovementOrder);
//! duan::state!(Position, Velocity, Health);
//! ```

pub(crate) mod storage;

use std::any::TypeId;

/// 实体组件统一约束（sealed supertrait）
///
/// 所有实体附加数据的基础约束。用户不直接实现此 trait，
/// 而是通过实现 [`Memory`]、[`Intent`] 或 [`State`] 之一来声明语义。
///
/// 框架内部以 Component 为统一泛型约束，用户只需关注三个语义 trait。
pub trait Component: Send + Sync + Clone + 'static {}

/// 实体可写标记
///
/// 只有 [`Memory`] 和 [`Intent`] 类型实现此 trait。
/// [`EntityContext::set`](crate::EntityContext::set) 使用此约束，在编译期阻止实体写入 [`State`] 类型。
pub trait EntityWritable: Component {}

/// 认知（`Memory`）
///
/// 实体内部认知数据，对外完全封闭：
/// - 实体 `tick()` 可读写
/// - 域 `compute()` 不能访问
/// - [`WorldSnapshot`](crate::WorldSnapshot) 中不包含
///
/// 适用于实体的内部决策状态，如路径规划缓存、有限状态机状态等。
pub trait Memory: EntityWritable {}

/// 意图（`Intent`）
///
/// 实体对外表达的意志与诉求（意图数据）：
/// - 实体 `tick()` 可写（当前帧）
/// - 域 `compute()` 可读（从快照，上帧值，只读）
/// - [`WorldSnapshot`](crate::WorldSnapshot) 中可见（只读）
///
/// 适用于实体希望驱动的行为，如移动命令、攻击意图等。
pub trait Intent: EntityWritable {}

/// 状态（`State`）
///
/// 由域权威写入的客观状态：
/// - 域 `compute()` 中声明 `Writes` 的域可独占写入
/// - 实体 `tick()` 可读（从快照，上帧值，只读）
/// - [`WorldSnapshot`](crate::WorldSnapshot) 中可见（只读）
///
/// 适用于由物理、战斗等域权威计算的结果，如位置、速度、生命值等。
pub trait State: Component {}

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

// ──── 便捷宏 ──────────────────────────────────────────────────────────────

/// 为类型声明认知语义（`Memory`）
///
/// 等价于依次 `impl Component`, `impl EntityWritable`, `impl Memory`。
///
/// # 用法
///
/// ```rust,ignore
/// duan::memory!(SoldierMemory);
/// duan::memory!(A, B, C);
/// ```
#[macro_export]
macro_rules! memory {
    ($($t:ty),+ $(,)?) => {
        $(impl $crate::Component for $t {})*
        $(impl $crate::EntityWritable for $t {})*
        $(impl $crate::Memory for $t {})*
    };
}

/// 为类型声明意图语义（`Intent`）
///
/// 等价于依次 `impl Component`, `impl EntityWritable`, `impl Intent`。
#[macro_export]
macro_rules! intent {
    ($($t:ty),+ $(,)?) => {
        $(impl $crate::Component for $t {})*
        $(impl $crate::EntityWritable for $t {})*
        $(impl $crate::Intent for $t {})*
    };
}

/// 为类型声明状态语义（`State`）
///
/// 等价于依次 `impl Component`, `impl State`。
#[macro_export]
macro_rules! state {
    ($($t:ty),+ $(,)?) => {
        $(impl $crate::Component for $t {})*
        $(impl $crate::State for $t {})*
    };
}
