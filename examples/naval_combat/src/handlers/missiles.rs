use std::sync::{Arc, Mutex};

use duan::{Reaction, World, WorldBuilder};
use naval_combat::components::{Faction, MissileBody, Position, Seeker, Velocity};
use naval_combat::entities::Missile;
use naval_combat::events::{FireEvent, MissileExpiredEvent};

use crate::display::LogEntry;
use crate::AppState;

pub(super) fn install(builder: WorldBuilder, app: &Arc<Mutex<AppState>>) -> WorldBuilder {
    builder
        .on::<FireEvent>(OnFire {
            app: Arc::clone(app),
        })
        .on::<MissileExpiredEvent>(OnMissileExpired {
            app: Arc::clone(app),
        })
}

// ──── 开火反应器 ─────────────────────────────────────────────────────────────

/// 开火反应器
///
/// 接收 [`FireEvent`]，在世界中生成导弹实体，并更新展示层追踪列表。
struct OnFire {
    app: Arc<Mutex<AppState>>,
}

impl Reaction<FireEvent> for OnFire {
    fn react(&mut self, ev: &FireEvent, world: &mut World) {
        let faction_team = world
            .get::<Faction>(ev.shooter_id)
            .map(|f| f.team)
            .unwrap_or(99);

        let missile_id = world.spawn_with::<Missile>((
            Position::new(ev.launch_x, ev.launch_y),
            Velocity::towards(ev.dir_x, ev.dir_y, ev.missile_speed),
            Seeker::new(ev.target_id, ev.shooter_id, ev.damage, ev.missile_range),
            MissileBody,
            Faction { team: faction_team },
        ));

        let t = world.time();
        let mut s = self.app.lock().unwrap();
        s.total_missiles += 1;
        s.missile_ids.push(missile_id);
        let shooter_name = s.log.get_name(ev.shooter_id);
        let target_name = s.log.get_name(ev.target_id);
        s.log
            .register_name(missile_id, format!("导弹({}→{})", shooter_name, target_name));
        s.log.log(
            t,
            LogEntry::Fire {
                shooter: shooter_name,
                target: target_name,
            },
        );
    }
}

// ──── 导弹过期反应器 ──────────────────────────────────────────────────────────

/// 导弹过期反应器
///
/// 接收 [`MissileExpiredEvent`]，销毁超出追踪范围的导弹实体。
struct OnMissileExpired {
    app: Arc<Mutex<AppState>>,
}

impl Reaction<MissileExpiredEvent> for OnMissileExpired {
    fn react(&mut self, ev: &MissileExpiredEvent, world: &mut World) {
        world.event_debug_for(ev.missile_id, "naval_combat::events", "missile_expired");
        world.destroy(ev.missile_id);
        self.app
            .lock()
            .unwrap()
            .missile_ids
            .retain(|&id| id != ev.missile_id);
    }
}
