//! 世界构建器

use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::diagnostics::{LogSink, LoggerHandle};
use crate::domain::{AnyDomain, Domain};
use crate::entity::id::EntityAllocator;
use crate::event::{
    AnyObserver, AnyReaction, Event, EventBuffer, Observer, ObserverWrapper, Reaction,
    ReactionWrapper,
};
use crate::runtime::scheduler::{DomainInfo, Scheduler};
use crate::runtime::timers::{TimeClock, TimerManager};
use crate::storage::Storage;

use super::World;

// ──── WorldBuilder ────────────────────────────────────────────────────────

/// 世界构建器
///
/// 以流式 API 装配仿真世界：
/// - 配置：[`time_scale`](WorldBuilder::time_scale)、[`paused`](WorldBuilder::paused)、[`logger`](WorldBuilder::logger)
/// - 注册：[`domain`](WorldBuilder::domain)、[`on`](WorldBuilder::on)、[`observe`](WorldBuilder::observe)
/// - 模块化：[`apply`](WorldBuilder::apply)（接受 `fn(WorldBuilder) -> WorldBuilder`，强制子系统封装为独立函数）
///
/// # 示例
///
/// ```rust,ignore
/// // 最小写法
/// let world = World::builder()
///     .domain(GravityDomain)
///     .build();
///
/// // 带事件
/// let world = World::builder()
///     .domain(MotionDomain)
///     .on::<HitEvent>(HandleHit)
///     .observe::<BounceEvent>(LogBounce)
///     .build();
///
/// // 大型项目模块化装配
/// let world = World::builder()
///     .logger(Arc::new(MyLogger))
///     .domain(MotionDomain)
///     .apply(combat::install)
///     .apply(collision::install)
///     .build();
/// ```
pub struct WorldBuilder {
    pub(super) time_scale: f64,
    pub(super) paused: bool,
    pub(super) domains: Vec<Box<dyn AnyDomain>>,
    pub(super) reactions: HashMap<TypeId, Vec<Box<dyn AnyReaction>>>,
    pub(super) observers: HashMap<TypeId, Vec<Box<dyn AnyObserver>>>,
    pub(super) logger: LoggerHandle,
}

impl WorldBuilder {
    pub fn new() -> Self {
        Self {
            time_scale: 1.0,
            paused: false,
            domains: Vec::new(),
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

    /// 注入日志后端
    ///
    /// 未调用此方法时使用内置 `Logger`（Info 级别）。
    pub fn logger(mut self, logger: Arc<dyn LogSink>) -> Self {
        self.logger = LoggerHandle::new(logger);
        self
    }

    /// 注册一个仿真域
    ///
    /// 域的执行顺序由 `Domain::After` 关联类型在构建期静态分析决定，与注册顺序无关。
    pub fn domain<D: Domain + 'static>(mut self, domain: D) -> Self {
        self.domains.push(Box::new(domain));
        self
    }

    /// 注册反应器（可修改世界）
    ///
    /// 接受任何实现了 [`Reaction<E>`] 的具名结构体类型。
    /// 推荐将处理器集中到模块的 `install` 函数中，通过 [`.apply()`](WorldBuilder::apply) 装配。
    pub fn on<E: Event + 'static>(mut self, handler: impl Reaction<E> + 'static) -> Self {
        self.reactions
            .entry(TypeId::of::<E>())
            .or_default()
            .push(Box::new(ReactionWrapper {
                inner: handler,
                _phantom: PhantomData,
            }));
        self
    }

