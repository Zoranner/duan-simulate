use duan_scenario::Value;

use crate::PackageError;

pub trait ScenarioComponent: Sized {
    fn item_id() -> &'static str;

    fn decode(value: &Value) -> Result<Self, PackageError>;
}

pub trait EntityFactory {
    fn item_id(&self) -> &'static str;

    fn describe(&self) -> String;
}

pub trait DomainFactory {
    fn item_id(&self) -> &'static str;

    fn describe(&self) -> String;
}

pub trait ReactionFactory {
    fn item_id(&self) -> &'static str;

    fn describe(&self) -> String;
}

pub trait ObserverFactory {
    fn item_id(&self) -> &'static str;

    fn describe(&self) -> String;
}
