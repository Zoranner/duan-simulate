//! 组件存储
//!
//! 按组件类型密集存储，提供 O(1) 访问和良好的缓存局部性。

use crate::entity::id::EntityId;
use crate::{Component, ComponentKind};
use std::any::{Any, TypeId};
use std::collections::HashMap;

// ──── AnyStorage：类型擦除的组件存储接口 ─────────────────────────────────

pub(crate) trait AnyStorage: Send + Sync {
    fn remove_entity(&mut self, id: EntityId);
    fn clone_box(&self) -> Box<dyn AnyStorage>;
    fn clone_for_snapshot_box(&self) -> Option<Box<dyn AnyStorage>>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

// ──── ComponentStorage<T>：单类型密集存储 ────────────────────────────────

/// 单类型组件密集存储
///
/// 连续内存布局，通过 sparse/dense 双索引实现 O(1) 随机访问和高效迭代。
pub struct ComponentStorage<T: Component> {
    /// 密集数组：连续内存，缓存友好
    dense: Vec<T>,
    /// 密集数组对应的 EntityId（与 dense 等长）
    dense_ids: Vec<EntityId>,
    /// EntityId.local_index → dense 槽位（None 表示不存在）
    sparse: Vec<Option<u32>>,
}

impl<T: Component> ComponentStorage<T> {
    pub fn new() -> Self {
        Self {
            dense: Vec::new(),
            dense_ids: Vec::new(),
            sparse: Vec::new(),
        }
    }

    fn ensure_sparse(&mut self, index: usize) {
        if self.sparse.len() <= index {
            self.sparse.resize(index + 1, None);
        }
    }

    pub fn insert(&mut self, id: EntityId, value: T) {
        let idx = id.local_index() as usize;
        self.ensure_sparse(idx);

        if let Some(slot) = self.sparse[idx] {
            // 已存在：原地替换
            self.dense[slot as usize] = value;
        } else {
            // 新增：追加到密集数组
            let slot = self.dense.len() as u32;
            self.dense.push(value);
            self.dense_ids.push(id);
            self.sparse[idx] = Some(slot);
        }
    }

    pub fn get(&self, id: EntityId) -> Option<&T> {
        let idx = id.local_index() as usize;
        let slot = *self.sparse.get(idx)?.as_ref()?;
        Some(&self.dense[slot as usize])
    }

    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut T> {
        let idx = id.local_index() as usize;
        let slot = *self.sparse.get(idx)?.as_ref()?;
        Some(&mut self.dense[slot as usize])
    }

    pub fn contains(&self, id: EntityId) -> bool {
        let idx = id.local_index() as usize;
        self.sparse.get(idx).and_then(|s| s.as_ref()).is_some()
    }

    pub fn remove(&mut self, id: EntityId) {
        let idx = id.local_index() as usize;
        let Some(slot) = self.sparse.get(idx).copied().flatten() else {
            return;
        };

        let slot = slot as usize;
        let last = self.dense.len() - 1;

        if slot != last {
            // swap-remove：将末尾元素移到被删槽位
            self.dense.swap(slot, last);
            self.dense_ids.swap(slot, last);
            // 更新移动元素的 sparse 指针
            let moved_id = self.dense_ids[slot];
            self.sparse[moved_id.local_index() as usize] = Some(slot as u32);
        }

        self.dense.pop();
        self.dense_ids.pop();
        self.sparse[idx] = None;
    }

    pub fn iter(&self) -> impl Iterator<Item = (EntityId, &T)> {
        self.dense_ids.iter().copied().zip(self.dense.iter())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (EntityId, &mut T)> {
        self.dense_ids.iter().copied().zip(self.dense.iter_mut())
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.dense.len()
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.dense.is_empty()
    }
}

impl<T: Component> Default for ComponentStorage<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Component> AnyStorage for ComponentStorage<T> {
    fn remove_entity(&mut self, id: EntityId) {
        self.remove(id);
    }

    fn clone_box(&self) -> Box<dyn AnyStorage> {
        Box::new(ComponentStorage {
            dense: self.dense.clone(),
            dense_ids: self.dense_ids.clone(),
            sparse: self.sparse.clone(),
        })
    }

    fn clone_for_snapshot_box(&self) -> Option<Box<dyn AnyStorage>> {
        if T::KIND == ComponentKind::Belief {
            return None;
        }
        Some(self.clone_box())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// ──── Storage：TypeId 管理所有 ComponentStorage ───────────────────────────

/// 世界组件存储（README：存储 / `Storage`）
///
/// 以 TypeId 为键管理所有类型的 ComponentStorage，
/// 提供统一的类型安全 get/insert/remove 接口。
pub struct Storage {
    storages: HashMap<TypeId, Box<dyn AnyStorage>>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            storages: HashMap::new(),
        }
    }

