use std::fmt;
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, RunnerGeneratorError>;

#[derive(Debug)]
pub enum RunnerGeneratorError {
    InvalidModel(String),
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
}

impl fmt::Display for RunnerGeneratorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidModel(message) => write!(f, "invalid runner project model: {message}"),
            Self::Io { path, source } => {
                write!(
                    f,
                    "failed to write generated runner file {}: {source}",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for RunnerGeneratorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidModel(_) => None,
            Self::Io { source, .. } => Some(source),
        }
    }
}
