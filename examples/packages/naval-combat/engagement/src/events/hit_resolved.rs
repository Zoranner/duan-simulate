use duan::EntityId;
use duan_macros::Event;

#[derive(Event, Debug)]
#[event(id = "hit-resolved", label = "Hit Resolved")]
pub struct HitResolved {
    pub target_id: EntityId,
    pub damage: f64,
}
