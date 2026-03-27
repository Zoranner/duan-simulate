//! 实体（Entity）是仿真中的基本对象单元
//!
//! 实体是组件的容器，通过组合不同的组件来表达不同的能力。
//! 实体本身不包含行为逻辑，行为由域来决定。

use crate::Component;
use std::any::TypeId;
use std::collections::{HashMap, HashSet};

/// 实体标识
///
/// 每个实体有一个唯一标识，用于在仿真中区分不同的实体。
/// 标识一旦创建就不会改变，即使实体被销毁，其标识也不会被复用。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, PartialOrd, Ord)]
pub struct EntityId(pub u64);

impl EntityId {
    /// 创建一个新的实体标识
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// 获取原始标识值
    pub fn raw(&self) -> u64 {
        self.0
    }
}

/// 生命周期状态
///
/// 描述实体从创建到销毁的状态变迁过程。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Lifecycle {
    /// 刚创建，正在执行域归属校验，尚未加入任何域
    #[default]
    Initializing,
    /// 正常活动，参与仿真
    Active,
    /// 正在销毁（过渡阶段），已从所有域完全脱离
    Destroying,
    /// 已销毁，等待清理
    Destroyed,
}

impl Lifecycle {
    /// 检查实体是否活跃
    pub fn is_active(&self) -> bool {
        matches!(self, Lifecycle::Active)
    }

    /// 检查实体是否可以被清理
    pub fn is_destroyed(&self) -> bool {
        matches!(self, Lifecycle::Destroyed)
    }

    /// 检查实体是否应该参与计算
    ///
    /// 只有活跃状态的实体参与计算
    pub fn should_compute(&self) -> bool {
        matches!(self, Lifecycle::Active)
    }
}

/// 组件容器
///
/// 存储实体的所有组件，支持 O(1) 时间复杂度的组件查询。
/// 通过 `Entity` 上的方法访问；不直接暴露给外部使用者。
pub(crate) struct ComponentBag {
    components: HashMap<TypeId, Box<dyn Component>>,
}

#[allow(dead_code)]
impl ComponentBag {
    /// 创建一个空的组件容器
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    /// 添加组件
    ///
    /// 如果已存在相同类型的组件，会替换旧的组件。
    pub fn add<T: Component>(&mut self, component: T) {
        self.components
            .insert(TypeId::of::<T>(), Box::new(component));
    }

    /// 获取组件（只读）
    ///
    /// 返回指定类型组件的不可变引用。
    pub fn get<T: Component>(&self) -> Option<&T> {
        self.components
            .get(&TypeId::of::<T>())
            .and_then(|c| c.as_any().downcast_ref::<T>())
    }

    /// 获取组件（可变）
    ///
    /// 返回指定类型组件的可变引用。
    pub fn get_mut<T: Component>(&mut self) -> Option<&mut T> {
        self.components
            .get_mut(&TypeId::of::<T>())
            .and_then(|c| c.as_any_mut().downcast_mut::<T>())
    }

    /// 检查是否有组件
    ///
    /// 检查是否存在指定类型的组件。
    pub fn has<T: Component>(&self) -> bool {
        self.components.contains_key(&TypeId::of::<T>())
    }

    /// 移除组件
    ///
    /// 移除并返回指定类型的组件。
    pub fn remove<T: Component + 'static>(&mut self) -> Option<T> {
        self.components
            .remove(&TypeId::of::<T>())
            .and_then(|boxed| {
                let any_box = boxed.into_any_boxed();
                any_box.downcast::<T>().ok().map(|b| *b)
            })
    }

    /// 获取组件数量
    pub fn len(&self) -> usize {
        self.components.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    /// 获取所有组件的类型 ID
    pub fn component_types(&self) -> impl Iterator<Item = TypeId> + '_ {
        self.components.keys().copied()
    }

    /// 清空所有组件
    pub fn clear(&mut self) {
        self.components.clear();
    }
}

impl Default for ComponentBag {
    fn default() -> Self {
        Self::new()
    }
}

/// 实体
///
/// 仿真中的基本对象单元，是组件的容器。
pub struct Entity {
    /// 唯一标识（由 `World::spawn` 分配，构造前为占位符 `EntityId(0)`）
    pub id: EntityId,
    /// 实体类型（字符串，用于分类和展示）
    pub entity_type: String,
    /// 生命周期状态
    pub lifecycle: Lifecycle,
    /// 自声明的域归属列表
    pub domains: HashSet<String>,
    /// 组件容器（通过 Entity 方法访问）
    pub(crate) components: ComponentBag,
}

impl Entity {
    /// 创建一个新实体
    ///
    /// ID 由 `World::spawn` 在入世时自动分配，构造阶段无需提供。
    /// 若需在 spawn 前预知 ID（如跨实体引用），可通过 `World::generate_entity_id` 显式生成
    /// 并用 `Entity::with_id` 构造。
    pub fn new(entity_type: impl Into<String>) -> Self {
        Self {
            id: EntityId::default(), // 占位，spawn 时覆写
            entity_type: entity_type.into(),
            lifecycle: Lifecycle::Initializing,
            domains: HashSet::new(),
            components: ComponentBag::new(),
        }
    }

    /// 创建指定 ID 的实体（高级用法）
    ///
    /// 用于需要在 spawn 前就持有 ID 的场景，例如跨实体相互引用。
    /// 通常情况下应使用 `Entity::new`，由框架自动分配 ID。
    pub fn with_id(id: EntityId, entity_type: impl Into<String>) -> Self {
        Self {
            id,
            entity_type: entity_type.into(),
            lifecycle: Lifecycle::Initializing,
            domains: HashSet::new(),
            components: ComponentBag::new(),
        }
    }

