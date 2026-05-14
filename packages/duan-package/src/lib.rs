pub mod assembly;
pub mod error;
pub mod factory;
pub mod id;
pub mod metadata;
pub mod package;
pub mod registry;
pub mod schema;

pub use assembly::{AssemblyPlan, AssemblyStep};
pub use error::{PackageError, PackageResult};
pub use factory::{
    DomainFactory, EntityFactory, ObserverFactory, ReactionFactory, ScenarioComponent,
};
pub use id::{ItemId, PackageId, VersionReqText};
pub use metadata::{PackageMetadata, ProvidesMetadata, PACKAGE_METADATA_FILE};
pub use package::{
    ComponentDescriptor, EntityDescriptor, EventDescriptor, Package, PackageBuilder,
    PackageDependency, PackageItem, PackageItemKind, RegistrationDescriptor,
};
pub use registry::Registry;
pub use schema::{EditorMetadata, FieldSchema, PrimitiveKind, PrimitiveValue, Range, Schema, Unit};
