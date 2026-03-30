//! 碰撞域
//!
//! 在运动域之后执行，检测小球穿越地面（y ≤ 0）并：
//! - 读取当帧位置/速度（MotionDomain 已完成积分，从当帧存储取值）
//! - 修正位置/速度（弹性反射；Position/Velocity 由 MotionDomain 独占拥有，本域作为约束修正）
//! - 独占写入**状态** `DidBounce`（供 Ball::tick() 在下一帧通过快照感知）
//! - 发出 `GroundCollisionEvent`

use duan::{Domain, DomainContext};

use crate::components::{Collider, DidBounce, Position, StaticBody, Velocity};
use crate::domains::MotionDomain;
use crate::events::GroundCollisionEvent;

/// 碰撞域
pub struct CollisionDomain;

impl Domain for CollisionDomain {
    type Writes = (DidBounce,);
    type Reads = (Collider, StaticBody);
    type After = (MotionDomain,);

    fn compute(&mut self, ctx: &mut DomainContext<Self>, _dt: f64) {
        // 读取地面弹性系数（从快照）
        let ground_restitution = ctx
            .entities::<StaticBody>()
            .find_map(|id| ctx.get::<Collider>(id))
            .map(|c| c.restitution)
            .unwrap_or(0.8);

        // 检测所有有 Position + Velocity 的动态实体
        let ids: Vec<_> = ctx.each_mut::<Velocity>().map(|(id, _)| id).collect();

        for id in ids {
            let Some(y) = ctx.get_mut::<Position>(id).map(|p| p.y) else {
                continue;
            };
            let Some(vy) = ctx.get_mut::<Velocity>(id).map(|v| v.vy) else {
                continue;
            };

            // 每帧先重置 DidBounce：确保下一帧快照反映的是本帧真实状态
            ctx.insert(id, DidBounce { value: false });

            if y <= 0.0 && vy < 0.0 {
                let impact = vy;

                if let Some(pos) = ctx.get_mut::<Position>(id) {
                    pos.y = 0.0;
                }
                if let Some(vel) = ctx.get_mut::<Velocity>(id) {
                    vel.vy = -vy * ground_restitution;
                }

                // 通知实体：本帧发生了弹跳（下帧 tick 可通过快照感知）
                ctx.insert(id, DidBounce { value: true });

                ctx.emit(GroundCollisionEvent {
                    impact_velocity: impact,
                    restitution: ground_restitution,
                });
            }
        }
    }
}
