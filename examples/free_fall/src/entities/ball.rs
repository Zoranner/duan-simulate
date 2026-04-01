use duan::{ComponentBundle, Entity, EntityContext};

use crate::components::{BounceCount, DidBounce, Elasticity};

/// 小球实体
///
/// **认知 / 意图 / 事实**完整示范：
///
/// - `bundle()` 提供**认知** `BounceCount`（`Belief`）和**意图** `Elasticity`（`Intent`）的初值
/// - `tick()` 从快照读取**事实** `DidBounce`，更新自身认知 `BounceCount`，
///   再根据认知重新声明意图 `Elasticity`
///
/// | 组件         | 术语（中文） | Rust    | 读写方向                                  |
/// |------------|-----------|---------|-----------------------------------------|
/// | BounceCount | 认知 | Belief  | Ball `tick()` 独占写；域和快照不可见         |
/// | Elasticity  | 意图 | Intent  | Ball `tick()` 每帧写；MotionDomain 快照只读 |
/// | DidBounce   | 事实 | Reality | MotionDomain 权威写；Ball 经快照只读       |
pub struct Ball;

impl Entity for Ball {
    fn bundle() -> impl ComponentBundle + Send + 'static {
        // 同时提供 Belief 和 Intent 的初值，展示 bundle 可包含多种语义组件
        (BounceCount { count: 0 }, Elasticity::default())
    }

    fn tick(ctx: &mut EntityContext) {
        // 感知：从上帧快照读取事实 DidBounce
        let bounced = ctx
            .snapshot()
            .get::<DidBounce>(ctx.id())
            .map(|d| d.value)
            .unwrap_or(false);

        // 决策：若发生弹跳，更新认知中的弹跳计数
        if bounced {
            let count = ctx.get::<BounceCount>().map(|b| b.count).unwrap_or(0);
            ctx.set(BounceCount { count: count + 1 });
        }

        // 意志：根据弹跳次数声明本帧期望弹性系数（随弹跳次数递减，模拟能量耗散）
        //
        // MotionDomain 将在同帧 Phase 3 从快照读取此意图，乘以地面 Collider.restitution
        // 得出最终碰撞弹性系数。这是实体意图驱动域行为的标准模式。
        let count = ctx.get::<BounceCount>().map(|b| b.count).unwrap_or(0);
        let restitution = (0.85_f64 - count as f64 * 0.05).max(0.1);
        ctx.set(Elasticity { restitution });
    }
}
