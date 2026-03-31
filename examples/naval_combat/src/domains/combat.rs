//! 战斗域
//!
//! 在运动域之后执行：
//! - 更新武器冷却
//! - 对射程内的敌方目标发出 FireEvent
//! - 检测已死亡的舰船，发出 ShipDestroyedEvent

use duan::{Domain, DomainContext, EntityId};

use crate::components::{Faction, Health, Position, Radar, Weapon};
use crate::domains::MotionDomain;
use crate::events::{FireEvent, ShipDestroyedEvent};

/// 战斗域
pub struct CombatDomain;

impl Domain for CombatDomain {
    type Writes = (Weapon,);
    type Reads = (Position, Faction, Radar, Health);
    type After = (MotionDomain,);

    fn compute(&mut self, ctx: &mut DomainContext<Self>, delta_time: f64) {
        // ── 1. 更新武器冷却 ────────────────────────────────────────────────
        let ship_ids: Vec<EntityId> = ctx.each_mut::<Weapon>().map(|(id, _)| id).collect();

        for id in &ship_ids {
            if let Some(w) = ctx.get_mut::<Weapon>(*id) {
                w.cooldown_remaining = (w.cooldown_remaining - delta_time).max(0.0);
            }
        }

        // ── 2. 探测并开火 ──────────────────────────────────────────────────
        // 从快照读取所有实体位置和阵营（上帧值，只读）
        let all_positions: Vec<(EntityId, f64, f64)> = ctx
            .each::<Position>()
            .map(|(id, p)| (id, p.x, p.y))
            .collect();

        let all_factions: Vec<(EntityId, u8)> =
            ctx.each::<Faction>().map(|(id, f)| (id, f.team)).collect();

        for &shooter_id in &ship_ids {
            // 检查是否已死亡（从快照读取）
            let is_dead = ctx
                .get::<Health>(shooter_id)
                .map(|h| h.is_dead())
                .unwrap_or(false);
            if is_dead {
                continue;
            }

            // 读取射手位置和阵营
            let Some((sx, sy)) = all_positions
                .iter()
                .find(|(id, _, _)| *id == shooter_id)
                .map(|(_, x, y)| (*x, *y))
            else {
                continue;
            };
            let Some(my_team) = all_factions
                .iter()
                .find(|(id, _)| *id == shooter_id)
                .map(|(_, t)| *t)
            else {
                continue;
            };

            // 读取武器参数（当前帧可变存储）
            let (range, damage, missile_speed, fire_cd) = {
                let Some(w) = ctx.get_mut::<Weapon>(shooter_id) else {
                    continue;
                };
                if !w.is_ready() {
                    continue;
                }
                (w.range, w.damage, w.missile_speed, w.fire_cooldown)
            };

            // 读取雷达范围（从快照）
            let radar_range = ctx
                .get::<Radar>(shooter_id)
                .map(|r| r.range)
                .unwrap_or(range * 1.5);

            // 在探测范围内找最近的敌方目标
            let mut best_target: Option<(EntityId, f64)> = None;
            for &(target_id, tx, ty) in &all_positions {
                if target_id == shooter_id {
                    continue;
                }
                let enemy_team = all_factions
                    .iter()
                    .find(|(id, _)| *id == target_id)
                    .map(|(_, t)| *t);

                if enemy_team != Some(1 - my_team) {
                    continue; // 同阵营或无阵营跳过
                }

                // 检查目标是否存活
                let target_dead = ctx
                    .get::<Health>(target_id)
                    .map(|h| h.is_dead())
                    .unwrap_or(true);
                if target_dead {
                    continue;
                }

                let dx = tx - sx;
                let dy = ty - sy;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist <= radar_range && best_target.is_none_or(|(_, d)| dist < d) {
                    best_target = Some((target_id, dist));
                }
            }

            // 对射程内的目标开火
            if let Some((target_id, dist)) = best_target {
                if dist <= range {
                    let Some((tx, ty)) = all_positions
                        .iter()
                        .find(|(id, _, _)| *id == target_id)
                        .map(|(_, x, y)| (*x, *y))
                    else {
                        continue;
                    };

                    if let Some(w) = ctx.get_mut::<Weapon>(shooter_id) {
                        w.cooldown_remaining = fire_cd;
                    }

                    ctx.emit(FireEvent {
                        shooter_id,
                        target_id,
                        launch_x: sx,
                        launch_y: sy,
                        dir_x: tx - sx,
                        dir_y: ty - sy,
                        missile_speed,
                        missile_range: range * 2.5,
                        damage,
                    });

                    ctx.info(
                        "naval_combat::combat",
                        &format!("fire shooter={shooter_id} -> target={target_id} dist={dist:.1}"),
                    );
                }
            }
        }

        // ── 3. 检测死亡舰船 ────────────────────────────────────────────────
        let health_ids: Vec<EntityId> = ctx.entities::<Health>().collect();
        for id in health_ids {
            let is_dead = ctx.get::<Health>(id).map(|h| h.is_dead()).unwrap_or(false);
            if is_dead {
                ctx.info(
                    "naval_combat::combat",
                    &format!("ship_destroyed ship_id={id}"),
                );
                ctx.emit(ShipDestroyedEvent { ship_id: id });
            }
        }
    }
}
