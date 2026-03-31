/// 小球的期望弹性系数（Intent：Ball 在 tick() 中每帧声明）
///
/// 属于**意图**（`Intent`）语义：`Ball::tick()` 根据自身弹跳计数（`BounceCount`）
/// 计算并写入期望弹性系数；`MotionDomain` 在同帧 Phase 3 从快照读取，
/// 乘以地面的 `Collider.restitution` 得出最终碰撞弹性系数。
///
/// 弹性随弹跳次数递减，模拟球体能量耗散：
/// ```text
/// restitution = (0.85 - bounce_count * 0.05).max(0.1)
/// ```
#[derive(Debug, Clone)]
pub struct Elasticity {
    /// 期望弹性系数（0.0 = 完全非弹性，1.0 = 完全弹性）
    pub restitution: f64,
}

impl Default for Elasticity {
    fn default() -> Self {
        Self { restitution: 0.85 }
    }
}

duan::intent!(Elasticity);
