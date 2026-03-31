//! 运动域
//!
//! 对所有有 `Position` + `Velocity` 的动态实体执行半隐式欧拉积分。
//! 无前置依赖，在 `CollisionDomain` 之前执行。
//!
//! # 积分算法
//!
//! 使用半隐式欧拉积分（Symplectic Euler）：
//! ```text
//! v_new = v_old + a * dt
//! p_new = p_old + v_new * dt
//! ```

use duan::{Domain, DomainContext};

use crate::components::{Position, Velocity};

/// 运动域：每帧积分速度和位置
pub struct MotionDomain {
    gravity: f64,
}

impl MotionDomain {
    pub fn earth() -> Self {
        Self { gravity: 9.8 }
    }
}

impl Domain for MotionDomain {
    type Writes = (Position, Velocity);
    type Reads = ();
    type After = ();

    fn compute(&mut self, ctx: &mut DomainContext<Self>, delta_time: f64) {
        let gravity = self.gravity;

        // 收集有 Velocity 的实体 ID（copy 出来避免借用冲突）
        let ids: Vec<_> = ctx.each_mut::<Velocity>().map(|(id, _)| id).collect();

        for id in ids {
            // 用 .map() 立即 copy 值并释放可变借用，然后再做下一次 get_mut
            let Some((x, y)) = ctx.get_mut::<Position>(id).map(|p| (p.x, p.y)) else {
                continue;
            };
            let Some((vx, vy)) = ctx.get_mut::<Velocity>(id).map(|v| (v.vx, v.vy)) else {
                continue;
            };

            let vy_new = vy - gravity * delta_time;

            if let Some(vel) = ctx.get_mut::<Velocity>(id) {
                vel.vy = vy_new;
            }
            if let Some(pos) = ctx.get_mut::<Position>(id) {
                pos.x = x + vx * delta_time;
                pos.y = y + vy_new * delta_time;
            }
        }
    }
}
