//! 自由落体小球仿真 — 主程序
//!
//! 展示 DUAN 双域架构的完整仿真流程。

use duan::World;
use std::cell::RefCell;

use free_fall::components::{Collider, Mass, Position, Velocity};
use free_fall::domains::{CollisionRules, MotionRules};
use free_fall::events::GroundCollisionEvent;

fn main() {
    println!("=== DUAN 自由落体小球示例（双域架构）===\n");

    // 创建仿真世界
    let mut world = World::builder().time_scale(1.0).build();

    // 注册两个域（碰撞域依赖运动域，框架自动保证执行顺序）
    world.register_domain("motion", MotionRules::earth());
    world.register_domain("collision", CollisionRules::new());

    // 创建地面实体（静态碰撞体：Position + Collider）
    let ground_id = world.generate_entity_id();
    let ground = duan::Entity::new(ground_id, "ground")
        .with_domain("collision")
        // 地面有位置（y=0），但没有速度（不参与运动积分）
        .with_component(Position::new(0.0, 0.0, 0.0))
        .with_component(Collider::ground(0.8, 0.05));
    world.spawn(ground);

    // 创建小球实体（动态碰撞体：Position + Velocity + Collider）
    let ball_id = world.generate_entity_id();
    let ball = duan::Entity::new(ball_id, "ball")
        .with_domain("motion") // 运动域处理积分
        .with_domain("collision") // 碰撞域处理落地检测
        .with_component(Position::new(0.0, 10.0, 0.0))
        .with_component(Velocity::new(0.0, 0.0, 0.0))
        .with_component(Collider::new("小球", 0.0, 0.8, 0.05))
        .with_component(Mass::new(1.0));
    world.spawn(ball);

    // 打印初始条件
    println!("初始条件：小球从 y=10m 处自由释放");
    println!("重力加速度：9.8 m/s²（向下）");
    println!("弹性系数：0.8（每次反弹保留 80% 能量）");
    println!("时间步长：0.01s（100 步/秒）\n");
    println!("{SEP}\n");

    // 仿真参数
    let dt = 0.01;
    let total_time = 20.0;
    let steps = (total_time / dt) as usize;

    // 事件处理器（只负责日志，不修改世界状态）
    // 世界状态由域规则直接修改，此处只读取碰撞信息用于打印
    let handler = |event: &dyn duan::CustomEvent, _world: &mut World| {
        if let Some(collision) = event.as_any().downcast_ref::<GroundCollisionEvent>() {
            println!(
                "  >> [碰撞] {} | 冲击速度：{:.2} m/s | 弹性：{:.2}",
                collision.surface_name, collision.impact_velocity, collision.restitution
            );
        }
    };
    let handler_cell = RefCell::new(handler);
    let handler_ref: &RefCell<_> = &handler_cell;

    // 仿真主循环
    for step in 0..steps {
        // 执行一步仿真
        world.step(dt, Some(handler_ref));

        // 获取小球状态
        let (pos, vel) = match world.get_entity(ball_id) {
            Some(e) => (e.get_component::<Position>(), e.get_component::<Velocity>()),
            None => continue,
        };

        let sim_time = world.sim_time();
        println!(
            "t={:6.2}s | 位置：({:7.2}, {:7.2}, {:7.2}) | 速度：({:7.2}, {:7.2}, {:7.2})",
            sim_time,
            pos.map(|p| p.x).unwrap_or(0.0),
            pos.map(|p| p.y).unwrap_or(0.0),
            pos.map(|p| p.z).unwrap_or(0.0),
            vel.map(|v| v.vx).unwrap_or(0.0),
            vel.map(|v| v.vy).unwrap_or(0.0),
            vel.map(|v| v.vz).unwrap_or(0.0),
        );

        // 检测静止条件
        let is_stationary = pos.map(|p| p.y <= 0.01).unwrap_or(false)
            && vel.map(|v| v.vy.abs() < 0.1).unwrap_or(false);

        if is_stationary {
            println!("\n  >> 小球已静止（速度 < 0.1 m/s），仿真结束。\n{SEP}");
            break;
        }

        // 每 100 帧打印分隔线
        if step > 0 && step % 100 == 0 {
            println!("\n{SEP}\n");
        }
    }

    // 打印统计
    println!("\n仿真统计：");
    println!("  仿真时间：{:.2}s", world.sim_time());
    println!("  总步数：{}", world.clock.step_count);
    println!("\n=== 仿真完成 ===");
}

const SEP: &str = "----------------------------------------";
