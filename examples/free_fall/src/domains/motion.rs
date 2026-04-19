//! 运动域（含碰撞响应）
//!
//! 作为 `Position` / `Velocity` / `DidBounce` 的唯一权威域，每帧顺序执行：
//!
//! 1. 从快照读取小球**意图** `Elasticity`（Ball::tick() 在 Phase 2 写入），
//!    乘以地面 `Collider.restitution` 得出本帧实际弹性系数。
//! 2. 半隐式欧拉积分（Symplectic Euler）：
//!    ```text
//!    v_new = v_old - g * dt
//!    p_new = p_old + v_new * dt
//!    ```
//! 3. 地面碰撞检测与弹性响应：
//!    若 `y_new ≤ 0` 且 `vy_new < 0`，贴地并按弹性系数反转速度。
//! 4. 更新 [`DidBounce`] 事实（Ball::tick() 下帧经快照感知）。
//! 5. 发出 [`GroundCollisionEvent`]（由 handlers 模块中的 Observer 消费）。
//!
//! # 为什么合并而不拆分？
//!
//! 框架规定每个 Reality（事实）组件只能由**唯一**的域写入。积分和碰撞修正都需要写入
//! `Position` 和 `Velocity`，因此它们必须属于同一个权威域。
//! 将它们硬拆为两个域会引发调度器写冲突（Scheduler write-conflict panic）。

use duan::{Domain, DomainContext};

use crate::components::{Collider, DidBounce, Elasticity, Position, StaticBody, Velocity};
use crate::events::GroundCollisionEvent;

/// 运动域：积分 + 碰撞响应
pub struct MotionDomain {
    gravity: f64,
}

impl MotionDomain {
    pub fn earth() -> Self {
        Self { gravity: 9.8 }
    }
}

impl Domain for MotionDomain {
    type Writes = duan::component_set!(Position, Velocity, DidBounce);
    /// Reads 同时包含 Reality（Collider / StaticBody）和 Intent（Elasticity）
    type Reads = duan::component_set!(Collider, StaticBody, Elasticity);
    type After = duan::domain_set!();

    fn compute(&mut self, ctx: &mut DomainContext<Self>, delta_time: f64) {
        let gravity = self.gravity;

        // 地面弹性系数（Reality，来自静态体的 Collider，快照只读）
        let ground_restitution = ctx
            .entities::<StaticBody>()
            .find_map(|id| ctx.get::<Collider>(id))
            .map(|c| c.restitution)
            .unwrap_or(1.0);

        // 收集有 Velocity 的实体 ID（避免多重借用）
        let ids: Vec<_> = ctx.each_mut::<Velocity>().map(|(id, _)| id).collect();

        for id in ids {
            // 当帧存储：读取 position / velocity 当前值
            let Some((x, y)) = ctx.get_mut::<Position>(id).map(|p| (p.x, p.y)) else {
                continue;
            };
            let Some((vy, vx)) = ctx.get_mut::<Velocity>(id).map(|v| (v.vy, v.vx)) else {
                continue;
            };

            // 快照读取：小球的意图弹性系数（Intent，由 Ball::tick() 在 Phase 2 写入）
            // 与地面弹性系数相乘，体现实体意图和物理参数的共同作用
            let ball_restitution = ctx
                .get::<Elasticity>(id)
                .map(|e| e.restitution)
                .unwrap_or(0.8);
            let restitution = ball_restitution * ground_restitution;

            // Phase 1：半隐式欧拉积分
            let vy_new = vy - gravity * delta_time;
            let y_new = y + vy_new * delta_time;
            let x_new = x + vx * delta_time;

            // 每帧先重置 DidBounce（默认无弹跳）
            ctx.insert(id, DidBounce { value: false });

            if y_new <= 0.0 && vy_new < 0.0 {
                // 碰撞响应：贴地 + 弹性反射
                if let Some(pos) = ctx.get_mut::<Position>(id) {
                    pos.x = x_new;
                    pos.y = 0.0;
                }
                if let Some(vel) = ctx.get_mut::<Velocity>(id) {
                    vel.vy = -vy_new * restitution;
                }

                ctx.insert(id, DidBounce { value: true });

                ctx.emit(GroundCollisionEvent {
                    impact_velocity: vy_new,
                    restitution,
                });
            } else {
                // 正常积分写回
                if let Some(pos) = ctx.get_mut::<Position>(id) {
                    pos.x = x_new;
                    pos.y = y_new;
                }
                if let Some(vel) = ctx.get_mut::<Velocity>(id) {
                    vel.vy = vy_new;
                }
            }
        }
    }
}
