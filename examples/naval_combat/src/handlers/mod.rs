use std::sync::{Arc, Mutex};

use duan::WorldBuilder;

use crate::AppState;

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
///     .apply(handlers::install(&app))
///     .build()
/// ```
pub(crate) fn install(
    app: &Arc<Mutex<AppState>>,
) -> impl FnOnce(WorldBuilder) -> WorldBuilder + '_ {
    |builder| {
        let builder = missiles::install(builder, app);
        let builder = combat::install(builder, app);
        ships::install(builder, app)
    }
}
