use std::sync::{Arc, Mutex};

use duan::{Observer, Reaction, World, WorldBuilder};
use naval_combat::components::Health;
use naval_combat::events::HitEvent;

use crate::display::LogEntry;
use crate::AppState;

pub(super) fn install(builder: WorldBuilder, app: &Arc<Mutex<AppState>>) -> WorldBuilder {
    builder
        // Reaction：修改世界（销毁导弹）
        .on::<HitEvent>(OnHit {
            app: Arc::clone(app),
        })
        // Observer：只读统计（更新展示层战斗日志）
        .observe::<HitEvent>(LogHit {
            app: Arc::clone(app),
        })
}

// ──── 反应器：销毁导弹 ───────────────────────────────────────────────────────

/// 命中反应器
///
/// 接收 [`HitEvent`]，销毁已命中的导弹并写入框架事件日志。
/// 凡是需要修改仿真世界的逻辑（`world.destroy` 等）均在此处理。
struct OnHit {
    app: Arc<Mutex<AppState>>,
}

impl Reaction<HitEvent> for OnHit {
    fn react(&mut self, ev: &HitEvent, world: &mut World) {
        world.destroy(ev.missile_id);
        let target_name = self.app.lock().unwrap().log.get_name(ev.target_id);
        world.event_info_for(
            ev.target_id,
            "naval_combat::events",
            &format!("hit target={target_name} damage={}", ev.damage),
        );
    }
}

// ──── 观察器：统计与战斗日志 ────────────────────────────────────────────────

/// 命中观察器
///
/// 只读消费 [`HitEvent`]，更新展示层命中统计和战斗日志。
/// 不修改仿真世界，是纯展示层副作用的标准写法。
struct LogHit {
    app: Arc<Mutex<AppState>>,
}

impl Observer<HitEvent> for LogHit {
    fn observe(&mut self, ev: &HitEvent, world: &World) {
        let health_after = world
            .get::<Health>(ev.target_id)
            .map(|h| h.current)
            .unwrap_or(0.0);
        let t = world.time();
        let mut s = self.app.lock().unwrap();
        s.total_hits += 1;
        s.missile_ids.retain(|&id| id != ev.missile_id);
        let target_name = s.log.get_name(ev.target_id);
        s.log.log(
            t,
            LogEntry::Hit {
                target: target_name,
                damage: ev.damage,
                health_after,
            },
        );
    }
}
