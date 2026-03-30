use duan::{ComponentBundle, Entity, EntityContext};

use crate::components::{BounceCount, DidBounce};

/// 小球实体
///
/// **认知 / 意图 / 状态**示范：
///
/// - `bundle()` 提供**认知**组件 `BounceCount`（`Memory`）的默认初值（弹跳数 = 0）
/// - `tick()` 从快照读取**状态** `DidBounce`（`State`），将弹跳计数写回自身**认知**
///
/// | 组件         | 术语（中文） | Rust   | 读写方向                       |
/// |------------|-----------|--------|-------------------------------|
/// | BounceCount | 认知 | Memory | Ball `tick()` 独占写；域和快照不可见 |
/// | DidBounce   | 状态 | State  | CollisionDomain 权威写；Ball 经快照只读 |
pub struct Ball;

impl Entity for Ball {
    fn bundle() -> impl ComponentBundle + Send + 'static {
        (BounceCount { count: 0 },)
    }

    fn tick(ctx: &mut EntityContext) {
        // 读取上帧 CollisionDomain 写入的状态 DidBounce（经快照只读）
        let bounced = ctx
            .snapshot()
            .get::<DidBounce>(ctx.id())
            .map(|d| d.value)
            .unwrap_or(false);

        if bounced {
            // 更新认知中的弹跳计数（Memory：仅 Ball 可写，外部不可见）
            let count = ctx.get::<BounceCount>().map(|b| b.count).unwrap_or(0);
            ctx.set(BounceCount { count: count + 1 });
        }
    }
}
