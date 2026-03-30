//! 世界（World）
//!
//! 仿真系统的顶层容器，协调实体、域、时钟和事件的工作。

pub mod step;

use crate::component::storage::WorldStorage;
use crate::domain::AnyDomain;
use crate::entity::id::{EntityAllocator, EntityId};
use crate::entity::{
    dispatch_tick, ComponentBundle, Entity, EntityRecord, Lifecycle, PendingSpawn,
};
use crate::events::{CustomEvent, EventBuffer, TimerCallback};
use crate::scheduler::{DomainInfo, Scheduler};
use crate::time::{TimeClock, Timer, TimerManager};
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

// ──── WorldBuilder ────────────────────────────────────────────────────────

/// 世界构建器
///
/// # 示例
///
/// ```rust,ignore
/// let world = World::builder()
///     .with_domain(MotionDomain { gravity: 9.81 })
///     .with_domain(CollisionDomain::new())
///     .build();
/// ```
pub struct WorldBuilder {
    time_scale: f64,
    paused: bool,
    /// 待注册的域（保留完整类型信息，延迟到 build 时构建调度器）
    domains: Vec<Box<dyn AnyDomain>>,
    /// 已注册**认知**（`Memory`）组件的 TypeId（用于快照排除）
    memory_type_ids: Vec<TypeId>,
}

