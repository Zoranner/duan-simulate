use std::io;
use std::path::PathBuf;
use std::process::ExitStatus;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, CliError>;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("failed to load scenario `{}`: {source}", path.display())]
    LoadScenario {
        path: PathBuf,
        source: duan_scenario::ManifestError,
    },

    #[error(
        "failed to generate runner for scenario `{}` into `{}`: {source}",
        scenario.display(),
        output_dir.display()
    )]
    GenerateRunner {
        scenario: PathBuf,
        output_dir: PathBuf,
        source: duan_runner_generator::error::RunnerGeneratorError,
    },

    #[error("failed to inspect package path `{}`: {source}", path.display())]
    ReadPackage {
        path: PathBuf,
        source: duan_package::PackageError,
    },

    #[error(
        "runner directory `{}` does not contain runner manifest `{}`",
        runner_dir.display(),
        manifest_path.display()
    )]
    MissingRunnerManifest {
        runner_dir: PathBuf,
        manifest_path: PathBuf,
    },

    #[error(
        "failed to start cargo build for runner directory `{}`: {source}",
        runner_dir.display()
    )]
    BuildRunnerStart {
        runner_dir: PathBuf,
        source: io::Error,
    },

    #[error(
        "cargo build failed for runner directory `{}` with status {status}",
        runner_dir.display()
    )]
    BuildRunnerFailed {
        runner_dir: PathBuf,
        status: ExitStatus,
    },

    #[error(
        "invalid delivery path for scenario `{}`, runner `{}`, out dir `{}`: {detail}",
        scenario.display(),
        runner.display(),
        out_dir.display()
    )]
    InvalidDeliveryPath {
        scenario: PathBuf,
        runner: PathBuf,
        out_dir: PathBuf,
        detail: &'static str,
    },

    #[error(
        "failed to deliver scenario `{}` with runner `{}` into out dir `{}`: {source}",
        scenario.display(),
        runner.display(),
        out_dir.display()
    )]
    DeliverRunner {
        scenario: PathBuf,
        runner: PathBuf,
        out_dir: PathBuf,
        source: io::Error,
    },

    #[error("not implemented: {command}")]
    NotImplemented { command: &'static str },
}

impl CliError {
    pub fn not_implemented(command: &'static str) -> Self {
        Self::NotImplemented { command }
    }
}
