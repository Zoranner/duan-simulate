//! 自由落体小球示例
//!
//! 展示 DUAN 仿真体系的基本使用方式：
//! - 定义组件（纯数据）
//! - 实现域规则（计算逻辑）
//! - 创建实体并声明域归属
//! - 运行仿真循环

use duan::{
    Component, DomainContext, DomainRules, Entity, EntityId, World,
};
use std::any::Any;
use std::cell::RefCell;

// ============================================================================
// 组件定义（纯数据，无行为）
// ============================================================================

/// 位置组件
#[derive(Debug, Clone)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Component for Position {
    fn component_type(&self) -> &'static str {
        "position"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn into_any_boxed(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

/// 速度组件
#[derive(Debug, Clone)]
pub struct Velocity {
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
}

impl Component for Velocity {
    fn component_type(&self) -> &'static str {
        "velocity"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn into_any_boxed(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

/// 质量组件
#[derive(Debug, Clone)]
pub struct Mass {
    pub value: f64,
}

impl Component for Mass {
    fn component_type(&self) -> &'static str {
        "mass"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn into_any_boxed(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

// ============================================================================
// 物理域（权威计算单元）
// ============================================================================

/// 物理域规则
///
/// 负责所有物理相关的计算，包括重力、碰撞等。
/// 这是物理领域的唯一权威。
pub struct PhysicsRules {
    /// 重力加速度 (m/s²)
    gravity: f64,
    /// 地面高度
    ground_level: f64,
    /// 反弹系数（预留）
    #[allow(dead_code)]
    restitution: f64,
}

impl PhysicsRules {
    /// 创建新的物理规则
    pub fn new(gravity: f64, ground_level: f64, restitution: f64) -> Self {
        Self {
            gravity,
            ground_level,
            restitution,
        }
    }

    /// 创建地球物理规则
    pub fn earth() -> Self {
        Self::new(9.8, 0.0, 0.3)
    }
}

impl DomainRules for PhysicsRules {
    fn compute(&mut self, ctx: &mut DomainContext, dt: f64) {
        // 收集需要发出的事件
        let mut events = Vec::new();

        // 遍历该域中的所有实体
        let domain = ctx.get_domain_by_name("physics").unwrap();

        for entity_id in domain.entity_ids() {
            // 获取实体
            let Some(entity) = ctx.entities.get(entity_id) else {
                continue;
            };

            // 获取组件
            let Some(pos) = entity.get_component::<Position>() else {
                continue;
            };
            let Some(vel) = entity.get_component::<Velocity>() else {
                continue;
            };

            // 计算新位置和新速度（只读取，不修改）
            let new_x = pos.x + vel.vx * dt;
            let new_y = pos.y + vel.vy * dt - 0.5 * self.gravity * dt * dt;
            let new_z = pos.z + vel.vz * dt;
            let new_vy = vel.vy - self.gravity * dt;

            // 发出位置更新事件（由事件处理器写入实体状态）
            events.push(duan::DomainEvent::Custom(Box::new(PositionUpdateEvent {
                entity_id,
                x: new_x,
                y: new_y,
                z: new_z,
                vx: vel.vx,
                vy: new_vy,
                vz: vel.vz,
            })));

            // 检测地面碰撞
            if new_y <= self.ground_level {
                events.push(duan::DomainEvent::Custom(Box::new(GroundCollisionEvent {
                    entity_id,
                    impact_velocity: new_vy.abs(),
                })));
            }
        }

        // 发出所有收集的事件
        for event in events {
            ctx.emit(event);
        }
    }

    fn try_attach(&mut self, entity: &Entity) -> bool {
        // 实体必须有位置和速度组件才能加入物理域
        entity.has_component::<Position>() && entity.has_component::<Velocity>()
    }

    fn on_detach(&mut self, _entity_id: EntityId) {
        // 清理该实体相关的数据（如果有的话）
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// 地面碰撞事件
#[derive(Debug, Clone)]
pub struct GroundCollisionEvent {
    pub entity_id: EntityId,
    pub impact_velocity: f64,
}

impl duan::CustomEvent for GroundCollisionEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn clone_event(&self) -> Box<dyn duan::CustomEvent> {
        Box::new(self.clone())
    }
    fn event_name(&self) -> &str {
        "ground_collision"
    }
}

/// 位置更新事件
///
/// 由物理域计算新位置，事件处理器写入实体状态。
#[derive(Debug, Clone)]
pub struct PositionUpdateEvent {
    pub entity_id: EntityId,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
}

impl duan::CustomEvent for PositionUpdateEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn clone_event(&self) -> Box<dyn duan::CustomEvent> {
        Box::new(self.clone())
    }
    fn event_name(&self) -> &str {
        "position_update"
    }
}

// ============================================================================
// 主程序
// ============================================================================

fn main() {
    println!("=== DUAN 自由落体小球示例 ===\n");

    // 创建世界
    let mut world = World::builder()
        .time_scale(1.0) // 实时
        .build();

    // 注册物理域
    world.register_domain("physics", PhysicsRules::earth());

    // 创建小球实体
    let ball_id = world.generate_entity_id();
    let ball = Entity::new(ball_id, "ball")
        .with_domain("physics")
        .with_component(Position {
            x: 0.0,
            y: 100.0, // 100米高空
            z: 0.0,
        })
        .with_component(Velocity {
            vx: 0.0,
            vy: 0.0, // 初始速度为0
            vz: 0.0,
        })
        .with_component(Mass { value: 1.0 });

    // 生成实体
    world.spawn(ball);

    println!("小球初始状态: 位置 (0, 100, 0), 速度 (0, 0, 0)");
    println!("重力加速度: 9.8 m/s²");
    println!("开始仿真...\n");

    // 仿真参数
    let dt = 0.01; // 时间步长（秒），足够小以便捕捉微小弹跳
    let total_time = 10.0; // 总仿真时间（秒）
    let steps = (total_time / dt) as usize;

    // 事件处理器：处理位置更新和碰撞
    let handler = |event: &dyn duan::CustomEvent, world: &mut World| {
        // 尝试转换为位置更新事件
        if let Some(update) = event.as_any().downcast_ref::<PositionUpdateEvent>() {
            if let Some(entity) = world.get_entity_mut(update.entity_id) {
                if let Some(pos) = entity.components.get_mut::<Position>() {
                    pos.x = update.x;
                    pos.y = update.y;
                    pos.z = update.z;
                }
                if let Some(vel) = entity.components.get_mut::<Velocity>() {
                    vel.vx = update.vx;
                    vel.vy = update.vy;
                    vel.vz = update.vz;
                }
            }
            return;
        }

        // 尝试转换为地面碰撞事件：速度反转，实现反弹
        if let Some(collision) = event.as_any().downcast_ref::<GroundCollisionEvent>() {
            println!("  >> 地面碰撞！冲击速度: {:.2} m/s", collision.impact_velocity);
            if let Some(entity) = world.get_entity_mut(collision.entity_id) {
                if let Some(pos) = entity.components.get_mut::<Position>() {
                    pos.y = 0.0; // 修正位置到地面
                }
                if let Some(vel) = entity.components.get_mut::<Velocity>() {
                    // 当冲击速度过小时（小到重力在 dt 内就能让小球回落），
                    // 停止弹跳，避免每帧反复碰撞的死循环
                    if collision.impact_velocity < 0.3 {
                        println!("  >> 弹跳能量耗尽，小球静止。");
                        vel.vy = 0.0;
                    } else {
                        vel.vy = -collision.impact_velocity * 0.3;
                    }
                }
            }
        }
    };
    let handler_cell = RefCell::new(handler);
    let handler_ref: &RefCell<_> = &handler_cell;

    // 运行仿真
    for _step in 0..steps {
        // 执行一步仿真（带事件处理器）
        world.step(dt, Some(handler_ref));

        // 获取小球状态
        if let Some(ball) = world.get_entity(ball_id) {
            if let (Some(pos), Some(vel)) = (
                ball.get_component::<Position>(),
                ball.get_component::<Velocity>(),
            ) {
                let sim_time = world.sim_time();
                println!(
                    "t={:6.2}s | 位置: ({:7.2}, {:7.2}, {:7.2}) | 速度: ({:7.2}, {:7.2}, {:7.2})",
                    sim_time, pos.x, pos.y, pos.z, vel.vx, vel.vy, vel.vz
                );

                // 检测是否完全静止（速度足够小且贴近地面）
                if pos.y <= 0.01 && vel.vy.abs() < 0.1 {
                    println!("\n小球已静止！仿真结束。");
                    break;
                }
            }
        }
    }

    println!("\n=== 仿真完成 ===");
    println!("最终仿真时间: {:.2}s", world.sim_time());
    println!("总步数: {}", world.clock.step_count);
}
