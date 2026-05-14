use duan_scenario::Manifest;

use crate::{ItemId, PackageError, PackageItem, PackageItemKind, PackageResult, Registry};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssemblyPlan {
    steps: Vec<AssemblyStep>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssemblyStep {
    InstallDomain {
        item_id: ItemId,
    },
    InstallReaction {
        item_id: ItemId,
    },
    BuildWorld,
    SpawnEntity {
        entity_id: String,
        item_id: ItemId,
    },
    ApplyComponentOverride {
        entity_id: String,
        component_id: ItemId,
    },
}

impl AssemblyPlan {
    pub fn from_manifest(manifest: &Manifest, registry: &Registry) -> PackageResult<Self> {
        let mut steps = Vec::new();

        for (index, domain) in manifest.domains.iter().enumerate() {
            let item_id = expect_item(
                registry,
                &domain.type_id,
                format!("domains[{index}].type"),
                PackageItem::is_domain,
                "domain",
            )?;
            steps.push(AssemblyStep::InstallDomain { item_id });
        }

        for (index, reaction) in manifest.reactions.iter().enumerate() {
            let item_id = expect_item(
                registry,
                &reaction.type_id,
                format!("reactions[{index}].type"),
                PackageItem::is_reaction,
                "reaction",
            )?;
            steps.push(AssemblyStep::InstallReaction { item_id });
        }

        steps.push(AssemblyStep::BuildWorld);

        for (entity_index, entity) in manifest.entities.iter().enumerate() {
            let entity_type = expect_item(
                registry,
                &entity.type_id,
                format!("entities[{entity_index}].type"),
                PackageItem::is_entity,
                "entity",
            )?;
            let entity_item = registry
                .item(&entity_type)
                .expect("entity item was checked");
            let PackageItemKind::Entity(descriptor) = entity_item.kind() else {
                unreachable!("entity item kind was checked")
            };

            for component_id in entity.components.keys() {
                let component_id = ItemId::new(component_id.clone())?;
                if registry.item(&component_id).is_none() {
                    return Err(PackageError::MissingItem {
                        location: format!("entities[{entity_index}].components"),
                        item_id: component_id,
                    });
                }
                if !descriptor.components().contains(&component_id) {
                    return Err(PackageError::UnsupportedComponentOverride {
                        entity_id: entity.id.clone(),
                        entity_type: entity_type.clone(),
                        component_id,
                    });
                }
            }

            steps.push(AssemblyStep::SpawnEntity {
                entity_id: entity.id.clone(),
                item_id: entity_type,
            });

            for component_id in entity.components.keys() {
                steps.push(AssemblyStep::ApplyComponentOverride {
                    entity_id: entity.id.clone(),
                    component_id: ItemId::new(component_id.clone())?,
                });
            }
        }

        Ok(Self { steps })
    }

    pub fn steps(&self) -> &[AssemblyStep] {
        &self.steps
    }
}

fn expect_item(
    registry: &Registry,
    raw_id: &str,
    location: String,
    predicate: fn(&PackageItem) -> bool,
    expected_kind: &'static str,
) -> PackageResult<ItemId> {
    let item_id = ItemId::new(raw_id.to_owned())?;
    let Some(item) = registry.item(&item_id) else {
        return Err(PackageError::MissingItem { location, item_id });
    };
    if !predicate(item) {
        return Err(PackageError::UnexpectedItemKind {
            location,
            item_id,
            expected_kind,
        });
    }

    Ok(item_id)
}
