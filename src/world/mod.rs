//! 世界（World）
//!
//! 仿真系统的顶层容器，协调实体、域、时钟和事件的工作。

pub(crate) mod builder;
pub(crate) mod step;

pub use builder::WorldBuilder;

use std::any::TypeId;
use std::collections::HashMap;

use crate::diagnostics::{FramePhase, LogContext, LogLevel, LoggerHandle};
use crate::domain::AnyDomain;
use crate::entity::id::{EntityAllocator, EntityId};
use crate::entity::{
    dispatch_tick, ComponentBundle, Entity, EntityRecord, Lifecycle, PendingSpawn,
};
use crate::event::{AnyObserver, AnyReaction, EventBuffer};
use crate::runtime::scheduler::Scheduler;
use crate::runtime::timers::{TimeClock, Timer, TimerCallback, TimerManager};
use crate::storage::WorldStorage;

// ──── World ───────────────────────────────────────────────────────────────

/// 仿真世界
///
/// 顶层容器，驱动 5 阶段仿真循环。通过 [`World::builder()`] 构建。
pub struct World {
    pub(crate) clock: TimeClock,
    pub(crate) storage: WorldStorage,
    pub(crate) entities: HashMap<EntityId, EntityRecord>,
    pub(crate) allocator: EntityAllocator,
    pub(crate) domains: Vec<Box<dyn AnyDomain>>,
    pub(crate) scheduler: Scheduler,
    pub(crate) events: EventBuffer,
    pub(crate) timer_manager: TimerManager,
    pub(crate) reactions: HashMap<TypeId, Vec<Box<dyn AnyReaction>>>,
    pub(crate) observers: HashMap<TypeId, Vec<Box<dyn AnyObserver>>>,
    pub(crate) logger: LoggerHandle,
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
    pub fn spawn_with<E: Entity>(&mut self, extra: impl ComponentBundle) -> EntityId {
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
        self.timer_manager.remove_entity(id);
        self.timer_manager
            .schedule(id, Timer::self_destruct(self.clock.time + delay));
    }

    // ──── 查询 ──────────────────────────────────────────────────────────

    /// 读取实体的组件（只读）
    ///
    /// 主要用于宿主层（游戏循环、UI 展示、测试断言）读取实体状态。
    /// 域和实体的业务逻辑应优先使用各自的 Context API。
    pub fn get<T: crate::Component>(&self, id: EntityId) -> Option<&T> {
        self.storage.get::<T>(id)
    }

    /// 宿主侧检查实体组件（可变）
    ///
    /// 仅用于调试、测试和外部工具场景。
    /// 业务逻辑必须通过域（[`crate::DomainContext`]）或实体（[`crate::EntityContext`]）上下文修改状态。
    pub fn inspect_mut<T: crate::Component>(&mut self, id: EntityId) -> Option<&mut T> {
        self.storage.get_mut::<T>(id)
    }

    /// 检查实体是否存活
    pub fn is_alive(&self, id: EntityId) -> bool {
        self.entities
            .get(&id)
            .is_some_and(|r| r.lifecycle.is_alive())
    }

    /// 获取当前仿真时间（秒）
    pub fn time(&self) -> f64 {
        self.clock.time
    }

    /// 获取活跃实体数量
    pub fn entity_count(&self) -> usize {
        self.entities
            .values()
            .filter(|r| r.lifecycle.is_active())
            .count()
    }

    /// 获取日志句柄
    pub fn logger(&self) -> &LoggerHandle {
        &self.logger
    }

    /// 以指定上下文写日志（框架与业务统一入口）
    #[inline]
    pub fn emit_at(
        &self,
        level: LogLevel,
        phase: FramePhase,
        delta_time: f64,
        entity_id: Option<EntityId>,
        target: &str,
        message: &str,
    ) {
        self.logger.emit(
            level,
            LogContext::new(
                phase,
                self.clock.time,
                delta_time,
                self.clock.step_count,
                entity_id,
            ),
            target,
            message,
        );
    }

    /// 记录 Trace 级别日志（无特定阶段，`FramePhase::None`）
    #[inline]
    pub fn trace(&self, target: &str, message: &str) {
        self.emit_at(
            LogLevel::Trace,
            FramePhase::None,
            0.0,
            None,
            target,
            message,
        );
    }

    /// 记录 Debug 级别日志（无特定阶段，`FramePhase::None`）
    #[inline]
    pub fn debug(&self, target: &str, message: &str) {
        self.emit_at(
            LogLevel::Debug,
            FramePhase::None,
            0.0,
            None,
            target,
            message,
        );
    }

