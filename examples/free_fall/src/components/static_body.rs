/// 静态体标记（事实 Reality：零大小类型，标记地面等不受运动积分的物体）
#[derive(Debug, Clone)]
pub struct StaticBody;

duan::reality!(StaticBody);