    fn get_storage<T: Component>(&self) -> Option<&ComponentStorage<T>> {
        self.storages
            .get(&TypeId::of::<T>())?
            .as_any()
            .downcast_ref::<ComponentStorage<T>>()
    }

    fn get_or_create_storage<T: Component>(&mut self) -> &mut ComponentStorage<T> {
        self.storages
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(ComponentStorage::<T>::new()))
            .as_any_mut()
            .downcast_mut::<ComponentStorage<T>>()
            .expect("TypeId mismatch — should never happen")
    }

    pub fn insert<T: Component>(&mut self, id: EntityId, value: T) {
        self.get_or_create_storage::<T>().insert(id, value);
    }

    pub fn get<T: Component>(&self, id: EntityId) -> Option<&T> {
        self.get_storage::<T>()?.get(id)
    }

    pub fn get_mut<T: Component>(&mut self, id: EntityId) -> Option<&mut T> {
        self.storages
            .get_mut(&TypeId::of::<T>())?
            .as_any_mut()
            .downcast_mut::<ComponentStorage<T>>()?
            .get_mut(id)
    }

    pub fn remove_component<T: Component>(&mut self, id: EntityId) {
        if let Some(s) = self.storages.get_mut(&TypeId::of::<T>()) {
            s.remove_entity(id);
        }
    }

    /// 移除实体的所有组件
    pub fn remove_entity(&mut self, id: EntityId) {
        for storage in self.storages.values_mut() {
            storage.remove_entity(id);
        }
    }

    pub fn iter<T: Component>(&self) -> impl Iterator<Item = (EntityId, &T)> {
        self.get_storage::<T>()
            .map(|s| s.iter())
            .into_iter()
            .flatten()
    }

    pub fn iter_mut<T: Component>(&mut self) -> impl Iterator<Item = (EntityId, &mut T)> {
        self.storages
            .get_mut(&TypeId::of::<T>())
            .and_then(|s| s.as_any_mut().downcast_mut::<ComponentStorage<T>>())
            .map(|s| s.iter_mut())
            .into_iter()
            .flatten()
    }

    /// 克隆当前存储作为快照（Belief 类型由调用方选择性排除）
    pub fn clone_all(&self) -> Self {
        Self {
            storages: self
                .storages
                .iter()
                .map(|(k, v)| (*k, v.clone_box()))
                .collect(),
        }
    }

    /// 克隆快照可见的所有存储（自动排除 `Belief`）
    pub fn clone_for_snapshot(&self) -> Self {
        Self {
            storages: self
                .storages
                .iter()
                .filter_map(|(k, v)| v.clone_for_snapshot_box().map(|s| (*k, s)))
                .collect(),
        }
    }

    pub fn contains_component<T: Component>(&self, id: EntityId) -> bool {
        self.get_storage::<T>().is_some_and(|s| s.contains(id))
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, PartialEq, Debug)]
    struct Pos {
        x: f64,
        y: f64,
    }

    impl Component for Pos {}

    fn make_id(idx: u32) -> EntityId {
        EntityId::new(0, 0, idx)
    }

    #[test]
    fn test_component_storage_insert_get() {
        let mut s = ComponentStorage::<Pos>::new();
        let id = make_id(1);
        s.insert(id, Pos { x: 1.0, y: 2.0 });

        assert!(s.contains(id));
        assert_eq!(s.get(id), Some(&Pos { x: 1.0, y: 2.0 }));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn test_component_storage_remove() {
        let mut s = ComponentStorage::<Pos>::new();
        let id1 = make_id(1);
        let id2 = make_id(2);

        s.insert(id1, Pos { x: 1.0, y: 0.0 });
        s.insert(id2, Pos { x: 2.0, y: 0.0 });

        s.remove(id1);
        assert!(!s.contains(id1));
        assert!(s.contains(id2));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn test_world_storage() {
        let mut storage = Storage::new();
        let id = make_id(5);

        storage.insert(id, Pos { x: 3.0, y: 4.0 });
        assert_eq!(storage.get::<Pos>(id), Some(&Pos { x: 3.0, y: 4.0 }));

        storage.get_mut::<Pos>(id).unwrap().x = 99.0;
        assert_eq!(storage.get::<Pos>(id).unwrap().x, 99.0);

        storage.remove_component::<Pos>(id);
        assert_eq!(storage.get::<Pos>(id), None);
    }

    #[test]
    fn test_world_storage_clone() {
        let mut storage = Storage::new();
        let id = make_id(1);
        storage.insert(id, Pos { x: 1.0, y: 2.0 });

        let cloned = storage.clone_all();
        assert_eq!(cloned.get::<Pos>(id), Some(&Pos { x: 1.0, y: 2.0 }));
    }
}
