//! 碰撞域
//!
//! 检测导弹是否抵达目标（近距离碰撞），发出 HitEvent；
//! 检测导弹是否超出最大射程，发出 MissileExpiredEvent。

use duan::{Domain, DomainContext, EntityId};

use crate::components::{Health, MissileBody, Position, SeekerConfig, SeekerState};
use crate::domains::{CombatDomain, MotionDomain};
use crate::events::{HitEvent, MissileExpiredEvent};

/// 导弹碰撞阈值（像素/米）
const HIT_RADIUS: f64 = 12.0;

/// 碰撞域
pub struct CollisionDomain;

impl Domain for CollisionDomain {
    type Writes = (Health,);
    type Reads = (Position, MissileBody, SeekerConfig, SeekerState);
    type After = (MotionDomain, CombatDomain);

    fn compute(&mut self, ctx: &mut DomainContext<Self>, _delta_time: f64) {
        // 从快照中收集导弹列表（有 MissileBody + SeekerConfig）
        let missiles: Vec<(EntityId, f64, f64, EntityId, f64, f64)> = ctx
            .each::<MissileBody>()
            .filter_map(|(id, _)| {
                let pos = ctx.get::<Position>(id)?;
                let cfg = ctx.get::<SeekerConfig>(id)?;
                Some((id, pos.x, pos.y, cfg.target_id, cfg.damage, cfg.max_range))
            })
            .collect();

        // 从快照读取目标位置
        let target_positions: Vec<(EntityId, f64, f64)> = ctx
            .each::<Position>()
            .map(|(id, p)| (id, p.x, p.y))
            .collect();

        for (missile_id, mx, my, target_id, damage, max_range) in missiles {
            // 检查是否超出射程
            let traveled = ctx
                .get::<SeekerState>(missile_id)
                .map(|s| s.traveled)
                .unwrap_or(0.0);

            if traveled >= max_range {
                ctx.emit(MissileExpiredEvent { missile_id });
                continue;
            }

            // 检查是否击中目标
            let target_pos = target_positions
                .iter()
                .find(|(id, _, _)| *id == target_id)
                .map(|(_, x, y)| (*x, *y));

            if let Some((tx, ty)) = target_pos {
                let dx = mx - tx;
                let dy = my - ty;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist <= HIT_RADIUS {
                    // 扣血
                    if let Some(health) = ctx.get_mut::<Health>(target_id) {
                        health.current = (health.current - damage).max(0.0);
                    }
                    ctx.emit(HitEvent {
                        missile_id,
                        target_id,
                        damage,
                    });
                }
            }
        }
    }
}
