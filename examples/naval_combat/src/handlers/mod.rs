use std::sync::{Arc, Mutex};

use duan::WorldBuilder;

use crate::AppState;

mod combat;
mod missiles;
mod ships;

/// 将所有事件处理器注册到 WorldBuilder
pub(crate) fn install(builder: WorldBuilder, app: &Arc<Mutex<AppState>>) -> WorldBuilder {
    let builder = missiles::install(builder, app);
    let builder = combat::install(builder, app);
    ships::install(builder, app)
}
