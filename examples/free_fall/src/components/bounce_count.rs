/// 弹跳次数（Ball 的**认知**数据）
///
/// 属于**认知**（`Belief`）语义：仅由 Ball 自身在 `tick()` 中更新，
/// 域和快照均不可见。
#[derive(Debug, Clone, Default)]
pub struct BounceCount {
    pub count: u32,
}

duan::belief!(BounceCount);
