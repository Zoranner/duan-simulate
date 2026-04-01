//! 舰队对抗仿真 — 主程序
//!
//! 红蓝两支舰队（各 SHIPS_PER_SIDE 艘）在 2D 海域对峙。
//!
//! # 两阶段设计
//!
//! - **Phase 1（仿真）**：全速推进，每步记录 `RenderFrame` 到帧缓冲
//! - **Phase 2（回放）**：按帧的 `time` 以真实时钟 1:1 定时渲染

mod display;
mod handlers;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use duan::diagnostics::{FramePhase, LogLevel, LogRecord, LogSink};
use duan::{EntityId, World};
use naval_combat::components::{Faction, Health, Helm, Position, Radar, Velocity, Weapon};
use naval_combat::domains::{CollisionDomain, CombatDomain, MotionDomain};
use naval_combat::entities::Ship;
use rand::Rng;

use display::{CombatLog, MissileDot, NavalDisplay, RenderFrame, ShipFrame};

const DELTA_TIME: f64 = 0.05;
const MAX_TIME: f64 = 120.0;
const SHIPS_PER_SIDE: usize = 5;

/// 呈现层共享的仿真输出，由多个事件处理器共同读写
pub(crate) struct SimulationOutput {
    pub(crate) log: CombatLog,
    pub(crate) missile_ids: Vec<EntityId>,
    pub(crate) total_missiles: u32,
    pub(crate) total_hits: u32,
}

struct Archetype {
    tag: &'static str,
    hp: f64,
    radar: f64,
    weapon_range: f64,
    weapon_damage: f64,
    weapon_cooldown: f64,
    missile_speed: f64,
    turn_rate: f64,
}


