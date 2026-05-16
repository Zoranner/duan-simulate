pub mod domains;
mod package;

pub use domains::GravityField;
pub use package::package;

pub fn register_factories(
    registry: duan::catalog::FactoryRegistry,
) -> duan::catalog::PackageResult<duan::catalog::FactoryRegistry> {
    registry.with_domain(GravityFieldFactory)
}

struct GravityFieldFactory;

impl duan::catalog::DomainFactory for GravityFieldFactory {
    fn item_id(&self) -> &'static str {
        "example-freefall-gravity/earth"
    }

    fn describe(&self) -> String {
        "free-fall gravity field".to_owned()
    }

    fn install(
        &self,
        builder: duan::WorldBuilder,
        _options: &std::collections::BTreeMap<String, duan::catalog::Value>,
    ) -> duan::catalog::PackageResult<duan::WorldBuilder> {
        Ok(builder.domain(GravityField::earth()))
    }
}
