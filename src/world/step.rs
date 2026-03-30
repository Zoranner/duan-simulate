//! 5 阶段仿真循环
//!
//! ```text
//! Phase 1  clock.tick(dt)               时间推进
//! Phase 2  冻结 WorldSnapshot            实体 tick（每实体调用 Entity::tick）
//! Phase 3  Domain::compute              域计算（按调度顺序）
//! Phase 4  事件分发                      按类型分发到 Reaction / Observer
//! Phase 5  生命周期管理                  批量执行 spawn/destroy，清理已销毁实体
//! ```

use crate::domain::ComputeResources;
use crate::entity::context::EntityContext;
use crate::entity::id::EntityId;
use crate::entity::PendingSpawn;
use crate::events::ArcEvent;
use crate::logging::{FramePhase, LogLevel};
use crate::snapshot::WorldSnapshot;

use super::World;

/// 执行一步仿真
pub fn run(world: &mut World, dt: f64) {
    // Phase 1：时间推进
    let sim_dt = world.clock.tick(dt);
    if sim_dt == 0.0 {
        return;
    }

    let sim_time = world.clock.sim_time;
    let step_count = world.clock.step_count;

    world.emit_at(
        LogLevel::Debug,
        FramePhase::StepStart,
        sim_dt,
        None,
        "duan::step",
        &format!("step #{step_count} begin  sim_time={sim_time:.6}  dt={sim_dt:.6}"),
    );

    // Phase 2：冻结快照 + Entity tick
    do_entity_ticks(world, sim_dt);

    // Phase 3：域计算
    do_domain_compute(world, sim_dt);

    // 定时器检查（在域计算后、事件分发前）
    world.handle_timer_events();

    // Phase 4：事件分发
    let events = world.events.drain();
    if !events.is_empty() {
        world.emit_at(
            LogLevel::Debug,
            FramePhase::EventDispatch,
            sim_dt,
            None,
            "duan::step",
            &format!("dispatching {} event(s)", events.len()),
        );
    }
    do_event_dispatch(world, events, sim_dt);

    // Phase 5：生命周期管理
    world.cleanup_destroyed();

    world.emit_at(
        LogLevel::Debug,
        FramePhase::StepEnd,
        sim_dt,
        None,
        "duan::step",
        &format!("step #{step_count} end"),
    );
}

// ──── Phase 2：Entity tick ────────────────────────────────────────────────

fn do_entity_ticks(world: &mut World, dt: f64) {
    let snapshot = WorldSnapshot::build(&world.storage, &world.memory_type_ids);

    type TickEntry = (EntityId, fn(&mut EntityContext));
    let active: Vec<TickEntry> = world
        .entities
        .values()
        .filter(|r| r.lifecycle.is_active())
        .map(|r| (r.id, r.tick_fn))
        .collect();

    let entity_count = active.len();
    world.emit_at(
        LogLevel::Trace,
        FramePhase::EntityTick,
        dt,
        None,
        "duan::step",
        &format!("entity tick phase: {entity_count} active entities"),
    );

    let mut pending_spawns: Vec<PendingSpawn> = Vec::new();
    let mut pending_destroys: Vec<EntityId> = Vec::new();

    for (id, tick_fn) in active {
        if world.logger().enabled(LogLevel::Trace) {
            world.emit_at(
                LogLevel::Trace,
                FramePhase::EntityTick,
                dt,
                Some(id),
                "duan::entity",
                &format!("tick {id}"),
            );
        }

        let mut ctx = EntityContext {
            entity_id: id,
            storage: &mut world.storage,
            snapshot: &snapshot,
            pending_spawns: &mut pending_spawns,
            pending_destroys: &mut pending_destroys,
            events: &mut world.events,
            clock: &world.clock,
            logger: &world.logger,
            dt,
        };
        tick_fn(&mut ctx);
    }

    world.flush_pending(pending_spawns, pending_destroys);
}

// ──── Phase 3：Domain compute ─────────────────────────────────────────────

fn do_domain_compute(world: &mut World, dt: f64) {
    let snapshot = WorldSnapshot::build(&world.storage, &world.memory_type_ids);

    let order = world.scheduler.execution_order.clone();

    let domain_count = order.len();
    world.emit_at(
        LogLevel::Trace,
        FramePhase::DomainCompute,
        dt,
        None,
        "duan::step",
        &format!("domain compute phase: {domain_count} domain(s)"),
    );

    let mut pending_spawns: Vec<PendingSpawn> = Vec::new();
    let mut pending_destroys: Vec<EntityId> = Vec::new();

    let mut domains = std::mem::take(&mut world.domains);

    for (pos, idx) in order.iter().enumerate() {
        if world.logger().enabled(LogLevel::Trace) {
            world.emit_at(
                LogLevel::Trace,
                FramePhase::DomainCompute,
                dt,
                None,
                "duan::domain",
                &format!("compute domain[{pos}] idx={idx}"),
            );
        }

        domains[*idx].compute_dyn(ComputeResources {
            storage: &mut world.storage,
            snapshot: &snapshot,
            pending_spawns: &mut pending_spawns,
            pending_destroys: &mut pending_destroys,
            events: &mut world.events,
            clock: &world.clock,
            logger: &world.logger,
            dt,
        });
    }

    world.domains = domains;
    world.flush_pending(pending_spawns, pending_destroys);
}

// ──── Phase 4：事件分发 ───────────────────────────────────────────────────

/// 将本帧事实事件按类型分发到反应器和观察器。
///
/// 分发顺序：先依次执行所有反应器（可修改世界），再依次执行所有观察器（只读）。
/// 同一事件类型的多个处理器按注册顺序调用。
fn do_event_dispatch(world: &mut World, events: Vec<ArcEvent>, sim_dt: f64) {
    for arc_event in events {
        if world.logger().enabled(LogLevel::Trace) {
            world.emit_at(
                LogLevel::Trace,
                FramePhase::EventDispatch,
                sim_dt,
                None,
                "duan::step",
                &format!("dispatch event '{}'", arc_event.name),
            );
        }

        // 先执行反应器（可修改世界）
        // 将 reactions 暂时移出以满足借用检查器，执行后归还
        if let Some(mut reactions) = world.reactions.remove(&arc_event.type_id) {
            for r in &mut reactions {
                r.react_dyn(arc_event.inner.as_ref(), world);
            }
            world.reactions.insert(arc_event.type_id, reactions);
        }

        // 再执行观察器（只读消费）
        if let Some(mut observers) = world.observers.remove(&arc_event.type_id) {
            for o in &mut observers {
                o.observe_dyn(arc_event.inner.as_ref(), world);
            }
            world.observers.insert(arc_event.type_id, observers);
        }
    }
}
