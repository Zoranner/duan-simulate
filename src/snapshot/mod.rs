//! 世界快照
//!
//! [`WorldSnapshot`] 是 Phase 2（Entity tick）开始前对世界状态的冻结副本。
//! 它提供只读的跨实体视图，确保同一帧内所有实体的感知基于一致的"上帧终态"。
//!
//! # 可见性规则
//!
//! | 术语（中文） | Rust trait | WorldSnapshot 可见性 |
//! |-----------|-----------|-------------------|
//! | 认知 | Memory | 不可见（对外封闭）  |
//! | 意图 | Intent | 可见（只读）        |
//! | 状态 | State  | 可见（只读）        |
//!
//! 认知（`Memory`）类型不进入快照，因此域和其他实体无法访问另一实体的认知数据。

use crate::entity::id::EntityId;
use crate::storage::WorldStorage;
use crate::Component;

/// 世界快照（只读冻结副本）
///
/// Phase 2 开始前由框架自动构建，在 Phase 2 和 Phase 3 期间保持不变。
/// Phase 2 中实体经 [`EntityContext::snapshot`](crate::entity::context::EntityContext::snapshot) 读取与本结构一致的只读视图；
/// Phase 3 中域经 [`DomainContext::get`](crate::domain::context::DomainContext::get) 与
/// [`DomainContext::each`](crate::domain::context::DomainContext::each) 间接读快照（不直接持有 `WorldSnapshot` 引用）。
pub struct WorldSnapshot {
    storage: WorldStorage,
}

impl WorldSnapshot {
    /// 从当前世界存储构建快照（自动排除 Memory 类型）
    pub(crate) fn build(storage: &WorldStorage) -> Self {
        Self {
            storage: storage.clone_for_snapshot(),
        }
    }

    /// 获取指定实体的 Intent 或 State 组件（只读）
    ///
    /// Memory 类型不在快照中，调用时会返回 None。
    pub fn get<T: Component>(&self, id: EntityId) -> Option<&T> {
        self.storage.get::<T>(id)
    }

    /// 遍历所有实体的某个 Intent 或 State 组件（只读）
    pub fn iter<T: Component>(&self) -> impl Iterator<Item = (EntityId, &T)> {
        self.storage.iter::<T>()
    }

    /// 检查指定实体是否具有某个组件
    pub fn contains<T: Component>(&self, id: EntityId) -> bool {
        self.storage.contains_component::<T>(id)
    }
}
