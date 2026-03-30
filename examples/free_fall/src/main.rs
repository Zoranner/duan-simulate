//! 自由落体小球仿真 — 主程序
//!
//! 本示例展示 DUAN 框架新一代 API 的完整使用流程：
//! 1. `World::builder().with_domain(...).build()` 构建仿真世界（自动调度分析）
//! 2. `world.spawn_with::<Ball>(...)` 生成带运行时组件的实体
//! 3. `world.step_with(dt, |event, _| ...)` 推进仿真并处理事件
//! 4. `world.get::<Position>(id)` 读取实体组件状态
//!
//! # 两阶段设计
//!
//! - **Phase 1（仿真）**：全速推进，每步记录 `RenderFrame` 到帧缓冲
//! - **Phase 2（回放）**：按帧的 `sim_time` 以真实时钟定时渲染

mod display;

use std::time::{Duration, Instant};

use free_fall::components::{Collider, Position, StaticBody, Velocity};
use free_fall::domains::{CollisionDomain, MotionDomain};
use free_fall::entities::{Ball, Ground};
use free_fall::events::GroundCollisionEvent;

use display::{CollisionSnapshot, FreeFallDisplay, RenderFrame};

const BALL_INITIAL_HEIGHT: f64 = 10.0;
const SIM_DT: f64 = 0.01;
const MAX_SIM_TIME: f64 = 20.0;
const REST_HEIGHT_THRESHOLD: f64 = 0.01;
const REST_VELOCITY_THRESHOLD: f64 = 0.1;

fn main() {
    // ── 构建仿真世界 ───────────────────────────────────────────────────────
    // with_domain 注册域；build() 时调度器静态分析写入冲突和循环依赖
    let mut world = duan::World::builder()
        .with_domain(MotionDomain::earth())
        .with_domain(CollisionDomain)
        .build();

    // ── 生成实体 ───────────────────────────────────────────────────────────
    // spawn_with 允许传入运行时确定的组件，与 bundle() 默认值合并

    // 地面（y=0）：StaticBody 标记使运动域跳过此实体
    world.spawn_with::<Ground>((
        Position::new(0.0, 0.0),
        StaticBody,
        Collider::new(0.8, 0.05),
    ));

    // 小球：从初始高度自由落体；BounceCount Memory 由 Ball::bundle() 提供默认值
    let ball_id = world.spawn_with::<Ball>((
        Position::new(0.0, BALL_INITIAL_HEIGHT),
        Velocity::new(0.0, 0.0),
        Collider::new(0.8, 0.05),
    ));

    // ── Phase 1：全速仿真，缓存帧序列 ─────────────────────────────────────
    let mut frames: Vec<RenderFrame> = Vec::new();
    let mut bounce_count = 0u32;
    let mut last_collision: Option<CollisionSnapshot> = None;
    let mut bounce_flash_remaining: u32 = 0;

    let sim_start = Instant::now();

    loop {
        world.step_with(SIM_DT, |event, _world| {
            if let Some(c) = event.downcast::<GroundCollisionEvent>() {
                bounce_count += 1;
                bounce_flash_remaining = 8;
                last_collision = Some(CollisionSnapshot {
                    impact_velocity: c.impact_velocity,
                    restitution: c.restitution,
                });
            }
        });

        let y = world.get::<Position>(ball_id).map_or(0.0, |p| p.y);
        let vy = world.get::<Velocity>(ball_id).map_or(0.0, |v| v.vy);
        let sim_time = world.sim_time();

        let just_bounced = bounce_flash_remaining > 0;
        bounce_flash_remaining = bounce_flash_remaining.saturating_sub(1);

        frames.push(RenderFrame {
            sim_time,
            y,
            vy,
            bounce_count,
            last_collision,
            just_bounced,
        });

        if y <= REST_HEIGHT_THRESHOLD && vy.abs() < REST_VELOCITY_THRESHOLD {
            break;
        }
        if sim_time >= MAX_SIM_TIME {
            break;
        }
    }

    let sim_elapsed = sim_start.elapsed();

    // ── Phase 2：按 sim_time 时间戳回放 ───────────────────────────────────
    let display = match FreeFallDisplay::new(BALL_INITIAL_HEIGHT) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("无法初始化终端显示: {e}");
            return;
        }
    };

    let playback_start = Instant::now();

    for frame in &frames {
        let target = playback_start + Duration::from_secs_f64(frame.sim_time);
        let now = Instant::now();
        if target > now {
            std::thread::sleep(target - now);
        }
        display.render(frame).ok();
    }

    std::thread::sleep(Duration::from_secs(2));
    drop(display);

    println!("=== 仿真统计 ===");
    println!("  仿真时间：{:.2} s", world.sim_time());
    println!("  总帧数：  {}", frames.len());
    println!("  弹跳次数：{}", bounce_count);
    println!("  仿真耗时：{:.2} ms", sim_elapsed.as_secs_f64() * 1000.0);
    println!("=== 仿真完成 ===");
}
