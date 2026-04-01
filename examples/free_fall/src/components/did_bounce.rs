/// 本帧是否发生弹跳（由 MotionDomain 写入的**事实**）
///
/// 属于**事实**（`Reality`）语义：`MotionDomain` 每帧权威写入（先重置为 false，
/// 发生碰撞时置为 true）；Ball 在同帧的 `tick()` 中经快照只读（上帧值）。
#[derive(Debug, Clone, Default)]
pub struct DidBounce {
    pub value: bool,
}

duan::reality!(DidBounce);
