//! 世界（World）
//!
//! 仿真系统的顶层容器，协调实体、域、时钟和事件的工作。

pub mod step;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::component::storage::WorldStorage;
use crate::domain::AnyDomain;
use crate::entity::id::{EntityAllocator, EntityId};
use crate::entity::{
    dispatch_tick, ComponentBundle, Entity, EntityRecord, Lifecycle, PendingSpawn,
};
use crate::events::{Event, EventBuffer, TimerCallback};
use crate::logging::{FramePhase, LogContext, LogLevel, LogSink, LoggerHandle};
use crate::scheduler::{DomainInfo, Scheduler};
use crate::time::{TimeClock, Timer, TimerManager};

// ──── Reaction / Observer 公开 trait ────────────────────────────────────

/// 反应器 trait
///
/// 反应器接收特定类型的领域事实事件，并允许修改世界。
/// 用于处理仿真内副作用，例如生成导弹、销毁实体、应用伤害等。
///
/// 可以用闭包直接实现此 trait，无需定义独立的结构体：
///
/// ```rust,ignore
/// World::builder()
///     .with_reaction::<HitEvent, _>(|e: &HitEvent, world: &mut World| {
///         world.destroy(e.missile_id);
///     })
///     .build()
/// ```
///
/// 也可以为自定义结构体实现：
///
/// ```rust,ignore
/// struct DestroyMissileReaction;
/// impl Reaction<HitEvent> for DestroyMissileReaction {
///     fn react(&mut self, event: &HitEvent, world: &mut World) {
///         world.destroy(event.missile_id);
///     }
/// }
/// ```
pub trait Reaction<E: Event>: Send + Sync + 'static {
    fn react(&mut self, event: &E, world: &mut World);
}

/// 观察器 trait
///
/// 观察器接收特定类型的领域事实事件，但不能修改世界。
/// 用于统计、日志、测试采集、回放数据录制等只读消费场景。
///
/// ```rust,ignore
/// World::builder()
///     .with_observer::<HitEvent, _>(|e: &HitEvent, world: &World| {
///         println!("命中！目标 = {:?}，伤害 = {}", e.target_id, e.damage);
///     })
///     .build()
/// ```
pub trait Observer<E: Event>: Send + Sync + 'static {
    fn observe(&mut self, event: &E, world: &World);
}

// ──── 闭包 blanket impl ──────────────────────────────────────────────────

impl<E, F> Reaction<E> for F
where
    E: Event,
    F: FnMut(&E, &mut World) + Send + Sync + 'static,
{
    fn react(&mut self, event: &E, world: &mut World) {
        self(event, world);
    }
}

impl<E, F> Observer<E> for F
where
    E: Event,
    F: FnMut(&E, &World) + Send + Sync + 'static,
{
    fn observe(&mut self, event: &E, world: &World) {
        self(event, world);
    }
}

// ──── 类型擦除内部接口 ────────────────────────────────────────────────────

pub(crate) trait AnyReaction: Send + Sync {
    fn react_dyn(&mut self, event: &(dyn Any + Send + Sync), world: &mut World);
}

pub(crate) trait AnyObserver: Send + Sync {
    fn observe_dyn(&mut self, event: &(dyn Any + Send + Sync), world: &World);
}

struct ReactionWrapper<E: Event, R: Reaction<E>> {
    inner: R,
    _phantom: PhantomData<fn() -> E>,
}

impl<E: Event, R: Reaction<E>> AnyReaction for ReactionWrapper<E, R> {
    fn react_dyn(&mut self, event: &(dyn Any + Send + Sync), world: &mut World) {
        if let Some(e) = event.downcast_ref::<E>() {
            self.inner.react(e, world);
        }
    }
}

struct ObserverWrapper<E: Event, O: Observer<E>> {
    inner: O,
    _phantom: PhantomData<fn() -> E>,
}

impl<E: Event, O: Observer<E>> AnyObserver for ObserverWrapper<E, O> {
    fn observe_dyn(&mut self, event: &(dyn Any + Send + Sync), world: &World) {
        if let Some(e) = event.downcast_ref::<E>() {
            self.inner.observe(e, world);
        }
    }
}

// ──── WorldBuilder ────────────────────────────────────────────────────────

