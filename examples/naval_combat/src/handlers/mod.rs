use std::sync::{Arc, Mutex};

use duan::WorldBuilder;

use crate::SimulationOutput;

mod combat;
mod missiles;
mod ships;

/// 创建事件处理器安装函数
///
/// 返回 `fn(WorldBuilder) -> WorldBuilder` 形式的装配函数，
/// 通过 `WorldBuilder::apply` 将所有事件处理器模块化注册到世界中：
///
/// ```rust,ignore
/// World::builder()
///     .apply(handlers::install(&simulation_output))
///     .build()
/// ```
pub(crate) fn install(
    simulation_output: &Arc<Mutex<SimulationOutput>>,
) -> impl FnOnce(WorldBuilder) -> WorldBuilder + '_ {
    |builder| {
        let builder = missiles::install(builder, simulation_output);
        let builder = combat::install(builder, simulation_output);
        ships::install(builder, simulation_output)
    }
}