    /// 声明域归属
    pub fn with_domain(mut self, domain: impl Into<String>) -> Self {
        self.domains.insert(domain.into());
        self
    }

    /// 添加组件
    pub fn with_component<T: Component>(mut self, component: T) -> Self {
        self.components.add(component);
        self
    }

    /// 获取组件
    pub fn get_component<T: Component>(&self) -> Option<&T> {
        self.components.get()
    }

    /// 获取可变组件
    pub fn get_component_mut<T: Component>(&mut self) -> Option<&mut T> {
        self.components.get_mut()
    }

    /// 检查是否有组件
    pub fn has_component<T: Component>(&self) -> bool {
        self.components.has::<T>()
    }

    /// 检查实体是否活跃
    pub fn is_active(&self) -> bool {
        self.lifecycle.is_active()
    }

    /// 检查实体是否已销毁
    pub fn is_destroyed(&self) -> bool {
        self.lifecycle.is_destroyed()
    }

    /// 检查实体是否应该参与计算
    pub fn should_compute(&self) -> bool {
        self.lifecycle.should_compute()
    }

    /// 添加域归属
    pub fn add_domain(&mut self, domain: impl Into<String>) {
        self.domains.insert(domain.into());
    }

    /// 移除域归属
    pub fn remove_domain(&mut self, domain: &str) {
        self.domains.remove(domain);
    }

    /// 检查是否属于某个域
    pub fn belongs_to(&self, domain: &str) -> bool {
        self.domains.contains(domain)
    }
}

/// 实体存储容器
///
/// 管理所有实体，支持通过标识获取实体和按类型查询实体。
pub struct EntityStore {
    /// 实体映射：EntityId -> Entity
    entities: HashMap<EntityId, Entity>,
    /// 按类型索引（用于分类查询）
    by_type: HashMap<String, HashSet<EntityId>>,
}

impl EntityStore {
    /// 创建一个空的实体存储
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            by_type: HashMap::new(),
        }
    }

    /// 添加实体
    pub fn insert(&mut self, entity: Entity) {
        let id = entity.id;
        let entity_type = entity.entity_type.clone();
        self.entities.insert(id, entity);
        self.by_type.entry(entity_type).or_default().insert(id);
    }

    /// 获取实体
    pub fn get(&self, id: EntityId) -> Option<&Entity> {
        self.entities.get(&id)
    }

    /// 获取可变实体
    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        self.entities.get_mut(&id)
    }

    /// 移除实体
    pub fn remove(&mut self, id: EntityId) -> Option<Entity> {
        let entity = self.entities.remove(&id)?;
        if let Some(type_set) = self.by_type.get_mut(&entity.entity_type) {
            type_set.remove(&id);
        }
        Some(entity)
    }

    /// 迭代所有实体
    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.entities.values()
    }

    /// 迭代所有可变实体
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Entity> {
        self.entities.values_mut()
    }

    /// 按类型查询实体
    pub fn by_type(&self, entity_type: &str) -> impl Iterator<Item = &Entity> {
        self.by_type
            .get(entity_type)
            .into_iter()
            .flat_map(|ids| ids.iter())
            .filter_map(|id| self.entities.get(id))
    }

    /// 获取活跃实体
    pub fn active_entities(&self) -> impl Iterator<Item = &Entity> {
        self.entities.values().filter(|e| e.should_compute())
    }

    /// 获取活跃实体数量
    pub fn active_count(&self) -> usize {
        self.entities.values().filter(|e| e.is_active()).count()
    }

    /// 获取实体数量
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// 检查实体是否存在
    pub fn contains(&self, id: EntityId) -> bool {
        self.entities.contains_key(&id)
    }

    /// 检查实体是否正在销毁过渡中
    ///
    /// 返回 `true` 表示实体已触发销毁（`world.destroy()` 已被调用），正处于销毁过渡期。
    /// 此时实体仍在实体存储中可被读取，但已从所有域完全脱离，不参与计算。
    ///
    /// 常见用途：在事件处理器中避免对同一实体重复发出销毁事件。
    pub fn is_destroying(&self, id: EntityId) -> bool {
        self.entities
            .get(&id)
            .is_some_and(|e| matches!(e.lifecycle, Lifecycle::Destroying))
    }

    /// 清空所有实体
    pub fn clear(&mut self) {
        self.entities.clear();
        self.by_type.clear();
    }
}

impl Default for EntityStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;

    struct TestComponent {
        value: i32,
    }

    impl Component for TestComponent {
        fn component_type(&self) -> &'static str {
            "test"
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
        fn into_any_boxed(self: Box<Self>) -> Box<dyn Any> {
            self
        }
    }

    #[test]
    fn test_entity_creation() {
        let entity = Entity::new("ship");
        assert_eq!(entity.entity_type, "ship");
        assert_eq!(entity.lifecycle, Lifecycle::Initializing);

        // with_id 用于需要预知 ID 的场景
        let entity_with_id = Entity::with_id(EntityId::new(42), "ship");
        assert_eq!(entity_with_id.id.raw(), 42);
    }

    #[test]
    fn test_component_bag() {
        let mut bag = ComponentBag::new();
        bag.add(TestComponent { value: 42 });

        assert!(bag.has::<TestComponent>());
        assert_eq!(bag.get::<TestComponent>().unwrap().value, 42);

        bag.get_mut::<TestComponent>().unwrap().value = 100;
        assert_eq!(bag.get::<TestComponent>().unwrap().value, 100);
    }

    #[test]
    fn test_entity_store() {
        let mut store = EntityStore::new();
        let id = EntityId::new(1);

        let entity = Entity::with_id(id, "ship").with_domain("space");
        store.insert(entity);

        assert!(store.contains(id));
        assert_eq!(store.len(), 1);
        assert!(store.get(id).unwrap().belongs_to("space"));
    }
}
