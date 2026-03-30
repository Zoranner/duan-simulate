//! 实体标识
//!
//! 结构化 64 位 EntityId，支持分布式扩展和悬空引用检测。
//!
//! # 位布局
//!
//! ```text
//! [63:48] node_id:u16    分布式节点标识（0 = 本地节点）
//! [47:32] generation:u16 代际标识（每次回收 +1，检测悬空引用）
//! [31:0]  local_index:u32 节点内唯一序号
//! ```

use std::collections::HashMap;
use std::fmt;

/// 实体唯一标识
///
/// 64 位结构化 ID，包含节点、代际、序号三段信息。
/// 代际字段使 EntityId 在实体销毁后不再有效，防止悬空引用访问已销毁实体。
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntityId(u64);

impl EntityId {
    /// 构造结构化 EntityId
    pub fn new(node_id: u16, generation: u16, local_index: u32) -> Self {
        Self(((node_id as u64) << 48) | ((generation as u64) << 32) | (local_index as u64))
    }

    /// 节点标识（0 = 本地节点）
    pub fn node_id(self) -> u16 {
        (self.0 >> 48) as u16
    }

    /// 代际标识（每次回收 +1）
    pub fn generation(self) -> u16 {
        ((self.0 >> 32) & 0xffff) as u16
    }

    /// 节点内序号
    pub fn local_index(self) -> u32 {
        (self.0 & 0xffff_ffff) as u32
    }

    /// 是否属于本地节点
    pub fn is_local(self) -> bool {
        self.node_id() == 0
    }

    /// 原始 u64 值
    pub fn raw(self) -> u64 {
        self.0
    }

    /// 构造无效（占位）ID（node=0, gen=0, index=0）
    pub(crate) fn placeholder() -> Self {
        Self(0)
    }
}

impl fmt::Debug for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "EntityId(node={}, gen={}, idx={})",
            self.node_id(),
            self.generation(),
            self.local_index()
        )
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.local_index())
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::placeholder()
    }
}

// ──── EntityAllocator ────────────────────────────────────────────────────

/// 实体 ID 分配器
///
/// 管理 EntityId 的分配与回收，维护代际以检测悬空引用。
/// 回收的序号槽位会在下一次分配时重用（代际 +1 区分新旧 ID）。
pub struct EntityAllocator {
    /// 节点标识（当前仅支持单节点，固定为 0）
    node_id: u16,
    /// 下一个待分配的序号
    next_index: u32,
    /// 已回收的槽位：index → 下一次分配时使用的代际
    free_list: Vec<u32>,
    /// 活跃 ID 的代际：index → generation
    generations: HashMap<u32, u16>,
}

impl EntityAllocator {
    pub fn new() -> Self {
        Self {
            node_id: 0,
            next_index: 1, // 0 保留为占位符
            free_list: Vec::new(),
            generations: HashMap::new(),
        }
    }

    /// 分配新的 EntityId
    pub fn allocate(&mut self) -> EntityId {
        if let Some(index) = self.free_list.pop() {
            let gen = *self.generations.get(&index).unwrap_or(&0);
            self.generations.insert(index, gen);
            EntityId::new(self.node_id, gen, index)
        } else {
            let index = self.next_index;
            self.next_index += 1;
            self.generations.insert(index, 0);
            EntityId::new(self.node_id, 0, index)
        }
    }

    /// 回收 EntityId（代际 +1，使旧 ID 失效）
    pub fn deallocate(&mut self, id: EntityId) {
        let index = id.local_index();
        let next_gen = id.generation().wrapping_add(1);
        self.generations.insert(index, next_gen);
        self.free_list.push(index);
    }

    /// 检查 EntityId 是否仍然有效
    pub fn is_alive(&self, id: EntityId) -> bool {
        self.generations
            .get(&id.local_index())
            .is_some_and(|&gen| gen == id.generation())
    }
}

impl Default for EntityAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_id_layout() {
        let id = EntityId::new(1, 2, 3);
        assert_eq!(id.node_id(), 1);
        assert_eq!(id.generation(), 2);
        assert_eq!(id.local_index(), 3);
    }

    #[test]
    fn test_entity_id_local() {
        let id = EntityId::new(0, 0, 42);
        assert!(id.is_local());

        let remote = EntityId::new(1, 0, 1);
        assert!(!remote.is_local());
    }

    #[test]
    fn test_allocator_basic() {
        let mut alloc = EntityAllocator::new();
        let id1 = alloc.allocate();
        let id2 = alloc.allocate();

        assert_ne!(id1, id2);
        assert!(alloc.is_alive(id1));
        assert!(alloc.is_alive(id2));
    }

    #[test]
    fn test_allocator_deallocate() {
        let mut alloc = EntityAllocator::new();
        let id = alloc.allocate();

        alloc.deallocate(id);
        assert!(!alloc.is_alive(id));

        // 回收后再分配，使用新代际
        let new_id = alloc.allocate();
        assert_ne!(id, new_id);
        assert_eq!(id.local_index(), new_id.local_index()); // 槽位复用
        assert_ne!(id.generation(), new_id.generation()); // 代际不同
    }
}
