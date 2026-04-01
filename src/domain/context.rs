//! 域上下文
//!
//! [`DomainContext<D>`] 是域 `compute()` 的唯一数据入口，提供：
//! - 类型安全的组件读取（仅限 `D::Reads` 中声明的意图/事实类型）
//! - 类型安全的组件写入（仅限 `D::Writes` 中声明的事实类型）
//! - 事件发送、生命周期命令
//! - 带仿真上下文的统一日志接口

use super::Domain;
use crate::diagnostics::{FramePhase, LogContext, LoggerHandle};
use crate::entity::id::EntityId;
use crate::entity::{Entity, PendingSpawn};
use crate::event::{Event, EventBuffer};
use crate::runtime::timers::TimeClock;
use crate::snapshot::Snapshot;
use crate::storage::Storage;
use crate::{Component, ComponentSet};
use std::any::TypeId;
use std::marker::PhantomData;

/// 域计算上下文
///
/// Phase 3 中框架为每个域创建此上下文。
/// 类型参数 `D` 使编译器能在调用 `get`/`get_mut`/`each` 时验证约束：
///
/// - `get<T: InReads<D>>` / `each<T: InReads<D>>` → 从快照读取（上帧值）
/// - `get_mut<T: InWrites<D>>` → 写入活跃存储（当前帧）
pub struct DomainContext<'w, D: Domain> {
    /// 当前帧活跃存储（仅写 Writes 类型）
    pub(crate) storage: &'w mut Storage,
    /// 上帧快照（仅读 Reads 类型）
    pub(crate) snapshot: &'w Snapshot,
    pub(crate) pending_spawns: &'w mut Vec<PendingSpawn>,
    pub(crate) pending_destroys: &'w mut Vec<EntityId>,
    pub(crate) events: &'w mut EventBuffer,
    /// 仿真时钟（只读）
    pub(crate) clock: &'w TimeClock,
    /// 日志句柄
    pub(crate) logger: &'w LoggerHandle,
    /// 当前帧时间步长（秒）
    pub delta_time: f64,
    pub(crate) _phantom: PhantomData<D>,
}

impl<'w, D: Domain> DomainContext<'w, D> {
    #[inline]
    fn assert_read_declared<T: Component>() {
        let type_id = TypeId::of::<T>();
        if !D::Reads::type_ids().contains(&type_id) {
            panic!(
                "Domain `{}` read undeclared component `{}`; add it to `type Reads`",
                std::any::type_name::<D>(),
                std::any::type_name::<T>()
            );
        }
    }

    #[inline]
    fn assert_write_declared<T: Component>() {
        let type_id = TypeId::of::<T>();
        if !D::Writes::type_ids().contains(&type_id) {
            panic!(
                "Domain `{}` write undeclared component `{}`; add it to `type Writes`",
                std::any::type_name::<D>(),
                std::any::type_name::<T>()
            );
        }
    }

    // ──── 从快照读取（只读，上帧值）────────────────────────────────────

    /// 遍历快照中所有拥有组件 T 的实体（只读，上帧值）
    ///
    /// 读取的是意图或事实（`Intent` / `Reality`）的上帧快照，不受本帧其他域写入影响。
    /// 若 `T` 未在 `D::Reads` 中声明，会在运行时 panic。
    pub fn each<T: Component>(&self) -> impl Iterator<Item = (EntityId, &T)> {
        Self::assert_read_declared::<T>();
        self.snapshot.iter::<T>()
    }

    /// 从快照读取指定实体的组件 T（只读，上帧值）
    ///
    /// 若 `T` 未在 `D::Reads` 中声明，会在运行时 panic。
    pub fn get<T: Component>(&self, id: EntityId) -> Option<&T> {
        Self::assert_read_declared::<T>();
        self.snapshot.get::<T>(id)
    }

