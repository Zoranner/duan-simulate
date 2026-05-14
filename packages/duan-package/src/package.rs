use serde::{Deserialize, Serialize};

use crate::{ItemId, PackageId, Schema, VersionReqText};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageDependency {
    package_id: PackageId,
    version: VersionReqText,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Package {
    id: PackageId,
    version: String,
    dependencies: Vec<PackageDependency>,
    items: Vec<PackageItem>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PackageItem {
    package_id: PackageId,
    descriptor: PackageItemKind,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PackageItemKind {
    Component(ComponentDescriptor),
    Entity(EntityDescriptor),
    Domain(RegistrationDescriptor),
    Event(EventDescriptor),
    Reaction(RegistrationDescriptor),
    Observer(RegistrationDescriptor),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ComponentDescriptor {
    id: ItemId,
    schema: Schema,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityDescriptor {
    id: ItemId,
    components: Vec<ItemId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistrationDescriptor {
    id: ItemId,
    dependencies: Vec<ItemId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventDescriptor {
    id: ItemId,
}

#[derive(Clone, Debug)]
pub struct PackageBuilder {
    id: PackageId,
    version: String,
    dependencies: Vec<PackageDependency>,
    items: Vec<PackageItemKind>,
}

impl Package {
    pub fn builder(id: PackageId) -> PackageBuilder {
        PackageBuilder {
            id,
            version: "0.1.0".to_owned(),
            dependencies: Vec::new(),
            items: Vec::new(),
        }
    }

    pub fn id(&self) -> &PackageId {
        &self.id
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn dependencies(&self) -> &[PackageDependency] {
        &self.dependencies
    }

    pub fn items(&self) -> &[PackageItem] {
        &self.items
    }
}

impl PackageBuilder {
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    pub fn dependency(mut self, dependency: PackageDependency) -> Self {
        self.dependencies.push(dependency);
        self
    }

    pub fn component(mut self, descriptor: ComponentDescriptor) -> Self {
        self.items.push(PackageItemKind::Component(descriptor));
        self
    }

    pub fn entity(mut self, descriptor: EntityDescriptor) -> Self {
        self.items.push(PackageItemKind::Entity(descriptor));
        self
    }

    pub fn domain(mut self, descriptor: impl Into<RegistrationDescriptor>) -> Self {
        self.items.push(PackageItemKind::Domain(descriptor.into()));
        self
    }

    pub fn event(mut self, descriptor: impl Into<EventDescriptor>) -> Self {
        self.items.push(PackageItemKind::Event(descriptor.into()));
        self
    }

    pub fn reaction(mut self, descriptor: impl Into<RegistrationDescriptor>) -> Self {
        self.items
            .push(PackageItemKind::Reaction(descriptor.into()));
        self
    }

    pub fn observer(mut self, descriptor: impl Into<RegistrationDescriptor>) -> Self {
        self.items
            .push(PackageItemKind::Observer(descriptor.into()));
        self
    }

    pub fn build(self) -> Package {
        let package_id = self.id;
        let items = self
            .items
            .into_iter()
            .map(|descriptor| PackageItem {
                package_id: package_id.clone(),
                descriptor,
            })
            .collect();

        Package {
            id: package_id,
            version: self.version,
            dependencies: self.dependencies,
            items,
        }
    }
}

impl PackageDependency {
    pub fn new(package_id: PackageId, version: VersionReqText) -> Self {
        Self {
            package_id,
            version,
        }
    }

    pub fn package_id(&self) -> &PackageId {
        &self.package_id
    }

    pub fn version(&self) -> &VersionReqText {
        &self.version
    }
}

impl PackageItem {
    pub fn package_id(&self) -> &PackageId {
        &self.package_id
    }

    pub fn id(&self) -> &ItemId {
        self.descriptor.id()
    }

    pub fn kind(&self) -> &PackageItemKind {
        &self.descriptor
    }

    pub fn is_component(&self) -> bool {
        matches!(self.descriptor, PackageItemKind::Component(_))
    }

    pub fn is_entity(&self) -> bool {
        matches!(self.descriptor, PackageItemKind::Entity(_))
    }

    pub fn is_domain(&self) -> bool {
        matches!(self.descriptor, PackageItemKind::Domain(_))
    }

    pub fn is_event(&self) -> bool {
        matches!(self.descriptor, PackageItemKind::Event(_))
    }

    pub fn is_reaction(&self) -> bool {
        matches!(self.descriptor, PackageItemKind::Reaction(_))
    }

    pub fn is_observer(&self) -> bool {
        matches!(self.descriptor, PackageItemKind::Observer(_))
    }

    pub fn component(&self) -> Option<&ComponentDescriptor> {
        match &self.descriptor {
            PackageItemKind::Component(descriptor) => Some(descriptor),
            _ => None,
        }
    }
}

impl PackageItemKind {
    pub fn id(&self) -> &ItemId {
        match self {
            Self::Component(descriptor) => descriptor.id(),
            Self::Entity(descriptor) => descriptor.id(),
            Self::Domain(descriptor) | Self::Reaction(descriptor) | Self::Observer(descriptor) => {
                descriptor.id()
            }
            Self::Event(descriptor) => descriptor.id(),
        }
    }
}

impl ComponentDescriptor {
    pub fn new(id: ItemId, schema: Schema) -> Self {
        Self { id, schema }
    }

    pub fn id(&self) -> &ItemId {
        &self.id
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
    }
}

impl EntityDescriptor {
    pub fn new(id: ItemId) -> Self {
        Self {
            id,
            components: Vec::new(),
        }
    }

    pub fn component(mut self, id: ItemId) -> Self {
        self.components.push(id);
        self
    }

    pub fn id(&self) -> &ItemId {
        &self.id
    }

    pub fn components(&self) -> &[ItemId] {
        &self.components
    }
}

impl RegistrationDescriptor {
    pub fn new(id: ItemId) -> Self {
        Self {
            id,
            dependencies: Vec::new(),
        }
    }

    pub fn dependency(mut self, id: ItemId) -> Self {
        self.dependencies.push(id);
        self
    }

    pub fn id(&self) -> &ItemId {
        &self.id
    }

    pub fn dependencies(&self) -> &[ItemId] {
        &self.dependencies
    }
}

impl EventDescriptor {
    pub fn new(id: ItemId) -> Self {
        Self { id }
    }

    pub fn id(&self) -> &ItemId {
        &self.id
    }
}

impl From<ItemId> for RegistrationDescriptor {
    fn from(id: ItemId) -> Self {
        Self::new(id)
    }
}

impl From<ItemId> for EventDescriptor {
    fn from(id: ItemId) -> Self {
        Self::new(id)
    }
}
