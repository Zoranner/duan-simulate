//! 舰队对抗仿真 — 主程序
//!
//! 红蓝两支舰队（各 SHIPS_PER_SIDE 艘）在 2D 海域对峙。
//!
//! # 两阶段设计
//!
//! - **Phase 1（仿真）**：全速推进，每步记录 `RenderFrame` 到帧缓冲
//! - **Phase 2（回放）**：按帧的 `sim_time` 以真实时钟 1:1 定时渲染

mod display;

use std::time::{Duration, Instant};

use duan::EntityId;
use naval_combat::components::{
    Faction, Health, Helm, MissileBody, Position, Radar, Seeker, Velocity, Weapon,
};
use naval_combat::domains::{CollisionDomain, CombatDomain, MotionDomain};
use naval_combat::entities::{Missile, Ship};
use naval_combat::events::{FireEvent, HitEvent, MissileExpiredEvent, ShipDestroyedEvent};
use rand::Rng;

use display::{CombatLog, LogEntry, MissileDot, NavalDisplay, RenderFrame, ShipFrame};

const SIM_DT: f64 = 0.05;
const MAX_SIM_TIME: f64 = 120.0;
const SHIPS_PER_SIDE: usize = 5;

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

    // ── 构建仿真世界 ──────────────────────────────────────────────────────
    let mut world = duan::World::builder()
        .with_domain(MotionDomain)
        .with_domain(CombatDomain)
        .with_domain(CollisionDomain)
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

        ship_ids.push(id);
        ship_names.push(name);
        ship_teams.push(1);
        ship_max_hp.push(arch.hp);
        last_ship_pos.push((x, y));
    }

    let total_ships = ship_ids.len();

    // ── 仿真状态 ──────────────────────────────────────────────────────────
    let mut log = CombatLog::new();
    for (id, name) in ship_ids.iter().zip(ship_names.iter()) {
        log.register_name(*id, name.as_str());
    }

    let mut missile_ids: Vec<EntityId> = Vec::new();
    let mut total_missiles: u32 = 0;
    let mut total_hits: u32 = 0;
    let mut winner: Option<u8> = None;

    // ── Phase 1：全速仿真 ────────────────────────────────────────────────
    let mut frames: Vec<RenderFrame> = Vec::new();

    loop {
        world.step_with(SIM_DT, |event, world| {
            let t = world.sim_time();

            if let Some(e) = event.downcast::<FireEvent>() {
                total_missiles += 1;
                let shooter_name = log.get_name(e.shooter_id);
                let target_name = log.get_name(e.target_id);

                let missile_id = world.spawn_with::<Missile>((
                    Position::new(e.launch_x, e.launch_y),
                    Velocity::towards(e.dir_x, e.dir_y, e.missile_speed),
                    Seeker::new(e.target_id, e.shooter_id, e.damage, e.missile_range),
                    MissileBody,
                    Faction {
                        team: world
                            .get::<Faction>(e.shooter_id)
                            .map(|f| f.team)
                            .unwrap_or(99),
                    },
                ));

                missile_ids.push(missile_id);
                log.register_name(
                    missile_id,
                    format!("导弹({}→{})", shooter_name, target_name),
                );
                log.log(
                    t,
                    LogEntry::Fire {
                        shooter: shooter_name,
                        target: target_name,
                    },
                );
            } else if let Some(e) = event.downcast::<HitEvent>() {
                total_hits += 1;
                let target_name = log.get_name(e.target_id);
                let health_after = world
                    .get::<Health>(e.target_id)
                    .map(|h| h.current)
                    .unwrap_or(0.0);

                world.destroy(e.missile_id);
                missile_ids.retain(|&id| id != e.missile_id);

                log.log(
                    t,
                    LogEntry::Hit {
                        target: target_name,
                        damage: e.damage,
                        health_after,
                    },
                );
            } else if let Some(e) = event.downcast::<ShipDestroyedEvent>() {
                let name = log.get_name(e.ship_id);
                world.destroy(e.ship_id);
                log.log(t, LogEntry::ShipDestroyed { name });
            } else if let Some(e) = event.downcast::<MissileExpiredEvent>() {
                world.destroy(e.missile_id);
                missile_ids.retain(|&id| id != e.missile_id);
            }
        });

        log.drain_to_recent();
        missile_ids.retain(|&id| world.is_alive(id));

        // 收集舰船状态快照
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

        let missiles: Vec<MissileDot> = missile_ids
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
            .collect();

        let active_missile_count = missiles.len();

        frames.push(RenderFrame {
            sim_time: world.sim_time(),
            ships,
            missiles,
            recent_log: log.recent_log(),
            active_missile_count,
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

        if world.sim_time() >= MAX_SIM_TIME {
            break;
        }
    }

    let final_sim_time = world.sim_time();

    // ── Phase 2：按 sim_time 1:1 回放 ────────────────────────────────────
    let display = match NavalDisplay::new() {
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

    std::thread::sleep(Duration::from_secs(3));
    drop(display);

    println!("=== 舰队对抗仿真完成 ===");
    match winner {
        Some(0) => println!("  胜利方：红方"),
        Some(1) => println!("  胜利方：蓝方"),
        Some(_) => println!("  结果：未知"),
        None => {
            if final_sim_time >= MAX_SIM_TIME {
                println!("  结果：超时（平局）");
            } else {
                println!("  结果：双方同归于尽");
            }
        }
    }
    println!("  仿真时长：{final_sim_time:.1}s");
    println!("  总发射导弹：{total_missiles}");
    println!("  总命中次数：{total_hits}");
    println!("  总帧数：{}", frames.len());
    println!("========================");
}
