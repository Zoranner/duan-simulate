use std::sync::{Arc, Mutex};

use duan::{Reaction, World, WorldBuilder};
use naval_combat::events::ShipDestroyedEvent;

use crate::display::LogEntry;
use crate::AppState;

pub(super) fn install(builder: WorldBuilder, app: &Arc<Mutex<AppState>>) -> WorldBuilder {
    builder.on::<ShipDestroyedEvent>(on_ship_destroyed(app))
}

fn on_ship_destroyed(app: &Arc<Mutex<AppState>>) -> impl Reaction<ShipDestroyedEvent> {
    let app = Arc::clone(app);
    move |e: &ShipDestroyedEvent, world: &mut World| {
        let t = world.time();
        let mut s = app.lock().unwrap();
        let name = s.log.get_name(e.ship_id);
        world.event_info_for(
            e.ship_id,
            "naval_combat::events",
            &format!("ship_destroyed name={name}"),
        );
        world.destroy(e.ship_id);
        s.log.log(t, LogEntry::ShipDestroyed { name });
    }
}
