//! 世界快照
//!
//! [`Snapshot`] 是 Phase 2（Entity tick）开始前对世界的只读冻结截面（README：快照 / `Snapshot`）。
//! 它提供只读的跨实体视图，确保同一帧内所有实体的感知基于一致的「上帧终态」。
//!
//! # 可见性规则
//!
//! | 术语（中文） | Rust trait | Snapshot 可见性 |
//! |-----------|-----------|-------------------|
//! | 认知 | Belief | 不可见（对外封闭）  |
//! | 意图 | Intent | 可见（只读）        |
//! | 事实 | Reality | 可见（只读）        |
//!
//! 认知（`Belief`）类型不进入快照，因此域和其他实体无法访问另一实体的认知数据。

use crate::entity::id::EntityId;
use crate::storage::Storage;
use crate::Component;

/// 世界快照（只读冻结副本，README：快照 / `Snapshot`）
///
/// Phase 2 开始前由框架自动构建，在 Phase 2 和 Phase 3 期间保持不变。
/// Phase 2 中实体经 [`EntityContext::snapshot`](crate::entity::context::EntityContext::snapshot) 读取与本结构一致的只读视图；
/// Phase 3 中域经 [`DomainContext::get`](crate::domain::context::DomainContext::get) 与
/// [`DomainContext::each`](crate::domain::context::DomainContext::each) 间接读快照（不直接持有 `Snapshot` 引用）。
pub struct Snapshot {
    storage: Storage,
}

impl Snapshot {
    /// 从当前世界存储构建快照（自动排除 Belief 类型）
    pub(crate) fn build(storage: &Storage) -> Self {
        Self {
            storage: storage.clone_for_snapshot(),
        }
    }

    /// 获取指定实体的 Intent 或 Reality 组件（只读）
    ///
    /// Belief 类型不在快照中，调用时会返回 None。
    pub fn get<T: Component>(&self, id: EntityId) -> Option<&T> {
        self.storage.get::<T>(id)
    }

    /// 遍历所有实体的某个 Intent 或 Reality 组件（只读）
    pub fn iter<T: Component>(&self) -> impl Iterator<Item = (EntityId, &T)> {
        self.storage.iter::<T>()
    }

    /// 检查指定实体是否具有某个组件
    pub fn contains<T: Component>(&self, id: EntityId) -> bool {
        self.storage.contains_component::<T>(id)
    }
}