impl WorldBuilder {
    pub fn new() -> Self {
        Self {
            time_scale: 1.0,
            paused: false,
            domains: Vec::new(),
            memory_type_ids: Vec::new(),
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

    /// 注册域
    ///
    /// 域的类型决定调度顺序（通过 `Domain::After` 关联类型）。
    /// 同类型域只能注册一次（重复注册在 `build()` 时检测）。
    pub fn with_domain<D: crate::domain::Domain>(mut self, domain: D) -> Self {
        self.domains.push(Box::new(domain));
        self
    }

    /// 声明**认知**（`Memory`）组件类型（用于从快照中排除）
    ///
    /// 所有认知类型必须在此声明，否则会被包含到 `WorldSnapshot`，
    /// 导致其他实体可以访问本应封闭的认知数据。
    pub fn with_memory_type<T: crate::component::Memory>(mut self) -> Self {
        self.memory_type_ids.push(TypeId::of::<T>());
        self
    }

    /// 构建世界
    ///
    /// - 执行调度器静态分析（写入冲突、循环依赖检测）
    /// - 若存在问题立即 panic，使错误在配置阶段暴露而非运行时
    pub fn build(self) -> World {
        // 构建调度信息
        let infos: Vec<DomainInfo> = self
            .domains
            .iter()
            .map(|d| DomainInfo {
                type_id: d.type_id(),
                writes: d.writes_type_ids(),
                after: d.after_type_ids(),
            })
            .collect();

        let scheduler = Scheduler::build(&infos);

        let clock = if self.paused {
            let mut c = TimeClock::paused();
            c.time_scale = self.time_scale;
            c
        } else {
            TimeClock::with_scale(self.time_scale)
        };

        World {
            clock,
            storage: WorldStorage::new(),
            entities: HashMap::new(),
            allocator: EntityAllocator::new(),
            domains: self.domains,
            scheduler,
            memory_type_ids: self.memory_type_ids,
            events: EventBuffer::new(),
            timer_manager: TimerManager::new(),
        }
    }
}

impl Default for WorldBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ──── World ───────────────────────────────────────────────────────────────

/// 仿真世界
///
/// 顶层容器，驱动 5 阶段仿真循环。
pub struct World {
    /// 仿真时钟
    pub clock: TimeClock,
    /// 组件存储（按类型密集存储）
    pub(crate) storage: WorldStorage,
    /// 实体注册表
    pub(crate) entities: HashMap<EntityId, EntityRecord>,
    /// EntityId 分配器
    pub(crate) allocator: EntityAllocator,
    /// 所有注册域（`Vec<Box<dyn AnyDomain>>`）
    pub(crate) domains: Vec<Box<dyn AnyDomain>>,
    /// 执行计划（拓扑排序后的域索引）
    pub(crate) scheduler: Scheduler,
    /// **认知**（`Memory`）组件 TypeId 列表（构建快照时排除）
    pub(crate) memory_type_ids: Vec<TypeId>,
    /// 帧内事件缓冲
    pub(crate) events: EventBuffer,
    /// 定时器管理器
    pub(crate) timer_manager: TimerManager,
}

impl World {
    pub fn new() -> Self {
        WorldBuilder::new().build()
    }

    pub fn builder() -> WorldBuilder {
        WorldBuilder::new()
    }

    // ──── Spawn / Destroy ────────────────────────────────────────────────

    /// 生成实体
    ///
    /// 立即分配 EntityId，应用 Bundle 写入组件，注册到实体表。
    pub fn spawn<E: Entity>(&mut self) -> EntityId {
        let id = self.allocator.allocate();
        let bundle = E::bundle();
        bundle.apply(id, &mut self.storage);
        self.entities.insert(
            id,
            EntityRecord {
                id,
                lifecycle: Lifecycle::Active,
                tick_fn: dispatch_tick::<E>,
            },
        );
        id
    }

    /// 生成实体并附加额外组件（覆盖 bundle 中的同类组件）
    pub fn spawn_with<E: Entity>(
        &mut self,
        extra: impl crate::entity::ComponentBundle,
    ) -> EntityId {
        let id = self.spawn::<E>();
        extra.apply(id, &mut self.storage);
        id
    }

    /// 销毁实体（立即执行）
    ///
    /// 实体组件从存储移除，定时器取消，实体记录标记为 Destroyed。
    pub fn destroy(&mut self, id: EntityId) {
        if let Some(rec) = self.entities.get_mut(&id) {
            rec.lifecycle = Lifecycle::Destroying;
        } else {
            return;
        }
        self.storage.remove_entity(id);
        self.timer_manager.remove_entity(id);
        if let Some(rec) = self.entities.get_mut(&id) {
            rec.lifecycle = Lifecycle::Destroyed;
        }
    }

    /// 销毁实体（带过渡期，过渡期结束后移除）
    pub fn destroy_with_delay(&mut self, id: EntityId, delay: f64) {
        if let Some(rec) = self.entities.get_mut(&id) {
            if rec.lifecycle != Lifecycle::Active {
                return;
            }
            rec.lifecycle = Lifecycle::Destroying;
        } else {
            return;
        }
        // 组件暂不移除（过渡期可能还需要读取位置等数据）
        self.timer_manager.remove_entity(id);
        self.timer_manager
            .schedule(id, Timer::self_destruct(self.clock.sim_time + delay));
    }

    // ──── 查询 ──────────────────────────────────────────────────────────

    /// 读取实体的组件（只读）
    pub fn get<T: crate::component::Component>(&self, id: EntityId) -> Option<&T> {
        self.storage.get::<T>(id)
    }

    /// 读取实体的组件（可变）
    pub fn get_mut<T: crate::component::Component>(&mut self, id: EntityId) -> Option<&mut T> {
        self.storage.get_mut::<T>(id)
    }

    /// 检查实体是否存活
    pub fn is_alive(&self, id: EntityId) -> bool {
        self.entities
            .get(&id)
            .is_some_and(|r| r.lifecycle.is_alive())
    }

    /// 获取当前仿真时间
    pub fn sim_time(&self) -> f64 {
        self.clock.sim_time
    }

    /// 获取活跃实体数量
    pub fn entity_count(&self) -> usize {
        self.entities
            .values()
            .filter(|r| r.lifecycle.is_active())
            .count()
    }

    // ──── 仿真控制 ───────────────────────────────────────────────────────

    /// 执行一步仿真（无自定义事件处理）
    pub fn step(&mut self, dt: f64) {
        step::run(self, dt, &mut |_, _| {});
    }

    /// 执行一步仿真（带自定义事件回调）
    pub fn step_with<F>(&mut self, dt: f64, mut handler: F)
    where
        F: FnMut(&(dyn CustomEvent + 'static), &mut Self),
    {
        step::run(self, dt, &mut handler);
    }

    /// 执行一步仿真，收集并返回本帧所有自定义事件
    pub fn step_collect(&mut self, dt: f64) -> Vec<Arc<dyn CustomEvent + 'static>> {
        step::run_collect(self, dt)
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

    pub fn is_paused(&self) -> bool {
        self.clock.is_paused()
    }

    // ──── 定时器 ─────────────────────────────────────────────────────────

    /// 为实体调度定时器
    pub fn schedule_timer(&mut self, entity_id: EntityId, timer: Timer) {
        self.timer_manager.schedule(entity_id, timer);
    }

    /// 取消实体的定时器
    pub fn cancel_timer(&mut self, entity_id: EntityId, timer_id: &str) {
        self.timer_manager.cancel(entity_id, timer_id);
    }

    // ──── 内部：处理 pending 操作 ────────────────────────────────────────

    pub(crate) fn flush_pending(
        &mut self,
        pending_spawns: Vec<PendingSpawn>,
        pending_destroys: Vec<EntityId>,
    ) {
        for ps in pending_spawns {
            let id = self.allocator.allocate();
            (ps.apply_fn)(id, &mut self.storage);
            self.entities.insert(
                id,
                EntityRecord {
                    id,
                    lifecycle: Lifecycle::Active,
                    tick_fn: ps.tick_fn,
                },
            );
        }

        for id in pending_destroys {
            self.destroy(id);
        }
    }

    pub(crate) fn cleanup_destroyed(&mut self) {
        let destroyed: Vec<EntityId> = self
            .entities
            .iter()
            .filter(|(_, r)| r.lifecycle == Lifecycle::Destroyed)
            .map(|(id, _)| *id)
            .collect();
        for id in destroyed {
            self.entities.remove(&id);
            self.storage.remove_entity(id);
        }
    }

    pub(crate) fn handle_timer_events(&mut self) {
        let sim_time = self.clock.sim_time;
        let fired = self.timer_manager.check(sim_time);

        for evt in fired {
            match evt.callback {
                TimerCallback::SelfDestruct => {
                    if let Some(rec) = self.entities.get_mut(&evt.entity_id) {
                        rec.lifecycle = Lifecycle::Destroyed;
                    }
                }
                TimerCallback::Event(inner) => {
                    self.events.push(*inner);
                }
            }
        }
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

    struct Dummy;
    impl Entity for Dummy {}

    #[test]
    fn test_spawn_and_destroy() {
        let mut world = World::new();
        let id = world.spawn::<Dummy>();
        assert!(world.is_alive(id));
        assert_eq!(world.entity_count(), 1);

        world.destroy(id);
        assert!(!world.is_alive(id));
    }
}
