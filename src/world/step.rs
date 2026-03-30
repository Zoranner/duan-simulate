//! 5 阶段仿真循环
//!
//! ```text
//! Phase 1  clock.tick(dt)               时间推进
//! Phase 2  冻结 WorldSnapshot            实体 tick（每实体调用 Entity::tick）
//! Phase 3  Domain::compute              域计算（按调度顺序）
//! Phase 4  事件处理                      分发 FrameworkEvent 和 CustomEvent
//! Phase 5  生命周期管理                  批量执行 spawn/destroy，清理已销毁实体
//! ```

use crate::domain::ComputeResources;
use crate::entity::context::EntityContext;
use crate::entity::id::EntityId;
use crate::entity::{Lifecycle, PendingSpawn};
use crate::events::{CustomEvent, FrameworkEvent};
use crate::snapshot::WorldSnapshot;
use std::sync::Arc;

use super::World;

/// 执行一步仿真（带事件回调）
pub fn run<F>(world: &mut World, dt: f64, handler: &mut F)
where
    F: FnMut(&(dyn CustomEvent + 'static), &mut World),
{
    // Phase 1：时间推进
    let sim_dt = world.clock.tick(dt);
    if sim_dt == 0.0 {
        return;
    }

    // Phase 2：冻结快照 + Entity tick
    do_entity_ticks(world, sim_dt);

    // Phase 3：域计算
    do_domain_compute(world, sim_dt);

    // 定时器检查（在域计算后，事件处理前）
    world.handle_timer_events();

    // Phase 4：事件处理
    let events = world.events.drain();
    for event in events {
        if let FrameworkEvent::Custom(arc) = &event {
            handler(arc.as_ref(), world);
        }
        handle_framework_event(world, event);
    }

    // Phase 5：生命周期管理
    world.cleanup_destroyed();
}

/// 执行一步仿真，收集本帧所有自定义事件
pub fn run_collect(world: &mut World, dt: f64) -> Vec<Arc<dyn CustomEvent + 'static>> {
    let sim_dt = world.clock.tick(dt);
    if sim_dt == 0.0 {
        return Vec::new();
    }

    do_entity_ticks(world, sim_dt);
    do_domain_compute(world, sim_dt);
    world.handle_timer_events();

    let events = world.events.drain();
    let mut collected = Vec::new();

    for event in events {
        if let FrameworkEvent::Custom(arc) = &event {
            collected.push(arc.clone());
        }
        handle_framework_event(world, event);
    }

    world.cleanup_destroyed();
    collected
}

// ──── Phase 2：Entity tick ────────────────────────────────────────────────

fn do_entity_ticks(world: &mut World, dt: f64) {
    // 冻结快照（排除认知 Memory 类型）
    let snapshot = WorldSnapshot::build(&world.storage, &world.memory_type_ids);

    type TickEntry = (EntityId, fn(&mut EntityContext));
    // 收集所有活跃实体 ID 和 tick 函数（避免借用冲突）
    let active: Vec<TickEntry> = world
        .entities
        .values()
        .filter(|r| r.lifecycle.is_active())
        .map(|r| (r.id, r.tick_fn))
        .collect();

    let mut pending_spawns: Vec<PendingSpawn> = Vec::new();
    let mut pending_destroys: Vec<EntityId> = Vec::new();

    for (id, tick_fn) in active {
        let mut ctx = EntityContext {
            entity_id: id,
            storage: &mut world.storage,
            snapshot: &snapshot,
            pending_spawns: &mut pending_spawns,
            pending_destroys: &mut pending_destroys,
            events: &mut world.events,
            clock: &world.clock,
            dt,
        };
        tick_fn(&mut ctx);
    }

    // Phase 5 预处理：立即处理本帧 tick 产生的 spawn/destroy
    // （spawn 先于 destroy，保证同帧内 spawn 的实体不被立即销毁）
    world.flush_pending(pending_spawns, pending_destroys);
}

// ──── Phase 3：Domain compute ─────────────────────────────────────────────

fn do_domain_compute(world: &mut World, dt: f64) {
    // 域计算前再次冻结快照（包含 Entity tick 修改的意图与认知结果）
    let snapshot = WorldSnapshot::build(&world.storage, &world.memory_type_ids);

    let order = world.scheduler.execution_order.clone();

    let mut pending_spawns: Vec<PendingSpawn> = Vec::new();
    let mut pending_destroys: Vec<EntityId> = Vec::new();

    // 将 domains 暂时移出，使借用检查器能同时访问 world 的其他字段。
    // compute_dyn 期间 world.domains 为空 Vec；执行后归还。
    let mut domains = std::mem::take(&mut world.domains);

    for idx in order {
        domains[idx].compute_dyn(ComputeResources {
            storage: &mut world.storage,
            snapshot: &snapshot,
            pending_spawns: &mut pending_spawns,
            pending_destroys: &mut pending_destroys,
            events: &mut world.events,
            clock: &world.clock,
            dt,
        });
    }

    world.domains = domains;
    world.flush_pending(pending_spawns, pending_destroys);
}

// ──── Phase 4：事件处理 ───────────────────────────────────────────────────

fn handle_framework_event(world: &mut World, event: FrameworkEvent) {
    match event {
        FrameworkEvent::EntityDestroyed { entity_id } => {
            if let Some(rec) = world.entities.get_mut(&entity_id) {
                rec.lifecycle = Lifecycle::Destroyed;
            }
        }
        FrameworkEvent::Timer {
            entity_id,
            timer_id: _,
            callback,
        } => match callback {
            crate::events::TimerCallback::SelfDestruct => {
                if let Some(rec) = world.entities.get_mut(&entity_id) {
                    rec.lifecycle = Lifecycle::Destroyed;
                }
                world.storage.remove_entity(entity_id);
            }
            crate::events::TimerCallback::Event(inner) => {
                handle_framework_event(world, *inner);
            }
        },
        FrameworkEvent::Custom(_) => {
            // 已由调用方的 handler 处理
        }
    }
}
