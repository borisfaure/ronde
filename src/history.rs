use crate::runner::CommandResult;
use serde_derive::{Deserialize, Serialize};
use thiserror::Error;
use tokio::fs;

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
pub enum HistoryIoError {
    /// IO Error
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    /// Serde Error
    #[error("Serde Error: {0}")]
    SerdeError(#[from] serde_yaml::Error),
}

impl History {
    /// Load history from a file
    pub async fn load(history_file: &String) -> Result<Self, HistoryIoError> {
        match fs::read_to_string(history_file).await {
            Ok(contents) => {
                let history: History = serde_yaml::from_str(&contents)?;
                Ok(history)
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => Ok(History {
                commands: Vec::new(),
            }),
            Err(e) => Err(HistoryIoError::IoError(e)),
        }
    }

    /// Save history to a file
    pub async fn save(&self, history_file: &String) -> Result<(), HistoryIoError> {
        let bytes = serde_yaml::to_string(self)?;
        fs::write(history_file, &bytes).await?;
        Ok(())
    }

    /// Purge the history of commands that are not in the current configuration
    pub fn purge_from_results(&mut self, results: &[CommandResult]) {
        self.commands
            .retain(|c| results.iter().any(|r| r.config.name == c.name));
    }

    /// Update the history with new results
    pub fn update(&mut self, results: &Vec<CommandResult>) {
        for result in results {
            let command_history = self
                .commands
                .iter_mut()
                .find(|c| c.name == result.config.name);
            match command_history {
                Some(command_history) => {
                    let entry = CommandHistoryEntry {
                        result: match &result.result {
                            Ok(_) => Ok(()),
                            Err(e) => Err(HistoryError::Other(e.to_string())),
                        },
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    };
                    command_history.entries.push(entry);
                }
                None => {
                    let mut entries = Vec::new();
                    let entry = CommandHistoryEntry {
                        result: match &result.result {
                            Ok(_) => Ok(()),
                            Err(e) => Err(HistoryError::Other(e.to_string())),
                        },
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    };
                    entries.push(entry);
                    let command_history = CommandHistory {
                        name: result.config.name.clone(),
                        entries,
                    };
                    self.commands.push(command_history);
                }
            }
        }
    }
}
