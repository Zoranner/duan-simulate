//! 运动域
//!
//! 负责纯运动学积分，直接修改实体的位置和速度。
//! 这是本示例中最基础的域——无依赖、最先执行，其输出是碰撞域的输入。
//!
//! # 框架概念展示
//!
//! 本域体现了"域是权威"的核心设计原则：
//! - 域直接修改**自身管辖**实体的组件状态，而不是通过事件传递
//! - 域不产生事件——运动结果由碰撞域在下一阶段读取（执行顺序由依赖声明保证）
//!
//! # 积分算法
//!
//! 使用**半隐式欧拉积分**（Symplectic Euler）：
//! ```text
//! v_new = v_old + a * dt     （先更新速度）
//! p_new = p_old + v_new * dt （再用新速度更新位置）
//! ```
//! 与标准欧拉（先更新位置再更新速度）相比，半隐式欧拉在弹跳等能量守恒场景下更稳定，
//! 不会产生能量随时间虚假累积的问题。

use duan::{domain_rules_any, DomainContext, DomainRules, Entity, EntityId};

use crate::components::{Position, Velocity};

/// 运动域规则
///
/// 将重力加速度作为域的配置参数，每帧对管辖实体执行半隐式欧拉积分。
/// 域持有 `gravity` 作为跨帧不变的配置，符合框架"域可持有内部状态"的设计。
pub struct MotionRules {
    /// 重力加速度（m/s²，正值向下）
    gravity: f64,
}

impl MotionRules {
    pub fn new(gravity: f64) -> Self {
        Self { gravity }
    }

    pub fn earth() -> Self {
        Self::new(9.8)
    }
}

impl DomainRules for MotionRules {
    fn compute(&mut self, ctx: &mut DomainContext) {
        let dt = ctx.dt;

        // 先 collect 释放对 ctx 的不可变借用，之后才能可变访问 ctx.entities。
        // 这是 Rust 借用规则在 compute 中的标准写法（见框架文档 custom-domain.md）。
        let entity_ids: Vec<EntityId> = ctx.own_entity_ids().collect();

        for entity_id in entity_ids {
            // 阶段一：只读借用，提取当前帧状态到局部变量
            let (x, y, z, vx, vy, vz) = {
                let entity = match ctx.entities.get(entity_id) {
                    Some(e) => e,
                    None => continue,
                };
                let pos = match entity.get_component::<Position>() {
                    Some(p) => p,
                    None => continue, // 没有 Position 的实体跳过（不应发生，try_attach 已保证）
                };
                let vel = match entity.get_component::<Velocity>() {
                    Some(v) => v,
                    None => continue,
                };
                (pos.x, pos.y, pos.z, vel.vx, vel.vy, vel.vz)
            }; // 只读借用在此释放

            // 半隐式欧拉积分：先用加速度更新速度，再用新速度更新位置
            // 重力沿 -y 方向：vy_new = vy - g * dt
            // 水平方向（x/z）无外力，位置用当前速度直接积分，速度值不变
            let vy_new = vy - self.gravity * dt;

            // 阶段二：可变借用，写回计算结果（域作为权威直接修改组件）
            if let Some(entity) = ctx.entities.get_mut(entity_id) {
                if let Some(pos) = entity.get_component_mut::<Position>() {
                    pos.x = x + vx * dt;
                    pos.y = y + vy_new * dt; // 用更新后的 vy 积分，这是"半隐式"的关键
                    pos.z = z + vz * dt;
                }
                if let Some(vel) = entity.get_component_mut::<Velocity>() {
                    vel.vy = vy_new;
                    // vx、vz 无加速度，值不变，不需要重新赋值
                }
            }
        }
    }

    /// 准入条件：必须同时持有 Position 和 Velocity 组件
    ///
    /// 静态地面（有 Position 但无 Velocity）不会被接纳，这确保运动域只处理动态物体。
    fn try_attach(&self, entity: &Entity) -> bool {
        entity.has_component::<Position>() && entity.has_component::<Velocity>()
    }

    fn on_detach(&mut self, _entity_id: EntityId) {
        // 运动域无跨帧实体状态，脱离时无需清理
    }

    /// 依赖声明：空（运动域是基础域，常被其他域依赖，自身无依赖）
    fn dependencies(&self) -> Vec<&'static str> {
        vec![]
    }

    domain_rules_any!(MotionRules);
}
