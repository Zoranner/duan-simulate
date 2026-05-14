#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunnerProject {
    pub name: String,
    pub version: String,
    pub registry: RegistryConfig,
    pub scenario_path: String,
    pub dependencies: Vec<CrateDependency>,
    pub installs: Vec<PackageInstall>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistryConfig {
    pub name: String,
    pub index: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrateDependency {
    pub name: String,
    pub version: String,
    pub registry: String,
}

impl CrateDependency {
    pub fn registry(
        name: impl Into<String>,
        version: impl Into<String>,
        registry: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            registry: registry.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageInstall {
    pub crate_name: String,
}

impl PackageInstall {
    pub fn new(crate_name: impl Into<String>) -> Self {
        Self {
            crate_name: crate_name.into(),
        }
    }
}
