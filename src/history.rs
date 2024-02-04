use serde_derive::{Deserialize, Serialize};
use thiserror::Error;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

/// History Result
#[derive(Error, Debug, Serialize, Deserialize)]
pub enum HistoryError {
    /// Timeout
    Timeout,
    /// Command error
    CommandError { stdout: String, stderr: String },
    /// Other error
    Other(String),
}
impl std::fmt::Display for HistoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HistoryError::Timeout => write!(f, "Timeout"),
            HistoryError::CommandError { stdout, stderr } => {
                write!(f, "Command error: stdout: {}, stderr: {}", stdout, stderr)
            }
            HistoryError::Other(err) => write!(f, "Other error: {}", err),
        }
    }
}

/// History entry for a single command
#[derive(Debug, Deserialize, Serialize)]
pub struct CommandHistoryEntry {
    /// Result of the command
    pub result: Result<(), HistoryError>,
    /// Timestamp when the command was run
    pub timestamp: String,
}

/// History of a single command
#[derive(Debug, Deserialize, Serialize)]
pub struct CommandHistory {
    /// Name of the command
    pub name: String,
    /// Entries for the command
    pub entries: Vec<CommandHistoryEntry>,
}

/// History of commands
#[derive(Debug, Deserialize, Serialize)]
pub struct History {
    /// Vector of each command's history
    pub commands: Vec<CommandHistory>,
}

#[derive(Error, Debug)]
pub enum LoadError {
    /// IO Error
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    /// Serde Error
    #[error("Serde Error: {0}")]
    SerdeError(#[from] postcard::Error),
}

/// Load history from a file
pub async fn load(postcard_file: &String) -> Result<History, LoadError> {
    let mut file = File::open(postcard_file).await?;
    let mut contents = vec![];
    file.read_to_end(&mut contents).await?;
    let history: History = postcard::from_bytes(&contents)?;
    Ok(history)
}
