/// 舵令（舰船表达的**意图**）
///
/// 属于**意图**（`Intent`）语义：由 `Ship::tick()` 在每帧写入期望航向，
/// `MotionDomain` 从快照只读并据此修正**状态**中的速度方向。
///
/// Entity → Domain 数据流示范：
/// - 实体通过 `tick()` 写入**意图**（`Helm`）
/// - 域读取意图，写入**状态**（`Position` / `Velocity` 等）
#[derive(Debug, Clone)]
pub struct Helm {
    /// 最大转向速率（弧度/秒），决定舰船转向敏捷度
    pub turn_rate: f64,
    /// 期望航向（弧度，以 x 轴正方向为 0，逆时针为正）
    pub heading: f64,
}

impl Helm {
    pub fn new(turn_rate: f64) -> Self {
        Self {
            turn_rate,
            heading: 0.0,
        }
    }
}

duan::intent!(Helm);
