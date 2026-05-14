use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::{Args, Parser, Subcommand};
use duan_package::PackageMetadata;
use duan_runner_generator::model::{
    CrateDependency, PackageInstall, RegistryConfig, RunnerProject,
};
use duan_runner_generator::writer::write_runner_project;
use duan_scenario::Manifest;

use crate::error::{CliError, Result};

#[derive(Debug, Parser)]
#[command(name = "duan", version, about = "DUAN platform command line")]
pub struct Cli {
    #[arg(long)]
    version_info: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Scenario {
        #[command(subcommand)]
        command: ScenarioCommand,
    },
    Runner {
        #[command(subcommand)]
        command: RunnerCommand,
    },
    Package {
        #[command(subcommand)]
        command: PackageCommand,
    },
    Run,
    Deliver(DeliverArgs),
}

#[derive(Debug, Subcommand)]
enum ScenarioCommand {
    Validate(ScenarioPath),
}

#[derive(Debug, Subcommand)]
enum RunnerCommand {
    Generate(RunnerGenerateArgs),
    Build(RunnerBuildArgs),
}

#[derive(Debug, Subcommand)]
enum PackageCommand {
    Inspect(PackageInspectArgs),
}

#[derive(Debug, Args)]
struct ScenarioPath {
    scenario: PathBuf,
}

#[derive(Debug, Args)]
struct RunnerGenerateArgs {
    scenario: PathBuf,

    #[arg(long)]
    out: PathBuf,

    #[arg(long)]
    registry_name: String,

    #[arg(long)]
    registry_index: String,
}

#[derive(Debug, Args)]
struct RunnerBuildArgs {
    runner_dir: PathBuf,

    #[arg(long)]
    release: bool,
}

#[derive(Debug, Args)]
struct DeliverArgs {
    scenario: PathBuf,

    #[arg(long)]
    runner: PathBuf,

    #[arg(long)]
    out: PathBuf,
}

#[derive(Debug, Args)]
struct PackageInspectArgs {
    path: PathBuf,
}

