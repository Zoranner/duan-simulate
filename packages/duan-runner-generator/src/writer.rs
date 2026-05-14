use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Result, RunnerGeneratorError};
use crate::model::{CrateDependency, PackageInstall, RunnerProject};

pub fn write_runner_project(project: &RunnerProject, output_dir: impl AsRef<Path>) -> Result<()> {
    validate_project(project)?;

    let output_dir = output_dir.as_ref();
    write_file(
        output_dir.join("Cargo.toml"),
        render_cargo_toml(project).into_bytes(),
    )?;
    write_file(
        output_dir.join(".cargo").join("config.toml"),
        render_cargo_config(project).into_bytes(),
    )?;
    write_file(
        output_dir.join("src").join("main.rs"),
        render_main_rs(project).into_bytes(),
    )?;

    Ok(())
}

fn validate_project(project: &RunnerProject) -> Result<()> {
    validate_not_empty("runner crate name", &project.name)?;
    validate_not_empty("runner crate version", &project.version)?;
    validate_not_empty("registry name", &project.registry.name)?;
    validate_not_empty("registry index", &project.registry.index)?;
    validate_not_empty("scenario path", &project.scenario_path)?;
    validate_not_empty("runner path", &project.runner_path)?;

    for dependency in &project.dependencies {
        validate_not_empty("dependency crate name", &dependency.name)?;
        validate_not_empty("dependency version", &dependency.version)?;
        validate_not_empty("dependency registry", &dependency.registry)?;
    }

    for install in &project.installs {
        validate_not_empty("package install crate name", &install.crate_name)?;
    }

    Ok(())
}

fn validate_not_empty(label: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(RunnerGeneratorError::InvalidModel(format!(
            "{label} is empty"
        )));
    }

    Ok(())
}

fn render_cargo_toml(project: &RunnerProject) -> String {
    let mut output = String::new();
    output.push_str("[package]\n");
    output.push_str(&format!("name = {:?}\n", project.name));
    output.push_str(&format!("version = {:?}\n", project.version));
    output.push_str("edition = \"2021\"\n\n");
    output.push_str("[dependencies]\n");

    for dependency in sorted_dependencies(&project.dependencies) {
        output.push_str(&format!(
            "{} = {{ version = {:?}, registry = {:?} }}\n",
            dependency.name, dependency.version, dependency.registry
        ));
    }

    output
}

fn render_cargo_config(project: &RunnerProject) -> String {
    format!(
        "[registries.{}]\nindex = {:?}\n",
        project.registry.name, project.registry.index
    )
}

fn render_main_rs(project: &RunnerProject) -> String {
    let mut output = String::new();
    output.push_str("use std::path::PathBuf;\n\n");
    output.push_str("fn main() -> Result<(), Box<dyn std::error::Error>> {\n");
    output.push_str("    let scenario_path = std::env::args_os()\n");
    output.push_str("        .nth(1)\n");
    output.push_str("        .map(PathBuf::from)\n");
    output.push_str(&format!(
        "        .unwrap_or_else(|| PathBuf::from({:?}));\n\n",
        project.scenario_path
    ));
    output.push_str("    let scenario = duan_scenario::load_from_path(&scenario_path)?;\n");

    let installs = sorted_installs(&project.installs);
    if installs.is_empty() {
        output.push_str("    let registry = duan_package::Registry::new();\n\n");
    } else {
        output.push_str("    let registry = duan_package::Registry::new()\n");
        for (index, install) in installs.iter().enumerate() {
            let suffix = if index + 1 == installs.len() {
                "?;"
            } else {
                "?"
            };
            output.push_str(&format!(
                "        .install({}::package()){}\n",
                crate_module_name(&install.crate_name),
                suffix
            ));
        }
        output.push('\n');
    }

    output.push_str(&format!(
        "    run_scenario(scenario, registry, {:?})?;\n",
        project.runner_path
    ));
    output.push_str("    Ok(())\n");
    output.push_str("}\n\n");
    output.push_str("fn run_scenario(\n");
    output.push_str("    _scenario: duan_scenario::Manifest,\n");
    output.push_str("    _registry: duan_package::Registry,\n");
    output.push_str("    runner_path: &str,\n");
    output.push_str(") -> Result<(), Box<dyn std::error::Error>> {\n");
    output.push_str("    Err(format!(\n");
    output.push_str(
        "        \"runner execution is not implemented in generated runner stub `{runner_path}`\"\n",
    );
    output.push_str("    )\n");
    output.push_str("    .into())\n");
    output.push_str("}\n");
    output
}

fn sorted_dependencies(dependencies: &[CrateDependency]) -> Vec<&CrateDependency> {
    let mut sorted: Vec<_> = dependencies.iter().collect();
    sorted.sort_by(|left, right| left.name.cmp(&right.name));
    sorted
}

fn sorted_installs(installs: &[PackageInstall]) -> Vec<&PackageInstall> {
    let mut sorted: Vec<_> = installs.iter().collect();
    sorted.sort_by(|left, right| left.crate_name.cmp(&right.crate_name));
    sorted
}

fn crate_module_name(crate_name: &str) -> String {
    crate_name.replace('-', "_")
}

fn write_file(path: PathBuf, content: Vec<u8>) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| RunnerGeneratorError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    fs::write(&path, content).map_err(|source| RunnerGeneratorError::Io { path, source })
}
