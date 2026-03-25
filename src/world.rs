//! 世界（World）是仿真系统的顶层容器
//!
//! 世界是仿真的整体环境，包含所有实体、域、时钟和事件通道。
//! 世界负责协调仿真循环的执行。

use crate::{
    DomainEvent, DomainRegistry, DomainRules, Entity, EntityId, EntityStore, EventChannel,
    Lifecycle, TimeClock, Timer, TimerCallback, TimerManager,
};

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

    /// 生成新的实体 ID
    pub fn generate_entity_id(&mut self) -> EntityId {
        let id = EntityId::new(self.next_entity_id);
        self.next_entity_id += 1;
        id
    }

    /// 注册域
    pub fn register_domain<T: DomainRules>(&mut self, name: &str, rules: T) {
        self.domains.register(name, rules);
    }

    /// 创建实体
    ///
    /// 创建实体后，尝试将其附加到声明的域。
    /// 返回实体 ID。
    pub fn spawn(&mut self, entity: Entity) -> EntityId {
        let id = entity.id;
        let declared_domains: Vec<String> = entity.domains.iter().cloned().collect();

        // 存储实体
        self.entities.insert(entity);

        // 尝试附加到各域
        for domain_name in declared_domains {
            if let Some(entity_ref) = self.entities.get(id) {
                if let Some(domain) = self.domains.get_by_name_mut(&domain_name) {
                    if domain.try_attach(entity_ref) {
                        // 附加成功
                    }
                    // 附加失败时域会拒绝实体
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
    /// 将实体标记为销毁中状态，设置销毁动画定时器。
    pub fn destroy(&mut self, entity_id: EntityId, destroy_time: f64) {
        if let Some(entity) = self.entities.get_mut(entity_id) {
            entity.lifecycle = Lifecycle::Destroying;

            // 设置销毁动画定时器
            self.timer_manager.schedule(
                entity_id,
                Timer::self_destruct(self.clock.sim_time + destroy_time),
            );
        }
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

    /// 执行一步仿真
    ///
    /// 执行完整的仿真循环：
    /// 1. 时间推进
    /// 2. 域计算
    /// 3. 定时器检查
    /// 4. 事件处理
    /// 5. 清理
    pub fn step(&mut self, real_dt: f64) {
        // 阶段 1：时间推进
        let dt = self.clock.tick(real_dt);
        let sim_time = self.clock.sim_time;

        // 如果暂停，跳过计算
        if dt == 0.0 {
            return;
        }

        // 阶段 2：域计算
        self.compute_domains(dt);

        // 阶段 3：定时器检查
        self.check_timers(sim_time);

        // 阶段 4：事件处理
        self.process_events();

        // 阶段 5：清理
        self.cleanup();
    }

    /// 域计算阶段
    fn compute_domains(&mut self, _dt: f64) {
        // 获取执行顺序
        let order: Vec<String> = self.domains.execution_order().to_vec();

        // 按顺序执行各域
        for domain_name in &order {
            // 使用 unsafe 或分割借用（这里使用简化方案）
            // 实际实现可能需要更复杂的借用管理

            // 先收集该域要发出的所有事件
            let mut domain_events = EventChannel::new();

            // 执行计算
            if let Some(domain) = self.domains.get_by_name_mut(domain_name) {
                // 获取域中的实体 ID 列表
                let _entity_ids: Vec<EntityId> = domain.entity_ids().collect();

                // 为每个实体执行计算（简化版本）
                // 实际实现中，这里会创建 DomainContext 并调用 rules.compute()

                // 这里需要更复杂的实现来处理借用问题
                // 当前为框架骨架
            }

            // 将域事件合并到主事件通道
            self.events.append(&mut domain_events);
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
    fn process_events(&mut self) {
        // 取出所有事件
        let events = std::mem::take(&mut self.events);

        for event in events {
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
                // 实体销毁事件
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
                // 自定义事件由用户处理器处理
                // 用户可以实现自己的事件处理器
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
                // 自毁：将实体标记为已销毁
                if let Some(entity) = self.entities.get_mut(entity_id) {
                    entity.lifecycle = Lifecycle::Destroyed;
                }
            }
            TimerCallback::Event(domain_event) => {
                // 发送事件：重新投递事件
                self.handle_event(*domain_event);
            }
            TimerCallback::Custom(_callback_id) => {
                // 自定义回调：由用户处理
            }
        }
    }

    /// 清理阶段
    fn cleanup(&mut self) {
        // 收集所有已销毁状态的实体
        let destroyed_ids: Vec<EntityId> = self
            .entities
            .iter()
            .filter(|e| e.lifecycle.is_destroyed())
            .map(|e| e.id)
            .collect();

        // 移除这些实体
        for id in destroyed_ids {
            self.remove_entity(id);
        }
    }

    /// 获取实体
    pub fn get_entity(&self, id: EntityId) -> Option<&Entity> {
        self.entities.get(id)
    }

    /// 获取可变实体
    pub fn get_entity_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        self.entities.get_mut(id)
    }

    /// 获取域
    pub fn get_domain<T: DomainRules>(&self) -> Option<&T> {
        self.domains.get::<T>()
    }

    /// 获取可变域
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
        let id = world.generate_entity_id();

        let entity = Entity::new(id, "ship");
        world.spawn(entity);

        assert_eq!(world.entity_count(), 1);
        assert!(world.get_entity(id).is_some());
    }

    #[test]
    fn test_time_advancement() {
        let mut world = World::new();
        world.step(0.1);

        assert_eq!(world.sim_time(), 0.1);
    }
}
