//! 域（Domain）是 DUAN 仿真体系的核心概念，是权威计算单元
//!
//! 每个域是某个领域的唯一权威，负责该领域内的所有计算和判定。
//! 域之间通过事件和服务接口协作。

use crate::{Entity, EntityId, EntityStore, EventChannel, TimeClock};
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

/// 域规则 trait
///
/// 每个域必须实现此 trait，定义域的计算逻辑、准入规则和依赖关系。
///
/// # 生命周期
///
/// 1. 域注册时，`dependencies()` 被调用来确定执行顺序
/// 2. 实体创建时，`try_attach()` 被调用来决定是否接纳实体
/// 3. 每帧仿真时，`compute()` 被调用来执行计算
/// 4. 实体销毁时，`on_detach()` 被调用来清理数据
///
/// # 线程安全
///
/// 域规则必须是 `Send + Sync`，以支持跨线程使用（未来扩展）。
pub trait DomainRules: Send + Sync + 'static {
    /// 每帧计算
    ///
    /// 执行域的计算逻辑，产生事件。
    ///
    /// # 约束
    ///
    /// - 此阶段只能读取实体数据
    /// - 此阶段只能写入事件通道
    /// - 此阶段不能修改实体状态
    ///
    /// # 参数
    ///
    /// - `ctx`: 域上下文，提供对仿真环境的访问
    /// - `dt`: 帧时间（仿真时间步长）
    fn compute(&mut self, ctx: &mut DomainContext, dt: f64);

    /// 尝试附加实体
    ///
    /// 检查实体是否满足该域的准入条件。
    /// 返回 `true` 表示接纳，`false` 表示拒绝。
    ///
    /// # 典型实现
    ///
    /// 检查实体是否具有必要的组件。
    fn try_attach(&mut self, entity: &Entity) -> bool;

    /// 实体脱离该域
    ///
    /// 当实体被销毁或主动脱离时调用。
    /// 用于清理域中与该实体相关的数据。
    fn on_detach(&mut self, entity_id: EntityId);

    /// 声明依赖的域
    ///
    /// 返回该域依赖的其他域的名称列表。
    /// 被依赖的域会先执行，确保计算结果的一致性。
    ///
    /// # 默认
    ///
    /// 默认无依赖。
    fn dependencies(&self) -> Vec<&'static str> {
        vec![]
    }

    /// 类型转换（只读）
    ///
    /// 用于将 trait 对象转换回具体类型。
    fn as_any(&self) -> &dyn Any;

    /// 类型转换（可变）
    ///
    /// 用于将 trait 对象转换回具体类型（可变引用）。
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// 域上下文
///
/// 域访问仿真环境的唯一入口。提供：
/// - 实体存储（只读）
/// - 域注册表（只读，用于查询其他域）
/// - 事件通道（只写）
/// - 仿真时钟（只读）
pub struct DomainContext<'a> {
    /// 实体存储（只读）
    pub entities: &'a EntityStore,
    /// 域注册表（只读）
    pub registry: &'a DomainRegistry,
    /// 事件通道（只写）
    pub events: &'a mut EventChannel,
    /// 仿真时钟（只读）
    pub clock: &'a TimeClock,
    /// 当前帧时间
    pub dt: f64,
}

impl<'a> DomainContext<'a> {
    /// 获取当前仿真时间
    pub fn sim_time(&self) -> f64 {
        self.clock.sim_time
    }

    /// 获取已执行的仿真步数
    pub fn step_count(&self) -> u64 {
        self.clock.step_count
    }

    /// 检查仿真是否暂停
    pub fn is_paused(&self) -> bool {
        !self.clock.running
    }

    /// 获取指定类型的域
    ///
    /// 推荐用于静态已知的域，提供编译时类型检查。
    pub fn get_domain<T: DomainRules>(&self) -> Option<&T> {
        self.registry.get::<T>()
    }

    /// 获取指定名称的域
    ///
    /// 适用于动态场景或配置驱动的场景。
    pub fn get_domain_by_name(&self, name: &str) -> Option<&Domain> {
        self.registry.get_by_name(name)
    }

    /// 发出事件
    ///
    /// 将事件添加到事件通道。
    pub fn emit<E: Into<crate::events::DomainEvent>>(&mut self, event: E) {
        self.events.push(event.into());
    }
}

/// 域
///
/// 域的运行时表示，包含名称、实体列表和规则实现。
pub struct Domain {
    /// 域的唯一标识（字符串）
    pub name: String,
    /// 归属该域的实体列表
    pub entities: HashSet<EntityId>,
    /// 该域的规则实现
    pub rules: Box<dyn DomainRules>,
}

impl Domain {
    /// 创建一个新域
    pub fn new(name: impl Into<String>, rules: impl DomainRules) -> Self {
        Self {
            name: name.into(),
            entities: HashSet::new(),
            rules: Box::new(rules),
        }
    }

    /// 尝试附加实体
    ///
    /// 如果实体被接纳，将其添加到实体列表。
    pub fn try_attach(&mut self, entity: &Entity) -> bool {
        if self.rules.try_attach(entity) {
            self.entities.insert(entity.id);
            true
        } else {
            false
        }
    }

    /// 脱离实体
    pub fn detach(&mut self, entity_id: EntityId) {
        self.entities.remove(&entity_id);
        self.rules.on_detach(entity_id);
    }

    /// 检查实体是否属于该域
    pub fn contains(&self, entity_id: EntityId) -> bool {
        self.entities.contains(&entity_id)
    }

    /// 获取域中的实体数量
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// 检查域是否为空
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// 迭代域中的实体 ID
    pub fn entity_ids(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.entities.iter().copied()
    }
}

