use std::collections::HashSet;
use std::path::{Component, Path};

use crate::{Manifest, ManifestError, OutputSpec, ValidationError};

pub fn validate_manifest(manifest: &Manifest) -> Result<(), ManifestError> {
    let mut errors = Vec::new();
    let package_ids = validate_packages(manifest, &mut errors);

    if !is_valid_package_id(&manifest.scenario.package) {
        errors.push(ValidationError::InvalidPackageId {
            location: "scenario.package".to_string(),
            id: manifest.scenario.package.clone(),
        });
    }

    for (index, domain) in manifest.domains.iter().enumerate() {
        validate_item_ref(
            &format!("domains[{index}].type"),
            &domain.type_id,
            &package_ids,
            false,
            &mut errors,
        );
    }

    for (index, reaction) in manifest.reactions.iter().enumerate() {
        validate_item_ref(
            &format!("reactions[{index}].type"),
            &reaction.type_id,
            &package_ids,
            false,
            &mut errors,
        );
    }

    validate_entities(manifest, &package_ids, &mut errors);
    validate_run_and_output_paths(manifest, &mut errors);

    if errors.is_empty() {
        Ok(())
    } else {
        Err(ManifestError::Validation(errors))
    }
}

fn validate_packages(manifest: &Manifest, errors: &mut Vec<ValidationError>) -> HashSet<String> {
    let mut package_ids = HashSet::new();

    for (index, package) in manifest.packages.iter().enumerate() {
        if !is_valid_package_id(&package.id) {
            errors.push(ValidationError::InvalidPackageId {
                location: format!("packages[{index}].id"),
                id: package.id.clone(),
            });
        }
        package_ids.insert(package.id.clone());
    }

    package_ids
}

fn validate_entities(
    manifest: &Manifest,
    package_ids: &HashSet<String>,
    errors: &mut Vec<ValidationError>,
) {
    let mut entity_ids = HashSet::new();

    for (entity_index, entity) in manifest.entities.iter().enumerate() {
        if !entity_ids.insert(entity.id.clone()) {
            errors.push(ValidationError::DuplicateEntityId {
                id: entity.id.clone(),
            });
        }

        validate_item_ref(
            &format!("entities[{entity_index}].type"),
            &entity.type_id,
            package_ids,
            false,
            errors,
        );

        for component_id in entity.components.keys() {
            validate_item_ref(
                &format!("entities[{entity_index}].components"),
                component_id,
                package_ids,
                true,
                errors,
            );
        }
    }

    for (experiment_index, experiment) in manifest.experiments.iter().enumerate() {
        for (entity_id, components) in &experiment.overrides.components {
            if !entity_ids.contains(entity_id) {
                errors.push(ValidationError::InvalidItemId {
                    location: format!("experiments[{experiment_index}].overrides.components"),
                    id: entity_id.clone(),
                });
            }

            for component_id in components.keys() {
                validate_item_ref(
                    &format!("experiments[{experiment_index}].overrides.components[{entity_id}]"),
                    component_id,
                    package_ids,
                    true,
                    errors,
                );
            }
        }
    }
}

fn validate_run_and_output_paths(manifest: &Manifest, errors: &mut Vec<ValidationError>) {
    if let Some(run) = &manifest.run {
        if let Some(output_dir) = &run.output_dir {
            validate_path("run.output_dir", output_dir, errors);
        }
    }

    if let Some(outputs) = &manifest.outputs {
        validate_outputs("outputs", outputs, errors);
    }

    for (index, experiment) in manifest.experiments.iter().enumerate() {
        if let Some(run) = &experiment.run {
            if let Some(output_dir) = &run.output_dir {
                validate_path(
                    &format!("experiments[{index}].run.output_dir"),
                    output_dir,
                    errors,
                );
            }
        }
    }
}

fn validate_outputs(location: &str, outputs: &OutputSpec, errors: &mut Vec<ValidationError>) {
    for (name, path) in outputs.paths() {
        validate_path(&format!("{location}.{name}"), path, errors);
    }
}

fn validate_item_ref(
    location: &str,
    id: &str,
    package_ids: &HashSet<String>,
    component_ref: bool,
    errors: &mut Vec<ValidationError>,
) {
    if !is_valid_item_id(id) {
        let error = if component_ref {
            ValidationError::InvalidComponentRef {
                location: location.to_string(),
                id: id.to_string(),
            }
        } else {
            ValidationError::InvalidItemId {
                location: location.to_string(),
                id: id.to_string(),
            }
        };
        errors.push(error);
        return;
    }

    if !has_package_dependency(id, package_ids) {
        errors.push(ValidationError::MissingPackageDependency {
            location: location.to_string(),
            item: id.to_string(),
        });
    }
}

fn validate_path(location: &str, path: &str, errors: &mut Vec<ValidationError>) {
    if !is_valid_relative_output_path(path) {
        errors.push(ValidationError::InvalidPath {
            location: location.to_string(),
            path: path.to_string(),
        });
    }
}

fn has_package_dependency(item_id: &str, package_ids: &HashSet<String>) -> bool {
    package_ids.iter().any(|package_id| {
        item_id
            .strip_prefix(package_id)
            .is_some_and(|tail| tail.starts_with('/'))
    })
}

fn is_valid_package_id(id: &str) -> bool {
    is_valid_name_segment(id)
}

fn is_valid_item_id(id: &str) -> bool {
    let Some((package, item)) = id.split_once('/') else {
        return false;
    };

    is_valid_package_id(package) && is_valid_name_segment(item)
}

fn is_valid_name_segment(segment: &str) -> bool {
    if segment.is_empty()
        || segment.starts_with('-')
        || segment.ends_with('-')
        || segment.contains("--")
    {
        return false;
    }

    segment
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn is_valid_relative_output_path(path: &str) -> bool {
    if path.trim().is_empty() {
        return false;
    }

    let path = Path::new(path);
    if path.is_absolute() {
        return false;
    }

    path.components()
        .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_package_dependency_by_longest_available_prefix_shape() {
        let package_ids = HashSet::from(["examples-motion".to_string()]);

        assert!(has_package_dependency(
            "examples-motion/position-2",
            &package_ids
        ));
        assert!(!has_package_dependency(
            "duan-other/position-2",
            &package_ids
        ));
    }
}
