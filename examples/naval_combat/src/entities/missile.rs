use duan::Entity;

/// 导弹实体（无主观行为，物理和寻的由 MotionDomain / CollisionDomain 驱动）
pub struct Missile;

impl Entity for Missile {}
