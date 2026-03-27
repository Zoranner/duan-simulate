//! 自由落体小球仿真 — 主程序
//!
//! Phase 1：仿真全速推进，每步记录 RenderFrame 到缓冲区。
//! Phase 2：按帧的 sim_time 时间戳以真实时钟回放，精确还原物理时间序列。

mod display;

use std::time::{Duration, Instant};

use duan::{CustomEvent, Entity, World};
use free_fall::components::{Collider, Mass, Position, Velocity};
use free_fall::domains::{CollisionRules, MotionRules};
use free_fall::events::GroundCollisionEvent;

use display::{FreeFallDisplay, RenderFrame};

fn main() {
    // ── 参数 ────────────────────────────────────────────────
    let dt = 0.01; // 仿真时间步（秒）
    let total_time = 20.0;

    // ── 仿真世界 ───────────────────────────────────────────
    let mut world = World::builder().time_scale(1.0).build();
    world.register_domain("motion", MotionRules::earth());
    world.register_domain("collision", CollisionRules::new());

    world.spawn(
        Entity::new("ground")
            .with_domain("collision")
            .with_component(Position::new(0.0, 0.0, 0.0))
            .with_component(Collider::ground(0.8, 0.05)),
    );

    let ball_id = world.spawn(
        Entity::new("ball")
            .with_domain("motion")
            .with_domain("collision")
            .with_component(Position::new(0.0, 10.0, 0.0))
            .with_component(Velocity::new(0.0, 0.0, 0.0))
            .with_component(Collider::new("小球", 0.0, 0.8, 0.05))
            .with_component(Mass::new(1.0)),
    );

    // ── Phase 1：全速仿真，缓存帧序列 ─────────────────────
    let mut frames: Vec<RenderFrame> = Vec::new();
    let mut bounce_count = 0u32;
    let mut last_collision: Option<(f64, f64)> = None;

    let sim_start = Instant::now();

    loop {
        world.step_with(dt, |event: &dyn CustomEvent, _world: &mut World| {
            if let Some(c) = event.as_any().downcast_ref::<GroundCollisionEvent>() {
                bounce_count += 1;
                last_collision = Some((c.impact_velocity, c.restitution));
            }
        });

        let entity = match world.get_entity(ball_id) {
            Some(e) => e,
            None => break,
        };
        let y = entity
            .get_component::<Position>()
            .map(|p| p.y)
            .unwrap_or(0.0);
        let vy = entity
            .get_component::<Velocity>()
            .map(|v| v.vy)
            .unwrap_or(0.0);
        let sim_time = world.sim_time();

        frames.push(RenderFrame {
            sim_time,
            y,
            vy,
            bounce_count,
            last_collision,
        });

        if y <= 0.01 && vy.abs() < 0.1 {
            break;
        }
        if sim_time >= total_time {
            break;
        }
    }

    let sim_elapsed = sim_start.elapsed();

    // ── Phase 2：按 sim_time 时间戳回放 ───────────────────
    let display = match FreeFallDisplay::new(10.0) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("无法初始化终端显示: {}", e);
            return;
        }
    };

    let playback_start = Instant::now();

    for frame in &frames {
        // 等到该帧对应的真实时刻
        let target = playback_start + Duration::from_secs_f64(frame.sim_time);
        let now = Instant::now();
        if target > now {
            std::thread::sleep(target - now);
        }

        display.render(frame).ok();
    }

    // 最终帧停留 2 秒
    std::thread::sleep(Duration::from_secs(2));
    drop(display);

    // ── 统计 ───────────────────────────────────────────────
    println!("=== 仿真统计 ===");
    println!("  仿真时间：{:.2} s", world.sim_time());
    println!("  总帧数：{}", frames.len());
    println!("  仿真耗时：{:.2} ms", sim_elapsed.as_secs_f64() * 1000.0);
    println!("=== 仿真完成 ===");
}
