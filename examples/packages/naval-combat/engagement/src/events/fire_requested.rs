use duan::EntityId;
use duan_macros::Event;

#[derive(Event, Debug)]
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
