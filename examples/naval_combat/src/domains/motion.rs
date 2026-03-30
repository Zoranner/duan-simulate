//! 运动域
//!
//! 每帧执行：
//! 1. **舰船转向**：读取**意图** `Helm`（`Intent`，上帧快照），以 `turn_rate` 限速逐渐转向期望航向
//! 2. **位置积分**：对所有有**状态** `Velocity` 的实体做欧拉积分
//! 3. **导弹里程**：累计**状态** `Seeker` 的飞行距离

use duan::{Domain, DomainContext};

use crate::components::{Helm, Position, Seeker, Velocity};

/// 运动域
pub struct MotionDomain;

impl Domain for MotionDomain {
    type Writes = (Position, Velocity, Seeker);
    type Reads = (Helm,);
    type After = ();

    fn compute(&mut self, ctx: &mut DomainContext<Self>, dt: f64) {
        // ── 1. 舰船转向（读意图 Helm，写状态 Velocity）──────────────────────
        //
        // Helm 为意图（Entity 写），域从快照只读，体现「意志 → 状态」数据流。
        let helms: Vec<_> = ctx
            .each::<Helm>()
            .map(|(id, h)| (id, h.heading, h.turn_rate))
            .collect();

        for (id, desired_heading, turn_rate) in helms {
            let Some((vx, vy)) = ctx.get_mut::<Velocity>(id).map(|v| (v.vx, v.vy)) else {
                continue;
            };
            let speed = (vx * vx + vy * vy).sqrt();
            if speed < 0.01 {
                continue;
            }
            let current_heading = vy.atan2(vx);
            let diff = angle_diff(desired_heading, current_heading);
            let turn = diff.clamp(-turn_rate * dt, turn_rate * dt);
            let new_heading = current_heading + turn;
            if let Some(vel) = ctx.get_mut::<Velocity>(id) {
                vel.vx = new_heading.cos() * speed;
                vel.vy = new_heading.sin() * speed;
            }
        }

        // ── 2. 位置积分（欧拉法）──────────────────────────────────────────
        let ids: Vec<_> = ctx.each_mut::<Velocity>().map(|(id, _)| id).collect();

        for id in ids {
            let Some((vx, vy)) = ctx.get_mut::<Velocity>(id).map(|v| (v.vx, v.vy)) else {
                continue;
            };

            if let Some(pos) = ctx.get_mut::<Position>(id) {
                pos.x += vx * dt;
                pos.y += vy * dt;
            }

            // ── 3. 导弹飞行里程 ──────────────────────────────────────────
            let speed = (vx * vx + vy * vy).sqrt();
            if let Some(seeker) = ctx.get_mut::<Seeker>(id) {
                seeker.traveled += speed * dt;
            }
        }
    }
}

/// 计算从 current 到 desired 的最短角度差，结果在 (-π, π] 范围内
fn angle_diff(desired: f64, current: f64) -> f64 {
    let pi = std::f64::consts::PI;
    let diff = desired - current;
    ((diff + pi).rem_euclid(2.0 * pi)) - pi
}
