//! 实体上下文
//!
//! [`EntityContext`] 是实体 `tick()` 的唯一数据入口，提供：
//! - 自身**认知**与**意图**（`Memory` / `Intent`）的读写（编译期通过 [`EntityWritable`] 约束防止写入**状态** `State`）
//! - 世界快照的只读访问（跨实体读取意图与状态 `Intent` / `State`）
//! - 事件发送
//! - 生命周期命令（spawn/destroy）

use crate::component::storage::WorldStorage;
use crate::component::{Component, EntityWritable};
use crate::entity::id::EntityId;
use crate::entity::{Entity, PendingSpawn};
use crate::events::{CustomEvent, EventBuffer};
use crate::snapshot::WorldSnapshot;
use crate::time::TimeClock;

/// 实体 tick 上下文
///
/// 在 Phase 2 期间，框架为每个活跃实体构建此上下文，
/// 调用 `Entity::tick(ctx)` 后销毁。
///
/// # 编译期安全保证
///
/// - `set<T: EntityWritable>`：仅接受认知与意图（`Memory` / `Intent`），状态（`State`）类型编译失败
/// - `snapshot().get<T>`：读取快照（只读），不含认知（`Memory`）类型
pub struct EntityContext<'w> {
    pub(crate) entity_id: EntityId,
    /// 当前帧活跃存储（认知/意图可读写，状态只读活跃值）
    pub(crate) storage: &'w mut WorldStorage,
    /// 上帧快照（意图与状态只读，不含认知）
    pub(crate) snapshot: &'w WorldSnapshot,
    pub(crate) pending_spawns: &'w mut Vec<PendingSpawn>,
    pub(crate) pending_destroys: &'w mut Vec<EntityId>,
    pub(crate) events: &'w mut EventBuffer,
    /// 仿真时钟（只读）
    pub clock: &'w TimeClock,
    /// 当前帧时间步长（秒）
    pub dt: f64,
}

impl<'w> EntityContext<'w> {
    /// 当前实体的 EntityId
    pub fn id(&self) -> EntityId {
        self.entity_id
    }

    // ──── 自身数据访问 ──────────────────────────────────────────────────

    /// 读取自身组件
    ///
    /// - 认知/意图（`Memory`/`Intent`）：读取当前帧活跃值
    /// - 状态（`State`）：读取活跃存储（在 Entity tick 阶段与上帧快照等价，因为域计算尚未运行）
    pub fn get<T: Component>(&self) -> Option<&T> {
        self.storage.get::<T>(self.entity_id)
    }

    /// 写入自身组件（编译期约束：T 必须是认知或意图，即 `Memory` 或 `Intent`）
    ///
    /// 状态（`State`）类型不实现 [`EntityWritable`]，调用 `set::<SomeState>` 将编译失败。
    pub fn set<T: EntityWritable>(&mut self, value: T) {
        self.storage.insert::<T>(self.entity_id, value);
    }

    /// 移除自身组件（编译期约束：T 必须是认知或意图）
    pub fn remove<T: EntityWritable>(&mut self) {
        self.storage.remove_component::<T>(self.entity_id);
    }

    // ──── 世界快照访问 ──────────────────────────────────────────────────

    /// 获取世界快照（只读，排除认知 `Memory`）
    ///
    /// 通过快照读取其他实体的意图与状态，或读取自身的上帧状态值。
    pub fn snapshot(&self) -> &WorldSnapshot {
        self.snapshot
    }

    // ──── 事件 ──────────────────────────────────────────────────────────

    /// 发送自定义事件
    pub fn emit<E: CustomEvent + 'static>(&mut self, event: E) {
        self.events.push_custom(event);
    }

    // ──── 生命周期命令 ───────────────────────────────────────────────────

    /// 请求生成新实体
    ///
    /// 实际 spawn 在 Phase 5 执行，返回的 EntityId 在当前帧不可用。
    pub fn spawn<E: Entity>(&mut self) -> EntityId {
        let bundle = E::bundle();
        self.pending_spawns.push(PendingSpawn::new::<E>(bundle));
        // 返回占位 ID，实际 ID 在 Phase 5 分配
        EntityId::placeholder()
    }

    /// 请求销毁实体
    ///
    /// 实际销毁在 Phase 5 执行。
    pub fn destroy(&mut self, id: EntityId) {
        self.pending_destroys.push(id);
    }

    // ──── 时钟快捷访问 ───────────────────────────────────────────────────

    /// 当前仿真时间（秒）
    pub fn sim_time(&self) -> f64 {
        self.clock.sim_time
    }
}
