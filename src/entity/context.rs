//! 实体上下文
//!
//! [`EntityContext`] 是实体 `tick()` 的唯一数据入口，提供：
//! - 自身**认知**与**意图**（`Belief` / `Intent`）的读写（编译期通过 [`EntityWritable`] 约束防止写入**事实** `Reality`）
//! - 世界快照的只读访问（跨实体读取意图与事实 `Intent` / `Reality`）
//! - 事件发送
//! - 生命周期命令（spawn/destroy）
//! - 带仿真上下文的统一日志接口

use crate::diagnostics::{FramePhase, LogContext, LoggerHandle};
use crate::entity::id::EntityId;
use crate::entity::{Entity, PendingSpawn};
use crate::event::{Event, EventBuffer};
use crate::runtime::timers::TimeClock;
use crate::snapshot::Snapshot;
use crate::storage::Storage;
use crate::{Component, EntityWritable};

/// 实体 tick 上下文
///
/// 在 Phase 2 期间，框架为每个活跃实体构建此上下文，
/// 调用 `Entity::tick(ctx)` 后销毁。
///
/// # 编译期安全保证
///
/// - `set<T: EntityWritable>`：仅接受认知与意图（`Belief` / `Intent`），事实（`Reality`）类型编译失败
/// - `snapshot().get<T>`：读取快照（只读），不含认知（`Belief`）类型
pub struct EntityContext<'w> {
    pub(crate) entity_id: EntityId,
    /// 当前帧活跃存储（认知/意图可读写，事实组件只读活跃值）
    pub(crate) storage: &'w mut Storage,
    /// 上帧快照（意图与事实只读，不含认知）
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
}

impl<'w> EntityContext<'w> {
    /// 当前实体的 EntityId
    pub fn id(&self) -> EntityId {
        self.entity_id
    }

    // ──── 自身数据访问 ──────────────────────────────────────────────────

    /// 读取自身组件
    ///
    /// - 认知/意图（`Belief`/`Intent`）：读取当前帧活跃值
    /// - 事实（`Reality`）：读取活跃存储（在 Entity tick 阶段与上帧快照等价，因为域计算尚未运行）
    pub fn get<T: Component>(&self) -> Option<&T> {
        self.storage.get::<T>(self.entity_id)
    }

    /// 写入自身组件（编译期约束：T 必须是认知或意图，即 `Belief` 或 `Intent`）
    ///
    /// 事实（`Reality`）类型不实现 [`EntityWritable`]，调用 `set::<SomeReality>` 将编译失败。
    pub fn set<T: EntityWritable>(&mut self, value: T) {
        self.storage.insert::<T>(self.entity_id, value);
    }

    /// 移除自身组件（编译期约束：T 必须是认知或意图）
    pub fn remove<T: EntityWritable>(&mut self) {
        self.storage.remove_component::<T>(self.entity_id);
    }

    // ──── 世界快照访问 ──────────────────────────────────────────────────

    /// 获取世界快照（只读，排除认知 `Belief`）
    ///
    /// 通过快照读取其他实体的意图与事实，或读取自身的上帧事实值。
    pub fn snapshot(&self) -> &Snapshot {
        self.snapshot
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

    /// 请求生成新实体
    ///
    /// 实际 spawn 在 Phase 5 执行，返回的 EntityId 在当前帧不可用。
    pub fn spawn<E: Entity>(&mut self) -> EntityId {
        let bundle = E::bundle();
        self.pending_spawns.push(PendingSpawn::new::<E>(bundle));
        EntityId::placeholder()
    }

    /// 请求销毁实体
    ///
    /// 实际销毁在 Phase 5 执行。
    pub fn destroy(&mut self, id: EntityId) {
        self.pending_destroys.push(id);
    }

    // ──── 时钟快捷访问 ───────────────────────────────────────────────────

    /// 当前时间（秒）
    pub fn time(&self) -> f64 {
        self.clock.time
    }

    // ──── 日志接口 ───────────────────────────────────────────────────────

    /// 构造当前实体的 [`LogContext`]（自动补齐 EntityTick 阶段与 entity_id）
    fn log_ctx(&self) -> LogContext {
        LogContext::new(
            FramePhase::EntityTick,
            self.clock.time,
            self.delta_time,
            self.clock.step_count,
            Some(self.entity_id),
        )
    }

    /// 记录 Trace 级别日志（自动附带实体 ID 和 EntityTick 阶段）
    pub fn trace(&self, target: &str, message: &str) {
        self.logger.trace(self.log_ctx(), target, message);
    }

    /// 记录 Debug 级别日志（自动附带实体 ID 和 EntityTick 阶段）
    pub fn debug(&self, target: &str, message: &str) {
        self.logger.debug(self.log_ctx(), target, message);
    }

    /// 记录 Info 级别日志（自动附带实体 ID 和 EntityTick 阶段）
    pub fn info(&self, target: &str, message: &str) {
        self.logger.info(self.log_ctx(), target, message);
    }

    /// 记录 Warn 级别日志（自动附带实体 ID 和 EntityTick 阶段）
    pub fn warn(&self, target: &str, message: &str) {
        self.logger.warn(self.log_ctx(), target, message);
    }

    /// 记录 Error 级别日志（自动附带实体 ID 和 EntityTick 阶段）
    pub fn error(&self, target: &str, message: &str) {
        self.logger.error(self.log_ctx(), target, message);
    }

    /// 获取底层日志句柄（用于复杂场景，如循环内条件日志）
    pub fn logger(&self) -> &LoggerHandle {
        self.logger
    }
}
