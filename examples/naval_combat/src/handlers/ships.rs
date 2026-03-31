use std::sync::{Arc, Mutex};

use duan::{Reaction, World, WorldBuilder};
use naval_combat::events::ShipDestroyedEvent;

use crate::display::LogEntry;
use crate::SimulationOutput;

pub(super) fn install(
    builder: WorldBuilder,
    simulation_output: &Arc<Mutex<SimulationOutput>>,
) -> WorldBuilder {
    builder.on::<ShipDestroyedEvent>(OnShipDestroyed {
        simulation_output: Arc::clone(simulation_output),
    })
}

// ──── 舰船摧毁反应器 ──────────────────────────────────────────────────────────

/// 舰船摧毁反应器
///
/// 接收 [`ShipDestroyedEvent`]，销毁舰船实体，记录战斗日志。
struct OnShipDestroyed {
    simulation_output: Arc<Mutex<SimulationOutput>>,
}

impl Reaction<ShipDestroyedEvent> for OnShipDestroyed {
    fn react(&mut self, ev: &ShipDestroyedEvent, world: &mut World) {
        let t = world.time();
        let mut s = self.simulation_output.lock().unwrap();
        let name = s.log.get_name(ev.ship_id);
        world.event_info_for(
            ev.ship_id,
            "naval_combat::events",
            &format!("ship_destroyed name={name}"),
        );
        world.destroy(ev.ship_id);
        s.log.log(t, LogEntry::ShipDestroyed { name });
    }
}