fn main() {
    let archetypes = [
        Archetype {
            tag: "驱",
            hp: 150.0,
            radar: 280.0,
            weapon_range: 200.0,
            weapon_damage: 50.0,
            weapon_cooldown: 8.0,
            missile_speed: 70.0,
            turn_rate: std::f64::consts::PI / 3.0,
        },
        Archetype {
            tag: "巡",
            hp: 320.0,
            radar: 300.0,
            weapon_range: 250.0,
            weapon_damage: 90.0,
            weapon_cooldown: 15.0,
            missile_speed: 55.0,
            turn_rate: std::f64::consts::PI / 6.0,
        },
        Archetype {
            tag: "护",
            hp: 100.0,
            radar: 340.0,
            weapon_range: 180.0,
            weapon_damage: 40.0,
            weapon_cooldown: 5.0,
            missile_speed: 80.0,
            turn_rate: std::f64::consts::PI / 2.0,
        },
    ];

    let mut rng = rand::thread_rng();

    // ── 构建统一日志后端 ──────────────────────────────────────────────────
    // 三层观察语义：
    //   Info  — 默认层，关键业务事件（开火/命中/击毁/事件分发摘要）
    //   Debug — 框架层，每帧边界、阶段汇总、定时器、生命周期变化
    //   Trace — 热路径层，逐实体 tick、逐域执行明细
    //
    // level 作为主过滤轴，phase/target 作为辅助屏蔽噪音：
    //   - Info 过滤掉纯框架 Debug 打点（StepStart/StepEnd 在 Debug 层）
    //   - Debug 开放全部框架摘要，同时过滤热路径 Trace 明细
    struct DebugLogger {
        min_level: LogLevel,
    }
    impl LogSink for DebugLogger {
        fn enabled(&self, level: LogLevel) -> bool {
            level >= self.min_level
        }
        fn log(&self, record: &LogRecord) {
            use LogLevel::{Debug, Info};
            let show = match record.level {
                // Info 层：只显示关键业务事件与事件分发摘要，屏蔽框架阶段噪音
                Info => match record.phase() {
                    FramePhase::DomainCompute => record.target.starts_with("naval_combat::"),
                    FramePhase::EventDispatch => true,
                    _ => false,
                },
                // Debug 层：框架阶段边界、汇总信息全显示（热路径 Trace 由 enabled 拦截）
                Debug => true,
                // Warn/Error：始终显示
                _ => record.level > Debug,
            };
            if !show {
                return;
            }
            let entity_str = record
                .entity_id()
                .map(|id| format!(" entity={id}"))
                .unwrap_or_default();
            eprintln!(
                "[{:.3}][{:>5}][{:<13}]{} {}",
                record.time(),
                record.level,
                record.phase(),
                entity_str,
                record.message
            );
        }
    }

    // ── 初始化展示模型 ────────────────────────────────────────────────────
    let simulation_output = Arc::new(Mutex::new(SimulationOutput {
        log: CombatLog::new(),
        missile_ids: Vec::new(),
        total_missiles: 0,
        total_hits: 0,
    }));

    // ── 构建仿真世界 ──────────────────────────────────────────────────────
    let mut world = World::builder()
        .logger(Arc::new(DebugLogger {
            min_level: LogLevel::Info,
        }))
        .domain(MotionDomain)
        .domain(CombatDomain)
        .domain(CollisionDomain)
        .apply(handlers::install(&simulation_output))
        .build();

    // ── 随机生成舰队 ──────────────────────────────────────────────────────
    let letters = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'];

    let mut ship_ids: Vec<EntityId> = Vec::new();
    let mut ship_names: Vec<String> = Vec::new();
    let mut ship_teams: Vec<u8> = Vec::new();
    let mut ship_max_hp: Vec<f64> = Vec::new();
    let mut last_ship_pos: Vec<(f64, f64)> = Vec::new();

    // 红方
    for letter in letters.iter().take(SHIPS_PER_SIDE) {
        let arch = &archetypes[rng.gen_range(0..archetypes.len())];
        let x = rng.gen_range(100.0_f64..900.0);
        let y = rng.gen_range(80.0_f64..250.0);
        let vx = rng.gen_range(-2.0_f64..2.0);
        let vy = rng.gen_range(8.0_f64..14.0);
        let name = format!("红-{}{}", arch.tag, letter);

        let id = world.spawn_with::<Ship>((
            Position::new(x, y),
            Velocity::new(vx, vy),
            Faction::red(),
            Radar::new(arch.radar),
            Weapon::new(
                arch.weapon_range,
                arch.weapon_damage,
                arch.weapon_cooldown,
                arch.missile_speed,
            ),
            Health::new(arch.hp),
            Helm::new(arch.turn_rate),
        ));

        simulation_output.lock().unwrap().log.register_name(id, &name);
        ship_ids.push(id);
        ship_names.push(name);
        ship_teams.push(0);
        ship_max_hp.push(arch.hp);
        last_ship_pos.push((x, y));
    }

    // 蓝方
    for letter in letters.iter().take(SHIPS_PER_SIDE) {
        let arch = &archetypes[rng.gen_range(0..archetypes.len())];
        let x = rng.gen_range(100.0_f64..900.0);
        let y = rng.gen_range(750.0_f64..920.0);
        let vx = rng.gen_range(-2.0_f64..2.0);
        let vy = -rng.gen_range(8.0_f64..14.0);
        let name = format!("蓝-{}{}", arch.tag, letter);

        let id = world.spawn_with::<Ship>((
            Position::new(x, y),
            Velocity::new(vx, vy),
            Faction::blue(),
            Radar::new(arch.radar),
            Weapon::new(
                arch.weapon_range,
                arch.weapon_damage,
                arch.weapon_cooldown,
                arch.missile_speed,
            ),
            Health::new(arch.hp),
            Helm::new(arch.turn_rate),
        ));

        simulation_output.lock().unwrap().log.register_name(id, &name);
        ship_ids.push(id);
        ship_names.push(name);
        ship_teams.push(1);
        ship_max_hp.push(arch.hp);
        last_ship_pos.push((x, y));
    }

    let total_ships = ship_ids.len();

    let mut winner: Option<u8> = None;

    // ── Phase 1：全速仿真 ────────────────────────────────────────────────
    let mut frames: Vec<RenderFrame> = Vec::new();

    loop {
        world.step(DELTA_TIME);

        {
            let mut s = simulation_output.lock().unwrap();
            s.log.drain_to_recent();
            s.missile_ids.retain(|&id| world.is_alive(id));
        }

        // 收集舰船事实与位置等快照数据
        let mut ships = Vec::with_capacity(total_ships);
        for (i, id) in ship_ids.iter().enumerate() {
            let name = &ship_names[i];
            let team = ship_teams[i];
            if world.is_alive(*id) {
                let pos = world
                    .get::<Position>(*id)
                    .map(|p| (p.x, p.y))
                    .unwrap_or(last_ship_pos[i]);
                let (hp, max_hp) = world
                    .get::<Health>(*id)
                    .map(|h| (h.current, h.max))
                    .unwrap_or((0.0, ship_max_hp[i]));
                last_ship_pos[i] = pos;
                ships.push(ShipFrame {
                    name: name.clone(),
                    x: pos.0,
                    y: pos.1,
                    health: hp,
                    max_health: max_hp,
                    team,
                    alive: true,
                });
            } else {
                ships.push(ShipFrame {
                    name: name.clone(),
                    x: last_ship_pos[i].0,
                    y: last_ship_pos[i].1,
                    health: 0.0,
                    max_health: ship_max_hp[i],
                    team,
                    alive: false,
                });
            }
        }

        let missiles: Vec<MissileDot> = {
            let s = simulation_output.lock().unwrap();
            s.missile_ids
                .iter()
                .filter_map(|&id| {
                    let pos = world.get::<Position>(id)?;
                    let team = world.get::<Faction>(id).map(|f| f.team).unwrap_or(0);
                    Some(MissileDot {
                        x: pos.x,
                        y: pos.y,
                        team,
                    })
                })
                .collect()
        };

        let (recent_log, total_missiles, total_hits) = {
            let s = simulation_output.lock().unwrap();
            (s.log.recent_log(), s.total_missiles, s.total_hits)
        };

        frames.push(RenderFrame {
            time: world.time(),
            ships,
            missiles,
            recent_log,
            active_missile_count: simulation_output.lock().unwrap().missile_ids.len(),
            total_missiles,
            total_hits,
        });

        let red_alive = ship_ids[..SHIPS_PER_SIDE]
            .iter()
            .any(|&id| world.is_alive(id));
        let blue_alive = ship_ids[SHIPS_PER_SIDE..]
            .iter()
            .any(|&id| world.is_alive(id));

        if !red_alive || !blue_alive {
            winner = if !red_alive && !blue_alive {
                None
            } else if red_alive {
                Some(0u8)
            } else {
                Some(1u8)
            };
            break;
        }

        if world.time() >= MAX_TIME {
            break;
        }
    }

    let final_time = world.time();
    let (final_total_missiles, final_total_hits) = {
        let s = simulation_output.lock().unwrap();
        (s.total_missiles, s.total_hits)
    };

    // ── Phase 2：按 time 1:1 回放 ────────────────────────────────────
    let display = match NavalDisplay::new() {
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

    std::thread::sleep(Duration::from_secs(3));
    drop(display);

    println!("=== 舰队对抗仿真完成 ===");
    match winner {
        Some(0) => println!("  胜利方：红方"),
        Some(1) => println!("  胜利方：蓝方"),
        Some(_) => println!("  结果：未知"),
        None => {
            if final_time >= MAX_TIME {
                println!("  结果：超时（平局）");
            } else {
                println!("  结果：双方同归于尽");
            }
        }
    }
    println!("  仿真时长：{final_time:.1}s");
    println!("  总发射导弹：{final_total_missiles}");
    println!("  总命中次数：{final_total_hits}");
    println!("  总帧数：{}", frames.len());
    println!("========================");
}
