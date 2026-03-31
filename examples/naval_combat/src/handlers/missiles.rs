use std::sync::{Arc, Mutex};

use duan::{Reaction, World, WorldBuilder};
use naval_combat::components::{Faction, MissileBody, Position, Seeker, Velocity};
use naval_combat::entities::Missile;
use naval_combat::events::{FireEvent, MissileExpiredEvent};

use crate::display::LogEntry;
use crate::AppState;

pub(super) fn install(builder: WorldBuilder, app: &Arc<Mutex<AppState>>) -> WorldBuilder {
    builder
        .on::<FireEvent>(on_fire(app))
        .on::<MissileExpiredEvent>(on_missile_expired(app))
}

fn on_fire(app: &Arc<Mutex<AppState>>) -> impl Reaction<FireEvent> {
    let app = Arc::clone(app);
    move |e: &FireEvent, world: &mut World| {
        let faction_team = world
            .get::<Faction>(e.shooter_id)
            .map(|f| f.team)
            .unwrap_or(99);

        let missile_id = world.spawn_with::<Missile>((
            Position::new(e.launch_x, e.launch_y),
            Velocity::towards(e.dir_x, e.dir_y, e.missile_speed),
            Seeker::new(e.target_id, e.shooter_id, e.damage, e.missile_range),
            MissileBody,
            Faction { team: faction_team },
        ));

        let t = world.time();
        let mut s = app.lock().unwrap();
        s.total_missiles += 1;
        s.missile_ids.push(missile_id);
        let shooter_name = s.log.get_name(e.shooter_id);
        let target_name = s.log.get_name(e.target_id);
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

fn on_missile_expired(app: &Arc<Mutex<AppState>>) -> impl Reaction<MissileExpiredEvent> {
    let app = Arc::clone(app);
    move |e: &MissileExpiredEvent, world: &mut World| {
        world.event_debug_for(e.missile_id, "naval_combat::events", "missile_expired");
        world.destroy(e.missile_id);
        app.lock().unwrap().missile_ids.retain(|&id| id != e.missile_id);
    }
}
