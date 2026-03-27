//! 舰队对抗仿真 — 主程序
//!
//! 红蓝两支舰队（各 2 艘）在 2D 海域对峙。
//! 场景：双方相向而行，雷达探测到对方后开火发射导弹，导弹追踪目标直至命中。
//!
//! # 域依赖与执行顺序
//!
//! faction → space → motion → detection → combat → collision
//!
//! # 事件处理
//!
//! - FireEvent   → spawn 导弹实体
//! - HitEvent    → 销毁导弹 + 扣减目标 Health
//! - ShipDestroyedEvent → 销毁舰船

mod display;

use duan::{Entity, EntityId, World};
use naval_combat::components::{
    Faction, Health, MissileBody, Position, Radar, Seeker, Velocity, Weapon,
};
use naval_combat::domains::{
    CollisionRules, CombatRules, DetectionRules, FactionRules, MotionRules, SpaceRules,
};
use naval_combat::events::{FireEvent, HitEvent, ShipDestroyedEvent};

use display::{CombatLog, LogEntry, ShipStatus};

const SIM_DT: f64 = 0.1;
const MAX_SIM_TIME: f64 = 120.0;
const STATUS_INTERVAL: u32 = 50; // 每 50 帧打印一次态势（5s）

fn main() {
    // ── 构建仿真世界 ─────────────────────────────────────
    let mut world = World::builder()
        .with_domain("faction", FactionRules::red_vs_blue())
        .with_domain("space", SpaceRules::new())
        .with_domain("motion", MotionRules::new())
        .with_domain("detection", DetectionRules::new())
        .with_domain("combat", CombatRules::new())
        .with_domain("collision", CollisionRules::new())
        .build();

    // ── 生成舰船实体 ────────────────────────────────────
    // 红方：A 舰和 B 舰，初始在南方，向北移动
    let red_a = world.spawn(
        Entity::new("destroyer")
            .with_domain("faction")
            .with_domain("space")
            .with_domain("motion")
            .with_domain("detection")
            .with_domain("combat")
            .with_component(Position::new(0.0, 0.0))
            .with_component(Velocity::new(0.0, 5.0))
            .with_component(Faction::red())
            .with_component(Radar::new(300.0))
            .with_component(Weapon::new(250.0, 60.0, 5.0, 60.0))
            .with_component(Health::new(200.0)),
    );

    let red_b = world.spawn(
        Entity::new("destroyer")
            .with_domain("faction")
            .with_domain("space")
            .with_domain("motion")
            .with_domain("detection")
            .with_domain("combat")
            .with_component(Position::new(60.0, 0.0))
            .with_component(Velocity::new(0.0, 5.0))
            .with_component(Faction::red())
            .with_component(Radar::new(300.0))
            .with_component(Weapon::new(250.0, 60.0, 5.0, 60.0))
            .with_component(Health::new(200.0)),
    );

    // 蓝方：C 舰和 D 舰，初始在北方，向南移动
    let blue_c = world.spawn(
        Entity::new("frigate")
            .with_domain("faction")
            .with_domain("space")
            .with_domain("motion")
            .with_domain("detection")
            .with_domain("combat")
            .with_component(Position::new(0.0, 400.0))
            .with_component(Velocity::new(0.0, -5.0))
            .with_component(Faction::blue())
            .with_component(Radar::new(300.0))
            .with_component(Weapon::new(250.0, 60.0, 5.0, 60.0))
            .with_component(Health::new(200.0)),
    );

    let blue_d = world.spawn(
        Entity::new("frigate")
            .with_domain("faction")
            .with_domain("space")
            .with_domain("motion")
            .with_domain("detection")
            .with_domain("combat")
            .with_component(Position::new(60.0, 400.0))
            .with_component(Velocity::new(0.0, -5.0))
            .with_component(Faction::blue())
            .with_component(Radar::new(300.0))
            .with_component(Weapon::new(250.0, 60.0, 5.0, 60.0))
            .with_component(Health::new(200.0)),
    );

    let ship_ids = [red_a, red_b, blue_c, blue_d];
    let ship_names = ["红-A舰", "红-B舰", "蓝-C舰", "蓝-D舰"];

    // ── 初始化战斗日志 ──────────────────────────────────
    let mut log = CombatLog::new();
    for (&id, &name) in ship_ids.iter().zip(ship_names.iter()) {
        log.register_name(id, name);
    }

    let mut total_missiles: u32 = 0;
    let mut total_hits: u32 = 0;
    let mut frame_count: u32 = 0;

    println!("=== 舰队对抗仿真开始 ===");
    println!("红方（A/B）从南向北，蓝方（C/D）从北向南，相向而行");
    println!("雷达范围 300m，武器射程 250m，导弹速度 60m/s，伤害 60");
    println!();

    // ── 仿真主循环 ──────────────────────────────────────
    loop {
        let sim_time = world.sim_time();

        // 收集本帧事件数据（在 step_with 前读取，为事件处理准备名称）
        world.step_with(SIM_DT, |event, world| {
            let t = world.sim_time();

            if let Some(e) = event.downcast::<FireEvent>() {
                total_missiles += 1;
                let shooter_name = log.get_name(e.shooter_id);
                let target_name = log.get_name(e.target_id);

                // spawn 导弹
                let missile = Entity::new("missile")
                    .with_domain("space")
                    .with_domain("motion")
                    .with_domain("collision")
                    .with_component(Position::new(e.launch_x, e.launch_y))
                    .with_component(Velocity::towards(e.dir_x, e.dir_y, e.missile_speed))
                    .with_component(Seeker::new(e.target_id, e.shooter_id, e.damage))
                    .with_component(MissileBody)
                    .with_component(Faction {
                        team: world
                            .get_entity(e.shooter_id)
                            .and_then(|en| en.get_component::<Faction>())
                            .map(|f| f.team)
                            .unwrap_or(99),
                    });
                let missile_id = world.spawn(missile);
                log.register_name(missile_id, format!("导弹({}→{})", shooter_name, target_name));
                log.log(t, LogEntry::Fire { shooter: shooter_name, target: target_name });
            } else if let Some(e) = event.downcast::<HitEvent>() {
                total_hits += 1;
                let target_name = log.get_name(e.target_id);

                // 销毁导弹
                world.destroy(e.missile_id, 0.0);

                // 扣减目标 Health
                let health_after = if let Some(target) = world.get_entity_mut(e.target_id) {
                    if let Some(health) = target.get_component_mut::<Health>() {
                        health.current -= e.damage;
                        health.current.max(0.0)
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };

                log.log(t, LogEntry::Hit {
                    target: target_name,
                    damage: e.damage,
                    health_after,
                });
            } else if let Some(e) = event.downcast::<ShipDestroyedEvent>() {
                let name = log.get_name(e.ship_id);
                world.destroy(e.ship_id, 0.5);
                log.log(t, LogEntry::ShipDestroyed { name });
            }
        });

        // 打印本帧日志
        log.flush();

        frame_count += 1;

        // 定期打印态势
        if frame_count % STATUS_INTERVAL == 0 {
            let ships: Vec<ShipStatus> = ship_ids
                .iter()
                .zip(ship_names.iter())
                .filter_map(|(&id, &name)| {
                    let entity = world.get_entity(id)?;
                    let pos = entity.get_component::<Position>()?;
                    let health = entity.get_component::<Health>()?;
                    let team = entity.get_component::<Faction>().map(|f| f.team).unwrap_or(0);
                    Some(ShipStatus {
                        name: name.to_string(),
                        team,
                        x: pos.x,
                        y: pos.y,
                        health: health.current,
                        max_health: health.max,
                    })
                })
                .collect();
            display::print_status(world.sim_time(), &ships);
        }

        // 终止条件：检查各阵营存活情况
        let red_alive = ship_ids[0..2]
            .iter()
            .any(|&id| world.get_entity(id).is_some());
        let blue_alive = ship_ids[2..4]
            .iter()
            .any(|&id| world.get_entity(id).is_some());

        if !red_alive || !blue_alive {
            let winner = if !red_alive && !blue_alive {
                None
            } else if red_alive {
                Some(0u8)
            } else {
                Some(1u8)
            };
            display::print_summary(winner, world.sim_time(), total_missiles, total_hits);
            break;
        }

        if sim_time >= MAX_SIM_TIME {
            display::print_summary(None, world.sim_time(), total_missiles, total_hits);
            break;
        }
    }
}
