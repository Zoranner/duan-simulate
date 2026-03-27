//! 世界（World）是仿真系统的顶层容器
//!
//! 世界是仿真的整体环境，包含所有实体、域、时钟和事件通道。
//! 世界负责协调仿真循环的执行。

use crate::{
    CustomEvent, DomainContext, DomainEvent, DomainRegistry, DomainRules, Entity, EntityId,
    EntityStore, EventChannel, Lifecycle, TimeClock, Timer, TimerCallback, TimerManager,
};
use std::collections::HashSet;

/// 世界构建器
///
/// 用于逐步配置和创建世界实例。
pub struct WorldBuilder {
    time_scale: f64,
    paused: bool,
}

impl WorldBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            time_scale: 1.0,
            paused: false,
        }
    }

    /// 设置时间比例
    pub fn time_scale(mut self, scale: f64) -> Self {
        self.time_scale = scale;
        self
    }

    /// 设置初始暂停状态
    pub fn paused(mut self, paused: bool) -> Self {
        self.paused = paused;
        self
    }

    /// 构建世界
    pub fn build(self) -> World {
        let clock = if self.paused {
            let mut c = TimeClock::paused();
            c.time_scale = self.time_scale;
            c
        } else {
            TimeClock::with_scale(self.time_scale)
        };

        World {
            clock,
            domains: DomainRegistry::new(),
            entities: EntityStore::new(),
            events: EventChannel::new(),
            timer_manager: TimerManager::new(),
            next_entity_id: 1,
        }
    }
}

impl Default for WorldBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 世界
///
/// 仿真系统的顶层容器，协调各组件的工作。
pub struct World {
    /// 仿真时钟
    pub clock: TimeClock,
    /// 域注册表
    pub domains: DomainRegistry,
    /// 实体存储
    pub entities: EntityStore,
    /// 事件通道
    pub events: EventChannel,
    /// 定时器管理器
    pub timer_manager: TimerManager,
    /// 下一个实体 ID
    next_entity_id: u64,
}

impl World {
    /// 创建新的世界
    pub fn new() -> Self {
        WorldBuilder::new().build()
    }

    /// 创建世界构建器
    pub fn builder() -> WorldBuilder {
        WorldBuilder::new()
    }

    /// 生成新的实体 ID（高级用法）
    ///
    /// 通常不需要显式调用，`spawn` 会自动分配 ID。
    /// 仅在需要跨实体相互引用、必须在 spawn 前预知 ID 时使用。
    pub fn generate_entity_id(&mut self) -> EntityId {
        let id = EntityId::new(self.next_entity_id);
        self.next_entity_id += 1;
        id
    }

    /// 注册域
    pub fn register_domain<T: DomainRules>(&mut self, name: &str, rules: T) {
        self.domains.register(name, rules);
    }

    /// 创建实体并加入仿真
    ///
    /// ID 由框架自动分配（覆写 entity.id）。返回分配到的 `EntityId`。
    /// 实体入世后尝试附加到其声明的各个域，随后进入 `Active` 状态。
    pub fn spawn(&mut self, mut entity: Entity) -> EntityId {
        // 分配 ID（覆写构造阶段的占位符）
        let id = self.generate_entity_id();
        entity.id = id;

        let declared_domains: Vec<String> = entity.domains.iter().cloned().collect();

        // 存储实体
        self.entities.insert(entity);

        // 尝试附加到各域
        for domain_name in declared_domains {
            if let Some(entity_ref) = self.entities.get(id) {
                if let Some(domain) = self.domains.get_by_name_mut(&domain_name) {
                    domain.try_attach(entity_ref);
                }
            }
        }

        // 设置为活跃状态
        if let Some(entity) = self.entities.get_mut(id) {
            entity.lifecycle = Lifecycle::Active;
        }

        id
    }

    /// 销毁实体
    ///
    /// 将实体转入销毁中状态，框架自动执行：
    /// 1. 从所有域中脱离（调用脱离接口）
    /// 2. 取消该实体的所有现有定时器
    /// 3. 调度过渡期定时器（到期后进入已销毁状态）
    pub fn destroy(&mut self, entity_id: EntityId, destroy_time: f64) {
        let is_active = self
            .entities
            .get(entity_id)
            .map_or(false, |e| e.lifecycle == Lifecycle::Active);

        if !is_active {
            return;
        }

        // 1. 设为销毁中状态
        if let Some(entity) = self.entities.get_mut(entity_id) {
            entity.lifecycle = Lifecycle::Destroying;
        }

        // 2. 从所有域中脱离
        for domain in self.domains.iter_mut() {
            if domain.contains(entity_id) {
                domain.detach(entity_id);
            }
        }

        // 3. 取消现有定时器
        self.timer_manager.remove_entity(entity_id);

        // 4. 调度过渡期定时器
        self.timer_manager.schedule(
            entity_id,
            Timer::self_destruct(self.clock.sim_time + destroy_time),
        );
    }