/// 域注册表
///
/// 管理所有域，支持按类型/名称查找域、执行调度。
pub struct DomainRegistry {
    /// 域映射：名称 -> 域
    domains: HashMap<String, Domain>,
    /// 类型索引：TypeId -> 域名称
    type_index: HashMap<TypeId, String>,
    /// 缓存的执行顺序（拓扑排序结果）
    execution_order: RefCell<Vec<String>>,
    /// 执行顺序是否需要重新计算
    order_dirty: RefCell<bool>,
}

impl DomainRegistry {
    /// 创建空的域注册表
    pub fn new() -> Self {
        Self {
            domains: HashMap::new(),
            type_index: HashMap::new(),
            execution_order: RefCell::new(Vec::new()),
            order_dirty: RefCell::new(true),
        }
    }

    /// 注册域
    ///
    /// 将域添加到注册表，类型信息会被记录用于类型查找。
    pub fn register<T: DomainRules>(&mut self, name: &str, rules: T) {
        self.type_index.insert(TypeId::of::<T>(), name.to_string());
        self.domains.insert(
            name.to_string(),
            Domain {
                name: name.to_string(),
                entities: HashSet::new(),
                rules: Box::new(rules),
            },
        );
        *self.order_dirty.borrow_mut() = true;
    }

    /// 注册域（带名称的 Box<dyn DomainRules>）
    pub fn register_boxed(&mut self, name: &str, rules: Box<dyn DomainRules>) {
        self.domains.insert(
            name.to_string(),
            Domain {
                name: name.to_string(),
                entities: HashSet::new(),
                rules,
            },
        );
        *self.order_dirty.borrow_mut() = true;
    }

    /// 按类型获取域
    ///
    /// 通过域的实现类型来查找域。
    pub fn get<T: DomainRules>(&self) -> Option<&T> {
        let name = self.type_index.get(&TypeId::of::<T>())?;
        let domain = self.domains.get(name)?;
        domain.rules.as_any().downcast_ref::<T>()
    }

    /// 按类型获取可变域
    pub fn get_mut<T: DomainRules>(&mut self) -> Option<&mut T> {
        let name = self.type_index.get(&TypeId::of::<T>())?.clone();
        let domain = self.domains.get_mut(&name)?;
        domain.rules.as_any_mut().downcast_mut::<T>()
    }

    /// 按名称获取域
    pub fn get_by_name(&self, name: &str) -> Option<&Domain> {
        self.domains.get(name)
    }

    /// 按名称获取可变域
    pub fn get_by_name_mut(&mut self, name: &str) -> Option<&mut Domain> {
        self.domains.get_mut(name)
    }

    /// 获取执行顺序
    ///
    /// 返回按拓扑排序后的域名称列表。
    /// 使用内部可变性，允许通过 `&self` 调用。
    pub fn execution_order(&self) -> std::cell::Ref<'_, Vec<String>> {
        if *self.order_dirty.borrow() {
            self.compute_execution_order();
            *self.order_dirty.borrow_mut() = false;
        }
        self.execution_order.borrow()
    }

    /// 计算执行顺序（拓扑排序）
    fn compute_execution_order(&self) {
        let mut order = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_mark = HashSet::new();

        fn visit(
            name: &str,
            domains: &HashMap<String, Domain>,
            visited: &mut HashSet<String>,
            temp_mark: &mut HashSet<String>,
            order: &mut Vec<String>,
        ) {
            if visited.contains(name) {
                return;
            }
            if temp_mark.contains(name) {
                panic!("Cyclic domain dependency detected at: {}", name);
            }

            temp_mark.insert(name.to_string());

            if let Some(domain) = domains.get(name) {
                for dep in domain.rules.dependencies() {
                    visit(dep, domains, visited, temp_mark, order);
                }
            }

            temp_mark.remove(name);
            visited.insert(name.to_string());
            order.push(name.to_string());
        }

        for name in self.domains.keys() {
            visit(
                name,
                &self.domains,
                &mut visited,
                &mut temp_mark,
                &mut order,
            );
        }

        *self.execution_order.borrow_mut() = order;
    }

    /// 检查域是否存在
    pub fn contains(&self, name: &str) -> bool {
        self.domains.contains_key(name)
    }

    /// 获取域数量
    pub fn len(&self) -> usize {
        self.domains.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.domains.is_empty()
    }

    /// 迭代所有域
    pub fn iter(&self) -> impl Iterator<Item = &Domain> {
        self.domains.values()
    }

    /// 迭代所有可变域
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Domain> {
        self.domains.values_mut()
    }

    /// 标记执行顺序需要重新计算
    pub fn mark_dirty(&mut self) {
        *self.order_dirty.borrow_mut() = true;
    }
}

impl Default for DomainRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestRules;

    impl DomainRules for TestRules {
        fn compute(&mut self, _ctx: &mut DomainContext, _dt: f64) {}
        fn try_attach(&mut self, _entity: &Entity) -> bool {
            true
        }
        fn on_detach(&mut self, _entity_id: EntityId) {}
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    #[test]
    fn test_domain_registry() {
        let mut registry = DomainRegistry::new();
        registry.register("test", TestRules);

        assert!(registry.contains("test"));
        assert_eq!(registry.len(), 1);
        assert!(registry.get::<TestRules>().is_some());
    }

    #[test]
    fn test_execution_order() {
        let mut registry = DomainRegistry::new();
        registry.register("a", TestRules);
        registry.register("b", TestRules);

        let order = registry.execution_order();
        assert_eq!(order.len(), 2);
    }
}
