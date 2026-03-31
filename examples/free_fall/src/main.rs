//! 自由落体小球仿真 — 主程序
//!
//! 本示例展示 DUAN 框架新一代 API 的完整使用流程：
//! 1. `World::builder().domain(...).apply(handlers::install(&app)).build()` 构建仿真世界
//! 2. `world.spawn_with::<Ball>(...)` 生成带运行时组件的实体
//! 3. `world.step(delta_time)` 推进仿真；事件由注册的 Observer 自动处理
//! 4. `world.get::<Position>(id)` 读取实体组件状态
//!
//! # 两阶段设计
//!
//! - **Phase 1（仿真）**：全速推进，每步记录 `RenderFrame` 到帧缓冲
//! - **Phase 2（回放）**：按帧的 `time` 以真实时钟定时渲染

mod display;
mod handlers;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use free_fall::components::{Collider, Position, StaticBody, Velocity};
use free_fall::domains::MotionDomain;
use free_fall::entities::{Ball, Ground};

use display::{CollisionSnapshot, FreeFallDisplay, RenderFrame};

const BALL_INITIAL_HEIGHT: f64 = 10.0;
const DELTA_TIME: f64 = 0.01;
const MAX_TIME: f64 = 20.0;
const REST_HEIGHT_THRESHOLD: f64 = 0.01;
const REST_VELOCITY_THRESHOLD: f64 = 0.1;

// 在事件处理器（handlers 模块）与主循环之间共享的展示层状态
pub(crate) struct AppState {
    pub(crate) bounce_count: u32,
    pub(crate) bounce_flash_remaining: u32,
    pub(crate) last_collision: Option<CollisionSnapshot>,
}

fn main() {
    let app = Arc::new(Mutex::new(AppState {
        bounce_count: 0,
        bounce_flash_remaining: 0,
        last_collision: None,
    }));

    // ── 构建仿真世界 ───────────────────────────────────────────────────────
    // handlers::install 将所有事件处理器封装为独立模块，通过 .apply() 装配
    let mut world = duan::World::builder()
        .domain(MotionDomain::earth())
        .apply(handlers::install(&app))
        .build();

    // ── 生成实体 ───────────────────────────────────────────────────────────
    // 地面（y=0）：StaticBody 标记使运动域跳过此实体
    world.spawn_with::<Ground>((
        Position::new(0.0, 0.0),
        StaticBody,
        Collider::new(0.8),
    ));

    // 小球：从初始高度自由落体；BounceCount Memory 由 Ball::bundle() 提供默认值
    let ball_id = world.spawn_with::<Ball>((
        Position::new(0.0, BALL_INITIAL_HEIGHT),
        Velocity::new(0.0, 0.0),
    ));

    // ── Phase 1：全速仿真，缓存帧序列 ─────────────────────────────────────
    let mut frames: Vec<RenderFrame> = Vec::new();

    // Phase 1 墙钟起点（与 `world.time()` 的世界时间无关）
    let phase1_wall_start = Instant::now();

    loop {
        world.step(DELTA_TIME);

        let y = world.get::<Position>(ball_id).map_or(0.0, |p| p.y);
        let vy = world.get::<Velocity>(ball_id).map_or(0.0, |v| v.vy);
        let time = world.time();

        let (bounce_count, last_collision, just_bounced) = {
            let mut s = app.lock().unwrap();
            let just_bounced = s.bounce_flash_remaining > 0;
            s.bounce_flash_remaining = s.bounce_flash_remaining.saturating_sub(1);
            (s.bounce_count, s.last_collision, just_bounced)
        };

        frames.push(RenderFrame {
            time,
            y,
            vy,
            bounce_count,
            last_collision,
            just_bounced,
        });

        if y <= REST_HEIGHT_THRESHOLD && vy.abs() < REST_VELOCITY_THRESHOLD {
            break;
        }
        if time >= MAX_TIME {
            break;
        }
    }

    let phase1_wall_duration = phase1_wall_start.elapsed();

    // ── Phase 2：按 time 时间戳回放 ───────────────────────────────────
    let display = match FreeFallDisplay::new(BALL_INITIAL_HEIGHT) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("无法初始化终端显示: {e}");
            return;
        }
    };

    let playback_start = Instant::now();

    for frame in &frames {
        let target = playback_start + Duration::from_secs_f64(frame.time);
        let now = Instant::now();
        if target > now {
            std::thread::sleep(target - now);
        }
        display.render(frame).ok();
    }

    std::thread::sleep(Duration::from_secs(2));
    drop(display);

    let final_bounce_count = app.lock().unwrap().bounce_count;
    println!("=== 仿真统计 ===");
    println!("  仿真时间：{:.2} s", world.time());
    println!("  总帧数：  {}", frames.len());
    println!("  弹跳次数：{}", final_bounce_count);
    println!(
        "  Phase 1 墙钟耗时：{:.2} ms",
        phase1_wall_duration.as_secs_f64() * 1000.0
    );
    println!("=== 仿真完成 ===");
}