    /// 直接移除实体（不经过销毁动画）
    pub fn remove_entity(&mut self, entity_id: EntityId) -> Option<Entity> {
        // 从所有域中脱离
        for domain in self.domains.iter_mut() {
            if domain.contains(entity_id) {
                domain.detach(entity_id);
            }
        }

        // 取消定时器
        self.timer_manager.remove_entity(entity_id);

        // 从实体存储中移除
        self.entities.remove(entity_id)
    }

    /// 执行一步仿真（无自定义事件处理）
    ///
    /// 执行完整的仿真循环：时间推进 → 域计算 → 定时器检查 → 事件处理 → 清理。
    /// 自定义事件在此版本中被忽略。若需处理自定义事件，使用 `step_with`。
    pub fn step(&mut self, dt: f64) {
        self.do_step(dt, &mut |_: &dyn CustomEvent, _: &mut World| {});
    }

    /// 执行一步仿真（带自定义事件处理器）
    ///
    /// 与 `step` 相同，额外接受一个闭包用于处理自定义事件。
    /// 闭包签名：`|event: &dyn CustomEvent, world: &mut World|`
    pub fn step_with<F>(&mut self, dt: f64, mut handler: F)
    where
        F: FnMut(&dyn CustomEvent, &mut Self),
    {
        self.do_step(dt, &mut handler);
    }

    /// 仿真步内部实现
    fn do_step<F>(&mut self, dt: f64, handler: &mut F)
    where
        F: FnMut(&dyn CustomEvent, &mut Self),
    {
        // 阶段 1：时间推进
        let sim_dt = self.clock.tick(dt);
        if sim_dt == 0.0 {
            return;
        }

        // 阶段 2：域计算
        self.compute_domains(sim_dt);

        // 阶段 3：定时器检查
        let sim_time = self.clock.sim_time;
        self.check_timers(sim_time);

        // 阶段 4：事件处理
        self.process_events(handler);

        // 阶段 5：清理
        self.cleanup();
    }

    /// 域计算阶段
    fn compute_domains(&mut self, dt: f64) {
        let order: Vec<String> = self.domains.execution_order().to_vec();

        for domain_name in &order {
            // 从 domain 中同时取出 rules 指针和 own_entities 指针
            // SAFETY：两个指针指向 Domain 结构体的不同字段（rules 和 entities），不存在别名。
            // compute_domains 期间注册表结构不变（无 insert/remove），
            // 域自身的实体集合通过 ctx.registry（不可变引用）也不会被修改。
            let (rules_ptr, own_entities_ptr) = match self.domains.get_by_name_mut(domain_name) {
                Some(domain) => (
                    &mut *domain.rules as *mut dyn DomainRules,
                    &domain.entities as *const HashSet<EntityId>,
                ),
                None => continue,
            };

            let mut ctx = DomainContext {
                own_entities: unsafe { &*own_entities_ptr },
                entities: &mut self.entities,
                registry: &self.domains,
                events: &mut self.events,
                clock: &self.clock,
                dt,
            };

            unsafe {
                (*rules_ptr).compute(&mut ctx, dt);
            }
        }
    }

    /// 定时器检查阶段
    fn check_timers(&mut self, sim_time: f64) {
        let timer_events = self.timer_manager.check(sim_time);

        for event in timer_events {
            self.events.push(DomainEvent::Timer {
                entity_id: event.entity_id,
                timer_id: event.timer_id,
                callback: event.callback,
            });
        }
    }

    /// 事件处理阶段
    fn process_events<F>(&mut self, handler: &mut F)
    where
        F: FnMut(&dyn CustomEvent, &mut Self),
    {
        let events = std::mem::take(&mut self.events);

        for event in events {
            // 自定义事件先交给用户处理器
            if let DomainEvent::Custom(event_arc) = &event {
                handler(event_arc.as_ref(), self);
            }
            self.handle_event(event);
        }
    }

