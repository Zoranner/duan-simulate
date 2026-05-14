use std::collections::{BTreeMap, BTreeSet};

use crate::{ItemId, Package, PackageError, PackageId, PackageItem, PackageResult};

#[derive(Clone, Debug, Default)]
pub struct Registry {
    packages: BTreeMap<PackageId, Package>,
    items: BTreeMap<ItemId, PackageItem>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn install(mut self, package: Package) -> PackageResult<Self> {
        self.install_package(package)?;
        Ok(self)
    }

    pub fn install_package(&mut self, package: Package) -> PackageResult<()> {
        let package_id = package.id().clone();
        if self.packages.contains_key(&package_id) {
            return Err(PackageError::DuplicatePackage(package_id));
        }

        let mut package_item_ids = BTreeSet::new();
        for item in package.items() {
            if !package_item_ids.insert(item.id().clone()) {
                return Err(PackageError::DuplicateItem(item.id().clone()));
            }
            if self.items.contains_key(item.id()) {
                return Err(PackageError::DuplicateItem(item.id().clone()));
            }
        }

        for item in package.items() {
            self.items.insert(item.id().clone(), item.clone());
        }
        self.packages.insert(package_id, package);

        Ok(())
    }

    pub fn package(&self, id: &PackageId) -> Option<&Package> {
        self.packages.get(id)
    }

    pub fn item(&self, id: &ItemId) -> Option<&PackageItem> {
        self.items.get(id)
    }

    pub fn packages(&self) -> impl Iterator<Item = &Package> {
        self.packages.values()
    }

    pub fn items(&self) -> impl Iterator<Item = &PackageItem> {
        self.items.values()
    }
}
