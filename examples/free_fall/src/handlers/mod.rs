//! 事件处理器模块
//!
//! 通过 [`install`] 函数统一装配，主程序只需一行：
//!
//! ```rust,ignore
//! World::builder()
//!     .domain(MotionDomain::earth())
//!     .domain(CollisionDomain)
//!     .apply(handlers::install(&app))
//!     .build()
//! ```

use std::sync::{Arc, Mutex};

use duan::{Observer, World, WorldBuilder};
use free_fall::events::GroundCollisionEvent;

use crate::display::CollisionSnapshot;
use crate::AppState;

/// 将所有事件处理器注册到 WorldBuilder
///
/// 返回 `FnOnce(WorldBuilder) -> WorldBuilder` 形式的装配函数，
/// 通过 [`WorldBuilder::apply`] 与其他子系统组合。
pub fn install(app: &Arc<Mutex<AppState>>) -> impl FnOnce(WorldBuilder) -> WorldBuilder + '_ {
    |builder| builder.observe::<GroundCollisionEvent>(OnGroundCollision { app: Arc::clone(app) })
}

// ──── 观察器 ──────────────────────────────────────────────────────────────

/// 地面碰撞观察器
///
/// 只读消费 [`GroundCollisionEvent`]，将弹跳统计写回展示状态 [`AppState`]。
/// 不修改仿真世界，纯展示层副作用。
struct OnGroundCollision {
    app: Arc<Mutex<AppState>>,
}

impl Observer<GroundCollisionEvent> for OnGroundCollision {
    fn observe(&mut self, ev: &GroundCollisionEvent, _world: &World) {
        let mut s = self.app.lock().unwrap();
        s.bounce_count += 1;
        s.bounce_flash_remaining = 8;
        s.last_collision = Some(CollisionSnapshot {
            impact_velocity: ev.impact_velocity,
            restitution: ev.restitution,
        });
    }
}