    /// 记录 Info 级别日志（无特定阶段，`FramePhase::None`）
    #[inline]
    pub fn info(&self, target: &str, message: &str) {
        self.emit_at(LogLevel::Info, FramePhase::None, 0.0, None, target, message);
    }

    /// 记录 Warn 级别日志（无特定阶段，`FramePhase::None`）
    #[inline]
    pub fn warn(&self, target: &str, message: &str) {
        self.emit_at(LogLevel::Warn, FramePhase::None, 0.0, None, target, message);
    }

    /// 记录 Error 级别日志（无特定阶段，`FramePhase::None`）
    #[inline]
    pub fn error(&self, target: &str, message: &str) {
        self.emit_at(
            LogLevel::Error,
            FramePhase::None,
            0.0,
            None,
            target,
            message,
        );
    }

    /// 记录 EventDispatch 阶段的 Trace 日志
    #[inline]
    pub fn event_trace(&self, target: &str, message: &str) {
        self.emit_at(
            LogLevel::Trace,
            FramePhase::EventDispatch,
            self.clock.current_delta_time,
            None,
            target,
            message,
        );
    }

    /// 记录 EventDispatch 阶段的 Debug 日志
    #[inline]
    pub fn event_debug(&self, target: &str, message: &str) {
        self.emit_at(
            LogLevel::Debug,
            FramePhase::EventDispatch,
            self.clock.current_delta_time,
            None,
            target,
            message,
        );
    }

    /// 记录 EventDispatch 阶段的 Info 日志
    #[inline]
    pub fn event_info(&self, target: &str, message: &str) {
        self.emit_at(
            LogLevel::Info,
            FramePhase::EventDispatch,
            self.clock.current_delta_time,
            None,
            target,
            message,
        );
    }

    /// 记录 EventDispatch 阶段并绑定实体的 Info 日志
    #[inline]
    pub fn event_info_for(&self, entity_id: EntityId, target: &str, message: &str) {
        self.emit_at(
            LogLevel::Info,
            FramePhase::EventDispatch,
            self.clock.current_delta_time,
            Some(entity_id),
            target,
            message,
        );
    }

    /// 记录 EventDispatch 阶段并绑定实体的 Debug 日志
    #[inline]
    pub fn event_debug_for(&self, entity_id: EntityId, target: &str, message: &str) {
        self.emit_at(
            LogLevel::Debug,
            FramePhase::EventDispatch,
            self.clock.current_delta_time,
            Some(entity_id),
            target,
            message,
        );
    }

    // ──── 仿真控制 ───────────────────────────────────────────────────────

    /// 执行一步仿真
    ///
    /// 运行完整的 5 阶段循环，在 Phase 4 将帧内事件分发到所有已注册的
    /// 反应器（[`crate::Reaction`]）和观察器（[`crate::Observer`]）。
    pub fn step(&mut self, delta_time: f64) {
        step::run(self, delta_time);
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
        let spawn_count = pending_spawns.len();
        let destroy_count = pending_destroys.len();

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

        if spawn_count > 0 || destroy_count > 0 {
            self.emit_at(
                LogLevel::Debug,
                FramePhase::StepEnd,
                0.0,
                None,
                "duan::lifecycle",
                &format!("flush_pending: spawned={spawn_count} destroyed={destroy_count}"),
            );
        }
    }

    pub(crate) fn cleanup_destroyed(&mut self) {
        let destroyed: Vec<EntityId> = self
            .entities
            .iter()
            .filter(|(_, r)| r.lifecycle == Lifecycle::Destroyed)
            .map(|(id, _)| *id)
            .collect();
        let count = destroyed.len();
        for id in destroyed {
            self.entities.remove(&id);
            self.storage.remove_entity(id);
        }
        if count > 0 {
            self.emit_at(
                LogLevel::Debug,
                FramePhase::StepEnd,
                0.0,
                None,
                "duan::lifecycle",
                &format!("cleanup_destroyed: removed={count}"),
            );
        }
    }

