pub mod components;
pub mod domains;
pub mod events;
mod package;
pub mod reactions;

pub use components::Weapon;
pub use domains::CombatDomain;
pub use events::{FireRequested, HitResolved};
pub use package::{package, PACKAGE_ID};
pub use reactions::ApplyDamage;
