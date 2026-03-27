//! 自由落体小球仿真 — 主程序
//!
//! 本示例展示 DUAN 框架的完整使用流程：
//! 1. `World::builder()` 链式注册域，构建仿真世界
//! 2. `world.spawn()` 创建带组件和域归属声明的实体
//! 3. `world.step_with()` 推进仿真，在闭包中处理域发出的事件
//! 4. `world.get_entity()` 读取实体状态
//!
//! # 两阶段设计
//!
//! 仿真与渲染分离，避免渲染 `sleep` 影响仿真计算：
//! - **Phase 1（仿真）**：全速推进，每步记录 `RenderFrame` 到帧缓冲
//! - **Phase 2（回放）**：按帧的 `sim_time` 以真实时钟定时渲染

mod display;

use std::time::{Duration, Instant};

use duan::Entity;
use free_fall::components::{Collider, Mass, Position, StaticBody, Velocity};
use free_fall::domains::{CollisionRules, MotionRules};
use free_fall::events::GroundCollisionEvent;

use display::{CollisionSnapshot, FreeFallDisplay, RenderFrame};

/// 小球初始高度（米）
const BALL_INITIAL_HEIGHT: f64 = 10.0;
/// 仿真时间步（秒）
///
/// 10ms 步长在此场景下提供足够精度，步长过大会导致低速末期穿透检测出现误差。
const SIM_DT: f64 = 0.01;
/// 最大仿真时长（秒），防止弹性系数接近 1.0 时仿真无限运行
const MAX_SIM_TIME: f64 = 20.0;
/// 静止判定：高度接近地面且速度足够小时，视为小球已停止弹跳
const REST_HEIGHT_THRESHOLD: f64 = 0.01;
const REST_VELOCITY_THRESHOLD: f64 = 0.1;

fn main() {
    // ── 构建仿真世界 ───────────────────────────────────────
    // with_domain 将域注册延迟到 build() 统一执行；build() 同时验证依赖，循环依赖在此 panic
    let mut world = duan::World::builder()
        .with_domain("motion", MotionRules::earth())
        .with_domain("collision", CollisionRules::new())
        .build();

    // ── 生成实体 ───────────────────────────────────────────
    // 实体通过 with_domain("name") 声明域归属意愿；spawn 时框架依次调用各域的
    // try_attach（判断是否满足准入条件）和 on_attach（域内初始化）。

    // 地面：仅归属碰撞域，不归属运动域（没有速度，不参与运动积分）
    // StaticBody 是零大小标记组件，碰撞域通过它识别静态表面
    world.spawn(
        Entity::new("ground")
            .with_domain("collision")
            .with_component(Position::new(0.0, 0.0, 0.0))
            .with_component(Collider::ground(0.8, 0.05)) // restitution=0.8, friction=0.05
            .with_component(StaticBody),
    );

    // 小球：同时归属运动域（每帧积分）和碰撞域（弹跳检测）
    let ball_id = world.spawn(
        Entity::new("ball")
            .with_domain("motion")
            .with_domain("collision")
            .with_component(Position::new(0.0, BALL_INITIAL_HEIGHT, 0.0))
            .with_component(Velocity::new(0.0, 0.0, 0.0))
            .with_component(Collider::new("小球", 0.0, 0.8, 0.05))
            .with_component(Mass::new(1.0)),
    );

    // ── Phase 1：全速仿真，缓存帧序列 ─────────────────────
    let mut frames: Vec<RenderFrame> = Vec::new();
    let mut bounce_count = 0u32;
    let mut last_collision: Option<CollisionSnapshot> = None;

    let sim_start = Instant::now();

    loop {
        // step_with 执行完整仿真步：时间推进 → 域计算（motion 先，collision 后）→ 事件处理
        // 闭包在事件处理阶段对每个 Custom 事件调用一次
        //
        world.step_with(SIM_DT, |event, _world| {
            if let Some(c) = event.downcast::<GroundCollisionEvent>() {
                bounce_count += 1;
                last_collision = Some(CollisionSnapshot {
                    impact_velocity: c.impact_velocity,
                    restitution: c.restitution,
                });
            }
        });

        // 读取小球当前状态；实体已销毁时退出循环
        let Some(entity) = world.get_entity(ball_id) else {
            break;
        };
        let y = entity.get_component::<Position>().map_or(0.0, |p| p.y);
        let vy = entity.get_component::<Velocity>().map_or(0.0, |v| v.vy);
        let sim_time = world.sim_time();

        frames.push(RenderFrame {
            sim_time,
            y,
            vy,
            bounce_count,
            last_collision,
        });

        // 终止条件：小球贴地且速度足够小（认为已静止），或仿真超过最大时长
        if y <= REST_HEIGHT_THRESHOLD && vy.abs() < REST_VELOCITY_THRESHOLD {
            break;
        }
        if sim_time >= MAX_SIM_TIME {
            break;
        }
    }

    let sim_elapsed = sim_start.elapsed();

    // ── Phase 2：按 sim_time 时间戳回放 ───────────────────
    let display = match FreeFallDisplay::new(BALL_INITIAL_HEIGHT) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("无法初始化终端显示: {e}");
            return;
        }
    };

    let playback_start = Instant::now();

    for frame in &frames {
        // 等到该帧对应的真实时刻再渲染，精确还原物理时间序列
        let target = playback_start + Duration::from_secs_f64(frame.sim_time);
        let now = Instant::now();
        if target > now {
            std::thread::sleep(target - now);
        }
        display.render(frame).ok();
    }

    // 最终帧停留 2 秒，方便观察终态
    std::thread::sleep(Duration::from_secs(2));
    drop(display); // Drop 自动恢复终端状态（LeaveAlternateScreen）

    // ── 仿真统计 ───────────────────────────────────────────
    println!("=== 仿真统计 ===");
    println!("  仿真时间：{:.2} s", world.sim_time());
    println!("  总帧数：  {}", frames.len());
    println!("  弹跳次数：{}", bounce_count);
    println!("  仿真耗时：{:.2} ms", sim_elapsed.as_secs_f64() * 1000.0);
    println!("=== 仿真完成 ===");
}