    /// 遍历快照中拥有组件 T 的所有 EntityId
    pub fn entities<T: Component>(&self) -> impl Iterator<Item = EntityId> + '_ {
        Self::assert_read_declared::<T>();
        self.snapshot.iter::<T>().map(|(id, _)| id)
    }

    // ──── 写入活跃存储（可变，当前帧）──────────────────────────────────

    /// 获取指定实体组件 T 的可变引用（写入当前帧）
    ///
    /// 若 `T` 未在 `D::Writes` 中声明，会在运行时 panic。
    pub fn get_mut<T: Component>(&mut self, id: EntityId) -> Option<&mut T> {
        Self::assert_write_declared::<T>();
        self.storage.get_mut::<T>(id)
    }

    /// 写入指定实体的组件 T（当前帧）
    ///
    /// 若 `T` 未在 `D::Writes` 中声明，会在运行时 panic。
    pub fn insert<T: Component>(&mut self, id: EntityId, value: T) {
        Self::assert_write_declared::<T>();
        self.storage.insert::<T>(id, value);
    }

    /// 遍历活跃存储中所有拥有组件 T 的实体（可变，当前帧）
    ///
    /// 若 `T` 未在 `D::Writes` 中声明，会在运行时 panic。
    pub fn each_mut<T: Component>(&mut self) -> impl Iterator<Item = (EntityId, &mut T)> {
        Self::assert_write_declared::<T>();
        self.storage.iter_mut::<T>()
    }

    // ──── 事件 ──────────────────────────────────────────────────────────

    /// 发出事件（README：事件 / Event）
    ///
    /// 事件将在帧末分发给所有通过 [`WorldBuilder::on`](crate::WorldBuilder::on) 或
    /// [`WorldBuilder::observe`](crate::WorldBuilder::observe) 注册的处理器。
    pub fn emit<E: Event>(&mut self, event: E) {
        self.events.emit(event);
    }

    // ──── 生命周期命令 ───────────────────────────────────────────────────

    /// 请求生成新实体（Phase 5 执行）
    pub fn spawn<E: Entity>(&mut self) -> EntityId {
        let bundle = E::bundle();
        self.pending_spawns.push(PendingSpawn::new::<E>(bundle));
        EntityId::placeholder()
    }

    /// 请求销毁实体（Phase 5 执行）
    pub fn destroy(&mut self, id: EntityId) {
        self.pending_destroys.push(id);
    }

    // ──── 时钟快捷访问 ───────────────────────────────────────────────────

    /// 当前时间（秒）
    pub fn time(&self) -> f64 {
        self.clock.time
    }

    // ──── 日志接口 ───────────────────────────────────────────────────────

    /// 构造当前域的 [`LogContext`]
    fn log_ctx(&self, entity_id: Option<EntityId>) -> LogContext {
        LogContext::new(
            FramePhase::DomainCompute,
            self.clock.time,
            self.delta_time,
            self.clock.step_count,
            entity_id,
        )
    }

    /// 记录 Trace 级别日志（带域阶段上下文）
    pub fn trace(&self, target: &str, message: &str) {
        self.logger.trace(self.log_ctx(None), target, message);
    }

    /// 记录 Debug 级别日志（带域阶段上下文）
    pub fn debug(&self, target: &str, message: &str) {
        self.logger.debug(self.log_ctx(None), target, message);
    }

    /// 记录 Info 级别日志（带域阶段上下文）
    pub fn info(&self, target: &str, message: &str) {
        self.logger.info(self.log_ctx(None), target, message);
    }

    /// 记录 Warn 级别日志（带域阶段上下文）
    pub fn warn(&self, target: &str, message: &str) {
        self.logger.warn(self.log_ctx(None), target, message);
    }

    /// 记录 Error 级别日志（带域阶段上下文）
    pub fn error(&self, target: &str, message: &str) {
        self.logger.error(self.log_ctx(None), target, message);
    }

    /// 获取底层日志句柄（用于复杂场景，如循环内条件日志）
    pub fn logger(&self) -> &LoggerHandle {
        self.logger
    }
}