    pub(crate) fn handle_timer_events(&mut self) {
        let time = self.clock.time;
        let fired = self.timer_manager.check(time);

        let fired_count = fired.len();
        if fired_count > 0 {
            self.emit_at(
                LogLevel::Debug,
                FramePhase::TimerDispatch,
                0.0,
                None,
                "duan::timer",
                &format!("timer fired: count={fired_count}"),
            );
        }

        for evt in fired {
            match evt.callback {
                TimerCallback::SelfDestruct => {
                    if let Some(rec) = self.entities.get_mut(&evt.entity_id) {
                        rec.lifecycle = Lifecycle::Destroyed;
                    } else {
                        self.emit_at(
                            LogLevel::Warn,
                            FramePhase::TimerDispatch,
                            0.0,
                            Some(evt.entity_id),
                            "duan::timer",
                            &format!(
                                "SelfDestruct timer fired but entity {} not found",
                                evt.entity_id
                            ),
                        );
                    }
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
    use crate::domain::Domain;
    use crate::event::Event;

    #[test]
    fn test_world_creation() {
        let world = World::new();
        assert_eq!(world.entity_count(), 0);
        assert_eq!(world.time(), 0.0);
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

    // ──── 事件系统测试 ────────────────────────────────────────────────────

    struct TestEvent {
        pub value: u32,
    }
    impl Event for TestEvent {
        fn event_name(&self) -> &'static str {
            "test"
        }
    }

    struct EmitDomain {
        pub emit_value: u32,
    }
    impl Domain for EmitDomain {
        type Writes = ();
        type Reads = ();
        type After = ();
        fn compute(
            &mut self,
            ctx: &mut crate::domain::context::DomainContext<Self>,
            _delta_time: f64,
        ) {
            ctx.emit(TestEvent {
                value: self.emit_value,
            });
        }
    }

    // ──── 测试用具名处理器（框架内部测试的标准写法）─────────────────────────

    use std::sync::{Arc, Mutex};

    struct CollectReaction {
        collected: Arc<Mutex<Vec<u32>>>,
    }
    impl crate::event::Reaction<TestEvent> for CollectReaction {
        fn react(&mut self, ev: &TestEvent, _world: &mut World) {
            self.collected.lock().unwrap().push(ev.value);
        }
    }

    struct CollectObserver {
        collected: Arc<Mutex<Vec<u32>>>,
    }
    impl crate::event::Observer<TestEvent> for CollectObserver {
        fn observe(&mut self, ev: &TestEvent, _world: &World) {
            self.collected.lock().unwrap().push(ev.value);
        }
    }

    struct LogReaction {
        log: Arc<Mutex<Vec<&'static str>>>,
        label: &'static str,
    }
    impl crate::event::Reaction<TestEvent> for LogReaction {
        fn react(&mut self, _ev: &TestEvent, _world: &mut World) {
            self.log.lock().unwrap().push(self.label);
        }
    }

    struct LogObserver {
        log: Arc<Mutex<Vec<&'static str>>>,
        label: &'static str,
    }
    impl crate::event::Observer<TestEvent> for LogObserver {
        fn observe(&mut self, _ev: &TestEvent, _world: &World) {
            self.log.lock().unwrap().push(self.label);
        }
    }

    #[test]
    fn test_reaction_receives_event() {
        let received = Arc::new(Mutex::new(Vec::<u32>::new()));

        let mut world = World::builder()
            .domain(EmitDomain { emit_value: 42 })
            .on::<TestEvent>(CollectReaction {
                collected: Arc::clone(&received),
            })
            .build();

        world.step(0.016);
        assert_eq!(*received.lock().unwrap(), vec![42]);
    }

    #[test]
    fn test_observer_receives_event() {
        let observed = Arc::new(Mutex::new(Vec::<u32>::new()));

        let mut world = World::builder()
            .domain(EmitDomain { emit_value: 7 })
            .observe::<TestEvent>(CollectObserver {
                collected: Arc::clone(&observed),
            })
            .build();

        world.step(0.016);
        assert_eq!(*observed.lock().unwrap(), vec![7]);
    }

    #[test]
    fn test_multiple_handlers_same_event() {
        let log = Arc::new(Mutex::new(Vec::<&'static str>::new()));

        let mut world = World::builder()
            .domain(EmitDomain { emit_value: 1 })
            .on::<TestEvent>(LogReaction {
                log: Arc::clone(&log),
                label: "reaction_1",
            })
            .on::<TestEvent>(LogReaction {
                log: Arc::clone(&log),
                label: "reaction_2",
            })
            .observe::<TestEvent>(LogObserver {
                log: Arc::clone(&log),
                label: "observer_1",
            })
            .build();

        world.step(0.016);
        let entries = log.lock().unwrap();
        assert_eq!(*entries, vec!["reaction_1", "reaction_2", "observer_1"]);
    }
}
