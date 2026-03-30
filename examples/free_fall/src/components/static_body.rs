/// 静态体标记（State：零大小类型，标记地面等不受运动积分的物体）
#[derive(Debug, Clone)]
pub struct StaticBody;

duan::state!(StaticBody);