    /// 注册观察器（只读访问世界）
    ///
    /// 接受任何实现了 [`Observer<E>`] 的具名结构体类型。
    /// 推荐将处理器集中到模块的 `install` 函数中，通过 [`.apply()`](WorldBuilder::apply) 装配。
    pub fn observe<E: Event + 'static>(mut self, handler: impl Observer<E> + 'static) -> Self {
        self.observers
            .entry(TypeId::of::<E>())
            .or_default()
            .push(Box::new(ObserverWrapper {
                inner: handler,
                _phantom: PhantomData,
            }));
        self
    }

    /// 应用一个模块化装配函数
    ///
    /// 接受 `fn(WorldBuilder) -> WorldBuilder` 签名的函数，将注册逻辑委托给子系统模块。
    /// 这是大型项目组织代码的推荐方式——每个子系统封装为独立的 `install` 函数：
    ///
    /// ```rust,ignore
    /// // combat/mod.rs
    /// pub fn install(builder: WorldBuilder) -> WorldBuilder {
    ///     builder
    ///         .domain(CombatDomain)
    ///         .on::<HitEvent>(HandleHit)
    /// }
    ///
    /// // main.rs
    /// World::builder()
    ///     .apply(combat::install)
    ///     .apply(collision::install)
    ///     .build();
    /// ```
    pub fn apply(self, f: impl FnOnce(Self) -> Self) -> Self {
        f(self)
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
            storage: Storage::new(),
            entities: HashMap::new(),
            allocator: EntityAllocator::new(),
            domains: self.domains,
            scheduler,
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

// ──── 测试 ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::context::DomainContext;
    use crate::domain::Domain;
    use crate::event::{Event, Observer, Reaction};

    // ──── 测试桩 ──────────────────────────────────────────────────────────

    #[derive(Clone)]
    struct Pos;
    impl crate::Component for Pos {}
    impl crate::Reality for Pos {}

    struct NopDomain;
    impl Domain for NopDomain {
        type Writes = crate::component_set!(Pos);
        type Reads = crate::component_set!();
        type After = crate::domain_set!();
        fn compute(&mut self, _ctx: &mut DomainContext<Self>, _delta_time: f64) {}
    }

    struct NopEvent;
    impl Event for NopEvent {
        fn event_name(&self) -> &'static str {
            "nop"
        }
    }

    struct NopReaction;
    impl Reaction<NopEvent> for NopReaction {
        fn react(&mut self, _event: &NopEvent, _world: &mut crate::World) {}
    }

    struct NopObserver;
    impl Observer<NopEvent> for NopObserver {
        fn observe(&mut self, _event: &NopEvent, _world: &crate::World) {}
    }

    // ──── 帮助函数：仿照 combat::install 的签名 ────────────────────────────

    fn install_nop_domain(builder: WorldBuilder) -> WorldBuilder {
        builder.domain(NopDomain)
    }

    fn install_nop_handlers(builder: WorldBuilder) -> WorldBuilder {
        builder
            .on::<NopEvent>(NopReaction)
            .observe::<NopEvent>(NopObserver)
    }

    // ──── 测试 ────────────────────────────────────────────────────────────

    /// .apply(fn) 与直接链式调用结果等价
    #[test]
    fn test_apply_equivalent_to_chain() {
        let via_chain = WorldBuilder::new()
            .domain(NopDomain)
            .on::<NopEvent>(NopReaction)
            .observe::<NopEvent>(NopObserver)
            .build();

        let via_apply = WorldBuilder::new()
            .apply(install_nop_domain)
            .apply(install_nop_handlers)
            .build();

        assert_eq!(via_chain.entity_count(), via_apply.entity_count());
    }

    /// 多个 .apply() 可以依次组合，顺序不变
    #[test]
    fn test_apply_compose_multiple_modules() {
        let world = WorldBuilder::new()
            .apply(install_nop_domain)
            .apply(install_nop_handlers)
            .apply(install_nop_handlers) // 同一事件可注册多个处理器
            .build();

        // 两次 install_nop_handlers → 2 个 reaction + 2 个 observer
        let reaction_count = world
            .reactions
            .get(&std::any::TypeId::of::<NopEvent>())
            .map_or(0, |v| v.len());
        let observer_count = world
            .observers
            .get(&std::any::TypeId::of::<NopEvent>())
            .map_or(0, |v| v.len());
        assert_eq!(reaction_count, 2);
        assert_eq!(observer_count, 2);
    }

    /// .apply() 接受闭包工厂（捕获外部状态后返回 FnOnce）
    #[test]
    fn test_apply_closure_factory() {
        // 模拟 handlers::install(&simulation_output) 闭包工厂模式：
        // install_with_config 返回 impl FnOnce(WorldBuilder) -> WorldBuilder
        fn install_with_config(emit_value: u32) -> impl FnOnce(WorldBuilder) -> WorldBuilder {
            move |builder| {
                let _ = emit_value; // 捕获外部参数
                builder.domain(NopDomain)
            }
        }

        let world = WorldBuilder::new().apply(install_with_config(42)).build();

        assert_eq!(world.entity_count(), 0);
    }

    /// .apply() 自身是零成本抽象（f(self) 无堆分配）
    #[test]
    fn test_apply_identity() {
        let world = WorldBuilder::new().apply(|b| b).build();
        assert_eq!(world.entity_count(), 0);
    }
}
