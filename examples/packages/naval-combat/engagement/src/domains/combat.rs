use duan::{Domain, DomainContext, EntityId, Reaction, World};
use example_naval_core::{Faction, Health, Radar};
use example_naval_motion::Position2;

use crate::Weapon;

#[derive(duan::Event, Debug)]
#[event(id = "fire-requested", label = "Fire Requested")]
pub struct FireRequested {
    pub shooter_id: EntityId,
    pub target_id: EntityId,
    pub launch_x: f64,
    pub launch_y: f64,
    pub dir_x: f64,
    pub dir_y: f64,
    pub missile_speed: f64,
    pub damage: f64,
}

#[derive(duan::Event, Debug)]
#[event(id = "hit-resolved", label = "Hit Resolved")]
pub struct HitResolved {
    pub target_id: EntityId,
    pub damage: f64,
}

pub struct CombatDomain;

#[duan::domain(
    id = "combat",
    label = "Combat",
    writes(Weapon),
    reads(Position2, Faction, Health, Radar)
)]
impl Domain for CombatDomain {
    fn compute(&mut self, ctx: &mut DomainContext<Self>, delta_time: f64) {
        let weapon_ids: Vec<_> = ctx.each_mut::<Weapon>().map(|(id, _)| id).collect();

        for id in &weapon_ids {
            if let Some(weapon) = ctx.get_mut::<Weapon>(*id) {
                weapon.cooldown_remaining = (weapon.cooldown_remaining - delta_time).max(0.0);
            }
        }

        let positions: Vec<_> = ctx
            .each::<Position2>()
            .map(|(id, position)| (id, position.x, position.y))
            .collect();
        let factions: Vec<_> = ctx
            .each::<Faction>()
            .map(|(id, faction)| (id, faction.team))
            .collect();

        for shooter_id in weapon_ids {
            let Some((sx, sy)) = positions
                .iter()
                .find(|(id, _, _)| *id == shooter_id)
                .map(|(_, x, y)| (*x, *y))
            else {
                continue;
            };
            let Some(my_team) = factions
                .iter()
                .find(|(id, _)| *id == shooter_id)
                .map(|(_, team)| *team)
            else {
                continue;
            };
            let Some(radar_range) = ctx.get::<Radar>(shooter_id).map(|radar| radar.range) else {
                continue;
            };

            let Some((target_id, tx, ty)) = positions
                .iter()
                .filter(|(id, _, _)| *id != shooter_id)
                .filter(|(id, _, _)| {
                    factions
                        .iter()
                        .find(|(faction_id, _)| faction_id == id)
                        .is_some_and(|(_, team)| *team != my_team)
                })
                .filter(|(id, _, _)| {
                    ctx.get::<Health>(*id)
                        .is_some_and(|health| !health.is_dead())
                })
                .filter(|(_, tx, ty)| {
                    let dx = *tx - sx;
                    let dy = *ty - sy;
                    (dx * dx + dy * dy).sqrt() <= radar_range
                })
                .min_by(|(_, ax, ay), (_, bx, by)| {
                    let ad = (*ax - sx).hypot(*ay - sy);
                    let bd = (*bx - sx).hypot(*by - sy);
                    ad.total_cmp(&bd)
                })
                .copied()
            else {
                continue;
            };

            let Some((range, missile_speed, damage)) =
                ctx.get::<Weapon>(shooter_id).and_then(|weapon| {
                    weapon
                        .is_ready()
                        .then_some((weapon.range, weapon.missile_speed, weapon.damage))
                })
            else {
                continue;
            };

            let dx = tx - sx;
            let dy = ty - sy;
            if dx.hypot(dy) > range {
                continue;
            }

            let Some(weapon) = ctx.get_mut::<Weapon>(shooter_id) else {
                continue;
            };
            weapon.cooldown_remaining = weapon.fire_cooldown;
            ctx.emit(FireRequested {
                shooter_id,
                target_id,
                launch_x: sx,
                launch_y: sy,
                dir_x: dx,
                dir_y: dy,
                missile_speed,
                damage,
            });
        }
    }
}

pub struct ApplyDamage;

#[duan::reaction(id = "apply-damage", label = "Apply Damage", event = HitResolved)]
impl Reaction<HitResolved> for ApplyDamage {
    fn react(&mut self, event: &HitResolved, world: &mut World) {
        if let Some(health) = world.inspect_mut::<Health>(event.target_id) {
            health.current = (health.current - event.damage).max(0.0);
        }
    }
}
