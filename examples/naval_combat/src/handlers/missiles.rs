use std::sync::{Arc, Mutex};

use duan::{Reaction, World, WorldBuilder};
use naval_combat::components::{
    Faction, MissileBody, Position, SeekerConfig, SeekerState, Velocity,
};
use naval_combat::entities::Missile;
use naval_combat::events::{FireEvent, MissileExpiredEvent};

use crate::display::LogEntry;
use crate::SimulationOutput;

pub(super) fn install(
    builder: WorldBuilder,
    simulation_output: &Arc<Mutex<SimulationOutput>>,
) -> WorldBuilder {
    builder
        .on::<FireEvent>(OnFire {
            simulation_output: Arc::clone(simulation_output),
        })
        .on::<MissileExpiredEvent>(OnMissileExpired {
            simulation_output: Arc::clone(simulation_output),
        })
}

// ──── 开火反应器 ─────────────────────────────────────────────────────────────

/// 开火反应器
///
/// 接收 [`FireEvent`]，在世界中生成导弹实体，并更新展示层追踪列表。
struct OnFire {
    simulation_output: Arc<Mutex<SimulationOutput>>,
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
            SeekerConfig::new(ev.target_id, ev.shooter_id, ev.damage, ev.missile_range),
            SeekerState::default(),
            MissileBody,
            Faction { team: faction_team },
        ));

        let t = world.time();
        let mut s = self.simulation_output.lock().unwrap();
        s.total_missiles += 1;
        s.missile_ids.push(missile_id);
        let shooter_name = s.log.get_name(ev.shooter_id);
        let target_name = s.log.get_name(ev.target_id);
        s.log.register_name(
            missile_id,
            format!("导弹({}→{})", shooter_name, target_name),
        );
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
    simulation_output: Arc<Mutex<SimulationOutput>>,
}

impl Reaction<MissileExpiredEvent> for OnMissileExpired {
    fn react(&mut self, ev: &MissileExpiredEvent, world: &mut World) {
        world.event_debug_for(ev.missile_id, "naval_combat::events", "missile_expired");
        world.destroy(ev.missile_id);
        self.simulation_output
            .lock()
            .unwrap()
            .missile_ids
            .retain(|&id| id != ev.missile_id);
    }
}
