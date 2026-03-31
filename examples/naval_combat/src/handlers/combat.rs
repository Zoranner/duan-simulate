use std::sync::{Arc, Mutex};

use duan::{Reaction, World, WorldBuilder};
use naval_combat::components::Health;
use naval_combat::events::HitEvent;

use crate::display::LogEntry;
use crate::AppState;

pub(super) fn install(builder: WorldBuilder, app: &Arc<Mutex<AppState>>) -> WorldBuilder {
    builder.on::<HitEvent>(on_hit(app))
}

fn on_hit(app: &Arc<Mutex<AppState>>) -> impl Reaction<HitEvent> {
    let app = Arc::clone(app);
    move |e: &HitEvent, world: &mut World| {
        let health_after = world
            .get::<Health>(e.target_id)
            .map(|h| h.current)
            .unwrap_or(0.0);
        world.destroy(e.missile_id);

        let t = world.time();
        let mut s = app.lock().unwrap();
        s.total_hits += 1;
        s.missile_ids.retain(|&id| id != e.missile_id);
        let target_name = s.log.get_name(e.target_id);
        world.event_info(
            "naval_combat::events",
            &format!(
                "hit target={target_name} damage={} hp_after={health_after:.1}",
                e.damage
            ),
        );
        s.log.log(
            t,
            LogEntry::Hit {
                target: target_name,
                damage: e.damage,
                health_after,
            },
        );
    }
}