pub fn run(cli: Cli) -> Result<()> {
    if cli.version_info {
        println!("duan-cli {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    match cli.command {
        Some(Commands::Scenario { command }) => run_scenario(command),
        Some(Commands::Runner { command }) => run_runner(command),
        Some(Commands::Package { command }) => run_package(command),
        Some(Commands::Run) => Err(CliError::not_implemented(
            "run runner execution is not implemented yet; use runner build and deliver",
        )),
        Some(Commands::Deliver(args)) => run_deliver(args),
        None => Ok(()),
    }
}

fn run_scenario(command: ScenarioCommand) -> Result<()> {
    match command {
        ScenarioCommand::Validate(args) => {
            load_scenario(&args.scenario)?;
            println!("valid: {}", args.scenario.display());
            Ok(())
        }
    }
}

fn run_runner(command: RunnerCommand) -> Result<()> {
    match command {
        RunnerCommand::Generate(args) => {
            let manifest = load_scenario(&args.scenario)?;
            let project = runner_project_from_manifest(
                &manifest,
                &args.scenario,
                args.registry_name,
                args.registry_index,
            );
            write_runner_project(&project, &args.out).map_err(|source| {
                CliError::GenerateRunner {
                    scenario: args.scenario,
                    output_dir: args.out.clone(),
                    source,
                }
            })?;
            println!("generated: {}", args.out.display());
            Ok(())
        }
        RunnerCommand::Build(args) => build_runner(&args.runner_dir, args.release),
    }
}

fn build_runner(runner_dir: &Path, release: bool) -> Result<()> {
    let manifest_path = runner_dir.join("Cargo.toml");
    if !manifest_path.is_file() {
        return Err(CliError::MissingRunnerManifest {
            runner_dir: runner_dir.to_path_buf(),
            manifest_path,
        });
    }

    let args = runner_build_command_args(runner_dir, release);
    let status = Command::new("cargo")
        .args(&args)
        .status()
        .map_err(|source| CliError::BuildRunnerStart {
            runner_dir: runner_dir.to_path_buf(),
            source,
        })?;

    if !status.success() {
        return Err(CliError::BuildRunnerFailed {
            runner_dir: runner_dir.to_path_buf(),
            status,
        });
    }

    println!("built: {}", runner_dir.display());
    Ok(())
}

fn runner_build_command_args(runner_dir: &Path, release: bool) -> Vec<String> {
    let mut args = vec![
        "build".to_string(),
        "--manifest-path".to_string(),
        runner_dir.join("Cargo.toml").display().to_string(),
    ];
    if release {
        args.push("--release".to_string());
    }
    args
}

fn run_deliver(args: DeliverArgs) -> Result<()> {
    deliver_runner(&args.scenario, &args.runner, &args.out)?;
    println!("delivered: {}", args.out.display());
    Ok(())
}

fn deliver_runner(scenario: &Path, runner: &Path, out_dir: &Path) -> Result<()> {
    let scenario_name = scenario
        .file_name()
        .ok_or_else(|| CliError::InvalidDeliveryPath {
            scenario: scenario.to_path_buf(),
            runner: runner.to_path_buf(),
            out_dir: out_dir.to_path_buf(),
            detail: "scenario path has no file name",
        })?;
    let runner_name = runner
        .file_name()
        .ok_or_else(|| CliError::InvalidDeliveryPath {
            scenario: scenario.to_path_buf(),
            runner: runner.to_path_buf(),
            out_dir: out_dir.to_path_buf(),
            detail: "runner path has no file name",
        })?;
    let scenario_dir = out_dir.join("scenario");
    let runs_dir = out_dir.join("runs");

    fs::create_dir_all(&scenario_dir).map_err(|source| CliError::DeliverRunner {
        scenario: scenario.to_path_buf(),
        runner: runner.to_path_buf(),
        out_dir: out_dir.to_path_buf(),
        source,
    })?;
    fs::create_dir_all(&runs_dir).map_err(|source| CliError::DeliverRunner {
        scenario: scenario.to_path_buf(),
        runner: runner.to_path_buf(),
        out_dir: out_dir.to_path_buf(),
        source,
    })?;
    fs::copy(runner, out_dir.join(runner_name)).map_err(|source| CliError::DeliverRunner {
        scenario: scenario.to_path_buf(),
        runner: runner.to_path_buf(),
        out_dir: out_dir.to_path_buf(),
        source,
    })?;
    fs::copy(scenario, scenario_dir.join(scenario_name)).map_err(|source| {
        CliError::DeliverRunner {
            scenario: scenario.to_path_buf(),
            runner: runner.to_path_buf(),
            out_dir: out_dir.to_path_buf(),
            source,
        }
    })?;
    fs::write(
        out_dir.join("README.md"),
        delivery_readme(
            runner_name.to_string_lossy(),
            scenario_name.to_string_lossy(),
        ),
    )
    .map_err(|source| CliError::DeliverRunner {
        scenario: scenario.to_path_buf(),
        runner: runner.to_path_buf(),
        out_dir: out_dir.to_path_buf(),
        source,
    })?;

    Ok(())
}

fn delivery_readme(runner_name: impl AsRef<str>, scenario_name: impl AsRef<str>) -> String {
    format!(
        "# DUAN 交付包\n\n\
         - Runner: `{}`\n\
         - Scenario: `scenario/{}`\n\
         - Runs: `runs/`\n\n\
         客户可以修改 scenario 下的场景文件来调整参数和输入。\n\
         不可修改 Rust 逻辑，不可修改 package set；如需变更逻辑或包集合，请重新生成并构建 runner。\n",
        runner_name.as_ref(),
        scenario_name.as_ref()
    )
}

fn run_package(command: PackageCommand) -> Result<()> {
    match command {
        PackageCommand::Inspect(args) => {
            let (metadata, base_path) =
                PackageMetadata::load_with_base_path(&args.path).map_err(|source| {
                    CliError::ReadPackage {
                        path: args.path.clone(),
                        source: Box::new(source),
                    }
                })?;
            metadata
                .validate_schema_files(&base_path)
                .map_err(|source| CliError::ReadPackage {
                    path: args.path.clone(),
                    source: Box::new(source),
                })?;
            println!("package: {}", metadata.package_id());
            println!("version: {}", metadata.package_version());
            println!("crate: {}", metadata.rust_crate());
            println!("provides: {}", metadata.provides_count());
            Ok(())
        }
    }
}

fn load_scenario(path: &Path) -> Result<Manifest> {
    duan_scenario::load_from_path(path).map_err(|source| CliError::LoadScenario {
        path: path.to_path_buf(),
        source,
    })
}

fn runner_project_from_manifest(
    manifest: &Manifest,
    scenario_path: &Path,
    registry_name: String,
    registry_index: String,
) -> RunnerProject {
    let mut dependencies = vec![
        CrateDependency::registry("duan", env!("CARGO_PKG_VERSION"), &registry_name),
        CrateDependency::registry("duan-package", env!("CARGO_PKG_VERSION"), &registry_name),
        CrateDependency::registry("duan-scenario", env!("CARGO_PKG_VERSION"), &registry_name),
    ];
    let mut installs = Vec::new();

    for package in &manifest.packages {
        let crate_name = crate_name_from_package_id(&package.id);
        dependencies.push(CrateDependency::registry(
            crate_name.clone(),
            package.version.clone(),
            &registry_name,
        ));
        installs.push(PackageInstall::new(crate_name));
    }

    RunnerProject {
        name: runner_name_from_scenario(&manifest.scenario.id),
        version: env!("CARGO_PKG_VERSION").to_string(),
        registry: RegistryConfig {
            name: registry_name,
            index: registry_index,
        },
        scenario_path: scenario_path.display().to_string(),
        dependencies,
        installs,
    }
}

fn runner_name_from_scenario(id: &str) -> String {
    format!("{}-runner", to_kebab(id))
}

fn crate_name_from_package_id(id: &str) -> String {
    to_kebab(id)
}

fn to_kebab(value: &str) -> String {
    let mut output = String::new();
    let mut last_was_separator = false;

    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            output.push(ch.to_ascii_lowercase());
            last_was_separator = false;
        } else if !last_was_separator && !output.is_empty() {
            output.push('-');
            last_was_separator = true;
        }
    }

    output.trim_end_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::runner_build_command_args;
    use std::path::PathBuf;

    #[test]
    fn runner_build_command_args_include_manifest_and_release_flag() {
        let runner_dir = PathBuf::from("runner-project");

        assert_eq!(
            runner_build_command_args(&runner_dir, false),
            vec![
                "build".to_string(),
                "--manifest-path".to_string(),
                runner_dir.join("Cargo.toml").display().to_string(),
            ]
        );
        assert_eq!(
            runner_build_command_args(&runner_dir, true),
            vec![
                "build".to_string(),
                "--manifest-path".to_string(),
                runner_dir.join("Cargo.toml").display().to_string(),
                "--release".to_string(),
            ]
        );
    }
}
