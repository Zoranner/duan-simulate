//! 事件系统
//!
//! DUAN 事件模型以"事实 → 反应/观察"为核心：
//!
//! - [`Event`]：仿真中已发生的领域事实，纯数据，不承担副作用逻辑。
//! - 反应（[`Reaction<E>`]）：由 [`crate::WorldBuilder::events`] 中的 `on` 注册，
//!   接收特定事件并允许修改世界，处理仿真内副作用（生成实体、销毁实体、应用伤害等）。
//! - 观察（[`Observer<E>`]）：由 [`crate::WorldBuilder::events`] 中的 `observe` 注册，
//!   只读消费事件，用于统计、日志、测试采集。
//!
//! # 定义事件
//!
//! ```rust,ignore
//! use duan::Event;
//!
//! #[derive(Debug)]
//! pub struct HitEvent {
//!     pub target_id: duan::EntityId,
//!     pub damage: f64,
//! }
//!
//! impl Event for HitEvent {
//!     fn event_name(&self) -> &'static str { "hit" }
//! }
//! ```
//!
//! # 注册与消费
//!
//! ```rust,ignore
//! World::builder()
//!     .events(|e| {
//!         e.on::<HitEvent>(|ev: &HitEvent, world: &mut World| {
//!             world.destroy(ev.target_id);
//!         });
//!         e.observe::<HitEvent>(|ev: &HitEvent, _world: &World| {
//!             println!("命中！伤害 = {}", ev.damage);
//!         });
//!     })
//!     .build()
//! ```

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

// World 与 events 在同一 crate 内互相引用，Rust 模块系统允许这种跨模块循环引用。
use crate::runtime::world::World;

// ──── Event ──────────────────────────────────────────────────────────────

/// 领域事实 trait
///
/// 实现此 trait 的类型表示仿真中已发生的领域事实，是纯数据载体。
///
/// # 约束
///
/// - 不需要实现 `Clone`
/// - 框架内部通过 `TypeId` 类型化分发，无需手动 downcast
///
/// # 示例
///
/// ```rust,ignore
/// use duan::Event;
///
/// #[derive(Debug)]
/// pub struct FireEvent { pub shooter_id: duan::EntityId }
///
/// impl Event for FireEvent {
///     fn event_name(&self) -> &'static str { "fire" }
/// }
/// ```
pub trait Event: Send + Sync + 'static {
    /// 事件名称（用于调试和日志）
    fn event_name(&self) -> &'static str;
}

// ──── 内部：类型擦除事件节点 ──────────────────────────────────────────────

/// 类型擦除的事件节点（框架内部使用）
pub(crate) struct ArcEvent {
    pub(crate) type_id: TypeId,
    pub(crate) inner: Arc<dyn Any + Send + Sync>,
    pub(crate) name: &'static str,
}

// ──── EventBuffer ────────────────────────────────────────────────────────

/// 帧内事件缓冲区
///
/// 收集一帧内产生的所有事实事件，在帧末统一分发到反应器和观察器。
/// 框架内部使用，用户通过 [`crate::EntityContext::emit`] 或
/// [`crate::DomainContext::emit`] 发送事件。
pub struct EventBuffer {
    facts: Vec<ArcEvent>,
}

impl EventBuffer {
    pub fn new() -> Self {
        Self { facts: Vec::new() }
    }

    /// 发出一个领域事实（框架内部接口，供 Context 调用）
    pub(crate) fn emit<E: Event>(&mut self, event: E) {
        let name = event.event_name();
        self.facts.push(ArcEvent {
            type_id: TypeId::of::<E>(),
            inner: Arc::new(event),
            name,
        });
    }

    pub(crate) fn drain(&mut self) -> Vec<ArcEvent> {
        std::mem::take(&mut self.facts)
    }

    pub fn len(&self) -> usize {
        self.facts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.facts.is_empty()
    }
}

impl Default for EventBuffer {
    fn default() -> Self {
        Self::new()
    }
}

// ──── Reaction / Observer 公开 trait ────────────────────────────────────

/// 反应器 trait
///
/// 反应器接收特定类型的领域事实事件，并允许修改世界，处理仿真内副作用
/// （如生成导弹、销毁实体、应用伤害等）。
///
/// 可以用闭包直接实现此 trait，无需定义独立的结构体：
///
/// ```rust,ignore
/// World::builder()
///     .events(|e| {
///         e.on::<HitEvent>(|ev: &HitEvent, world: &mut World| {
///             world.destroy(ev.missile_id);
///         });
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
/// 观察器接收特定类型的领域事实事件，但不能修改世界，
/// 用于统计、日志、测试采集、回放数据录制等只读消费场景。
///
/// ```rust,ignore
/// World::builder()
///     .events(|e| {
///         e.observe::<HitEvent>(|ev: &HitEvent, world: &World| {
///             println!("命中！目标 = {:?}，伤害 = {}", ev.target_id, ev.damage);
///         });
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

pub(crate) struct ReactionWrapper<E: Event, R: Reaction<E>> {
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

pub(crate) struct ObserverWrapper<E: Event, O: Observer<E>> {
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

// ──── EventRegistrar ──────────────────────────────────────────────────────

/// 事件处理器注册器
///
/// 通过 [`crate::WorldBuilder::events`] 的闭包参数获取，用于批量注册反应器和观察器。
///
/// # 示例
///
/// ```rust,ignore
/// World::builder()
///     .events(|e| {
///         e.on::<FireEvent>(on_fire(&app));
///         e.observe::<HitEvent>(on_hit_log(&app));
///     })
///     .build();
/// ```
pub struct EventRegistrar {
    pub(crate) reactions: HashMap<TypeId, Vec<Box<dyn AnyReaction>>>,
    pub(crate) observers: HashMap<TypeId, Vec<Box<dyn AnyObserver>>>,
}

impl EventRegistrar {
    pub(crate) fn new() -> Self {
        Self {
            reactions: HashMap::new(),
            observers: HashMap::new(),
        }
    }

    /// 注册反应器：当 `E` 类型事件发生时执行，可修改世界
    pub fn on<E: Event>(&mut self, handler: impl Reaction<E>) -> &mut Self {
        self.reactions
            .entry(TypeId::of::<E>())
            .or_default()
            .push(Box::new(ReactionWrapper {
                inner: handler,
                _phantom: PhantomData,
            }));
        self
    }

    /// 注册观察器：当 `E` 类型事件发生时执行，只读访问世界
    pub fn observe<E: Event>(&mut self, handler: impl Observer<E>) -> &mut Self {
        self.observers
            .entry(TypeId::of::<E>())
            .or_default()
            .push(Box::new(ObserverWrapper {
                inner: handler,
                _phantom: PhantomData,
            }));
        self
    }
}

// ──── 测试 ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    struct PingEvent {
        value: u32,
    }

    impl Event for PingEvent {
        fn event_name(&self) -> &'static str {
            "ping"
        }
    }

    #[test]
    fn test_event_buffer_emit_and_drain() {
        let mut buf = EventBuffer::new();
        buf.emit(PingEvent { value: 42 });
        assert_eq!(buf.len(), 1);

        let events = buf.drain();
        assert_eq!(events.len(), 1);
        assert!(buf.is_empty());

        assert_eq!(events[0].name, "ping");
        assert_eq!(events[0].type_id, TypeId::of::<PingEvent>());

        let ping = events[0].inner.downcast_ref::<PingEvent>().unwrap();
        assert_eq!(ping.value, 42);
    }

    #[test]
    fn test_event_buffer_default_empty() {
        let buf = EventBuffer::default();
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
    }
}