    /// 处理单个事件
    fn handle_event(&mut self, event: DomainEvent) {
        match event {
            DomainEvent::EntitySpawned {
                entity_id: _,
                entity_type: _,
            } => {
                // 实体创建事件（通常在 spawn 中处理）
            }

            DomainEvent::EntityDestroyed {
                entity_id,
                cause: _,
            } => {
                if let Some(entity) = self.entities.get_mut(entity_id) {
                    entity.lifecycle = Lifecycle::Destroyed;
                }
            }

            DomainEvent::Timer {
                entity_id,
                timer_id,
                callback,
            } => {
                self.handle_timer_event(entity_id, timer_id, callback);
            }

            DomainEvent::Custom(_) => {
                // 已由 process_events 中的用户处理器处理
            }
        }
    }

    /// 处理定时器事件
    fn handle_timer_event(
        &mut self,
        entity_id: EntityId,
        _timer_id: String,
        callback: TimerCallback,
    ) {
        match callback {
            TimerCallback::SelfDestruct => {
                if let Some(entity) = self.entities.get_mut(entity_id) {
                    entity.lifecycle = Lifecycle::Destroyed;
                }
            }
            TimerCallback::Event(domain_event) => {
                self.handle_event(*domain_event);
            }
            TimerCallback::Custom(_callback_id) => {
                // 自定义回调：由用户处理
            }
        }
    }

    /// 清理阶段
    ///
    /// 实体在进入销毁中状态时已从所有域完全脱离，定时器也已全部取消，
    /// 此阶段只需从实体存储中移除。
    fn cleanup(&mut self) {
        let destroyed_ids: Vec<EntityId> = self
            .entities
            .iter()
            .filter(|e| e.lifecycle.is_destroyed())
            .map(|e| e.id)
            .collect();

        for id in destroyed_ids {
            self.entities.remove(id);
        }
    }

    /// 获取实体（只读）
    pub fn get_entity(&self, id: EntityId) -> Option<&Entity> {
        self.entities.get(id)
    }

    /// 获取实体（可变）
    ///
    /// 用于事件处理阶段修改实体状态。
    pub fn get_entity_mut(&mut self, entity_id: EntityId) -> Option<&mut Entity> {
        self.entities.get_mut(entity_id)
    }

    /// 获取域（只读）
    pub fn get_domain<T: DomainRules>(&self) -> Option<&T> {
        self.domains.get::<T>()
    }

    /// 获取域（可变）
    pub fn get_domain_mut<T: DomainRules>(&mut self) -> Option<&mut T> {
        self.domains.get_mut::<T>()
    }

    /// 获取当前仿真时间
    pub fn sim_time(&self) -> f64 {
        self.clock.sim_time
    }

    /// 获取实体数量
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// 获取活跃实体数量
    pub fn active_entity_count(&self) -> usize {
        self.entities.active_count()
    }

    /// 暂停仿真
    pub fn pause(&mut self) {
        self.clock.pause();
    }

    /// 恢复仿真
    pub fn resume(&mut self) {
        self.clock.resume();
    }

    /// 设置时间比例
    pub fn set_time_scale(&mut self, scale: f64) {
        self.clock.set_time_scale(scale);
    }

    /// 检查是否暂停
    pub fn is_paused(&self) -> bool {
        self.clock.is_paused()
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_creation() {
        let world = World::new();
        assert_eq!(world.entity_count(), 0);
        assert_eq!(world.sim_time(), 0.0);
    }

    #[test]
    fn test_world_builder() {
        let world = World::builder().time_scale(2.0).paused(true).build();

        assert!(world.is_paused());
        assert_eq!(world.clock.time_scale, 2.0);
    }

    #[test]
    fn test_entity_spawn() {
        let mut world = World::new();
        let id = world.spawn(Entity::new("ship"));

        assert_eq!(world.entity_count(), 1);
        assert!(world.get_entity(id).is_some());
        // ID 从 1 开始，由框架分配
        assert_eq!(id.raw(), 1);
    }

    #[test]
    fn test_time_advancement() {
        let mut world = World::new();
        world.step(0.1);

        assert_eq!(world.sim_time(), 0.1);
    }

    #[test]
    fn test_step_with_handler() {
        let mut world = World::new();
        let mut received = false;

        world.step_with(0.1, |_event, _world| {
            received = true;
        });

        // 没有自定义事件，handler 不会被调用
        assert!(!received);
        assert_eq!(world.sim_time(), 0.1);
    }
}