/// 世界构建器
///
/// # 示例
///
/// ```rust,ignore
/// let world = World::builder()
///     .with_domain(MotionDomain { gravity: 9.81 })
///     .with_domain(CollisionDomain::new())
///     .with_reaction::<HitEvent, _>(|e: &HitEvent, world: &mut World| {
///         world.destroy(e.missile_id);
///     })
///     .with_observer::<HitEvent, _>(|e: &HitEvent, _world: &World| {
///         println!("命中！");
///     })
///     .build();
/// ```
pub struct WorldBuilder {
    time_scale: f64,
    paused: bool,
    /// 待注册的域（保留完整类型信息，延迟到 build 时构建调度器）
    domains: Vec<Box<dyn AnyDomain>>,
    /// 已注册**认知**（`Memory`）组件的 TypeId（用于快照排除）
    memory_type_ids: Vec<TypeId>,
    /// 按事件 TypeId 分组的反应器注册表
    reactions: HashMap<TypeId, Vec<Box<dyn AnyReaction>>>,
    /// 按事件 TypeId 分组的观察器注册表
    observers: HashMap<TypeId, Vec<Box<dyn AnyObserver>>>,
    /// 日志句柄（默认内置 Logger）
    logger: LoggerHandle,
}

impl WorldBuilder {
    pub fn new() -> Self {
        Self {
            time_scale: 1.0,
            paused: false,
            domains: Vec::new(),
            memory_type_ids: Vec::new(),
            reactions: HashMap::new(),
            observers: HashMap::new(),
            logger: LoggerHandle::default_logger(),
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

    /// 注册反应器
    ///
    /// 反应器在每帧接收到 `E` 类型事件时被调用，可修改世界。
    /// 同一事件类型可注册多个反应器，按注册顺序依次执行。
    ///
    /// 接受任何实现了 [`Reaction<E>`] 的类型，包括闭包：
    ///
    /// ```rust,ignore
    /// .with_reaction::<HitEvent, _>(|e: &HitEvent, world: &mut World| {
    ///     world.destroy(e.missile_id);
    /// })
    /// ```
    pub fn with_reaction<E: Event, R: Reaction<E>>(mut self, reaction: R) -> Self {
        self.reactions
            .entry(TypeId::of::<E>())
            .or_default()
            .push(Box::new(ReactionWrapper {
                inner: reaction,
                _phantom: PhantomData,
            }));
        self
    }

    /// 注册观察器
    ///
    /// 观察器在每帧接收到 `E` 类型事件时被调用，只读消费事件，不能修改世界。
    /// 同一事件类型可注册多个观察器，按注册顺序依次执行。
    ///
    /// 接受任何实现了 [`Observer<E>`] 的类型，包括闭包：
    ///
    /// ```rust,ignore
    /// .with_observer::<HitEvent, _>(|e: &HitEvent, world: &World| {
    ///     println!("命中！目标 HP = {}", world.get::<Health>(e.target_id).map_or(0.0, |h| h.current));
    /// })
    /// ```
    pub fn with_observer<E: Event, O: Observer<E>>(mut self, observer: O) -> Self {
        self.observers
            .entry(TypeId::of::<E>())
            .or_default()
            .push(Box::new(ObserverWrapper {
                inner: observer,
                _phantom: PhantomData,
            }));
        self
    }

    /// 注入日志后端
    ///
    /// 未调用此方法时使用内置 `Logger`（Info 级别）。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use duan::{World, logging::{LogSink, LogRecord}};
    /// use std::sync::Arc;
    ///
    /// struct PrintLogger;
    /// impl LogSink for PrintLogger {
    ///     fn log(&self, r: &LogRecord) {
    ///         eprintln!("t={:.3} [{}] {}", r.sim_time, r.level, r.message);
    ///     }
    /// }
    ///
    /// let world = World::builder()
    ///     .with_logger(Arc::new(PrintLogger))
    ///     .build();
    /// ```
    pub fn with_logger(mut self, logger: Arc<dyn LogSink>) -> Self {
        self.logger = LoggerHandle::new(logger);
        self
    }

    /// 构建世界
    ///
    /// - 执行调度器静态分析（写入冲突、循环依赖检测）
    /// - 若存在问题立即 panic，使错误在配置阶段暴露而非运行时
    pub fn build(self) -> World {
        let infos: Vec<DomainInfo> = self
            .domains
            .iter()
            .map(|d| DomainInfo {
                type_id: d.get_type_id(),
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
            reactions: self.reactions,
            observers: self.observers,
            logger: self.logger,
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
    /// 按事件 TypeId 分组的反应器注册表
    pub(crate) reactions: HashMap<TypeId, Vec<Box<dyn AnyReaction>>>,
    /// 按事件 TypeId 分组的观察器注册表
    pub(crate) observers: HashMap<TypeId, Vec<Box<dyn AnyObserver>>>,
    /// 日志句柄（默认内置 Logger）
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
        dt: f64,
        entity_id: Option<EntityId>,
        target: &str,
        message: &str,
    ) {
        self.logger.emit(
            level,
            LogContext::new(
                phase,
                self.clock.sim_time,
                dt,
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
    pub fn event_trace(&self, dt: f64, target: &str, message: &str) {
        self.emit_at(
            LogLevel::Trace,
            FramePhase::EventDispatch,
            dt,
            None,
            target,
            message,
        );
    }

    /// 记录 EventDispatch 阶段的 Debug 日志
    #[inline]
    pub fn event_debug(&self, dt: f64, target: &str, message: &str) {
        self.emit_at(
            LogLevel::Debug,
            FramePhase::EventDispatch,
            dt,
            None,
            target,
            message,
        );
    }

    /// 记录 EventDispatch 阶段的 Info 日志
    #[inline]
    pub fn event_info(&self, dt: f64, target: &str, message: &str) {
        self.emit_at(
            LogLevel::Info,
            FramePhase::EventDispatch,
            dt,
            None,
            target,
            message,
        );
    }

    /// 记录 EventDispatch 阶段并绑定实体的 Info 日志
    #[inline]
    pub fn event_info_for(&self, dt: f64, entity_id: EntityId, target: &str, message: &str) {
        self.emit_at(
            LogLevel::Info,
            FramePhase::EventDispatch,
            dt,
            Some(entity_id),
            target,
            message,
        );
    }

    /// 记录 EventDispatch 阶段并绑定实体的 Debug 日志
    #[inline]
    pub fn event_debug_for(&self, dt: f64, entity_id: EntityId, target: &str, message: &str) {
        self.emit_at(
            LogLevel::Debug,
            FramePhase::EventDispatch,
            dt,
            Some(entity_id),
            target,
            message,
        );
    }

    // ──── 仿真控制 ───────────────────────────────────────────────────────

    /// 执行一步仿真
    ///
    /// 运行完整的 5 阶段循环，并在 Phase 4 将帧内事件分发到所有已注册的
    /// 反应器（[`Reaction<E>`]）和观察器（[`Observer<E>`]）。
    pub fn step(&mut self, dt: f64) {
        step::run(self, dt);
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
        let sim_time = self.clock.sim_time;
        let fired = self.timer_manager.check(sim_time);

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

    // ──── 事件系统测试 ────────────────────────────────────────────────────

    use crate::domain::Domain;

    struct TestEvent {
        pub value: u32,
    }
    impl crate::events::Event for TestEvent {
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
        fn compute(&mut self, ctx: &mut crate::domain::context::DomainContext<Self>, _dt: f64) {
            ctx.emit(TestEvent {
                value: self.emit_value,
            });
        }
    }

    #[test]
    fn test_reaction_receives_event() {
        use std::sync::{Arc, Mutex};
        let received = Arc::new(Mutex::new(Vec::<u32>::new()));
        let recv_clone = Arc::clone(&received);

        let mut world = World::builder()
            .with_domain(EmitDomain { emit_value: 42 })
            .with_reaction::<TestEvent, _>(move |e: &TestEvent, _world: &mut World| {
                recv_clone.lock().unwrap().push(e.value);
            })
            .build();

        world.step(0.016);
        assert_eq!(*received.lock().unwrap(), vec![42]);
    }

    #[test]
    fn test_observer_receives_event() {
        use std::sync::{Arc, Mutex};
        let observed = Arc::new(Mutex::new(Vec::<u32>::new()));
        let obs_clone = Arc::clone(&observed);

        let mut world = World::builder()
            .with_domain(EmitDomain { emit_value: 7 })
            .with_observer::<TestEvent, _>(move |e: &TestEvent, _world: &World| {
                obs_clone.lock().unwrap().push(e.value);
            })
            .build();

        world.step(0.016);
        assert_eq!(*observed.lock().unwrap(), vec![7]);
    }

    #[test]
    fn test_multiple_handlers_same_event() {
        use std::sync::{Arc, Mutex};
        let log = Arc::new(Mutex::new(Vec::<&'static str>::new()));
        let l1 = Arc::clone(&log);
        let l2 = Arc::clone(&log);
        let l3 = Arc::clone(&log);

        let mut world = World::builder()
            .with_domain(EmitDomain { emit_value: 1 })
            .with_reaction::<TestEvent, _>(move |_e: &TestEvent, _w: &mut World| {
                l1.lock().unwrap().push("reaction_1");
            })
            .with_reaction::<TestEvent, _>(move |_e: &TestEvent, _w: &mut World| {
                l2.lock().unwrap().push("reaction_2");
            })
            .with_observer::<TestEvent, _>(move |_e: &TestEvent, _w: &World| {
                l3.lock().unwrap().push("observer_1");
            })
            .build();

        world.step(0.016);
        let entries = log.lock().unwrap();
        assert_eq!(*entries, vec!["reaction_1", "reaction_2", "observer_1"]);
    }
}
