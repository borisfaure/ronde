use crate::runner::{CommandError, CommandOutput, CommandResult};
use chrono::{DateTime, Datelike, Timelike, Utc};
use serde_derive::{Deserialize, Serialize};
use thiserror::Error;
use tokio::fs;

/// History Result
#[derive(Error, Debug, PartialEq, Serialize, Deserialize)]
pub enum HistoryError {
    /// Timeout
    Timeout,
    /// Command error
    CommandError { stdout: String, stderr: String },
    /// Other error
    Other { message: String },
}
impl std::fmt::Display for HistoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HistoryError::Timeout => write!(f, "Timeout"),
            HistoryError::CommandError { stdout, stderr } => {
                write!(f, "Command error: stdout: {}, stderr: {}", stdout, stderr)
            }
            HistoryError::Other { message } => write!(f, "Other error: {}", message),
        }
    }
}

/// How a command result is aggregated
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum TimeTag {
    /// Aggregated over a day
    Day(u8), // 0-6
    /// Aggregated over an hour
    Hour(u8), // 0-23
    /// Single entry
    Minute(u8), // 0-59
}

/// History entry for a single command
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct CommandHistoryEntry {
    /// Result of the command
    #[serde(with = "serde_yaml::with::singleton_map_recursive")]
    pub result: Result<Option<CommandOutput>, HistoryError>,
    /// Timestamp when the command was run
    pub timestamp: DateTime<Utc>,
    /// Tag for the time aggregation
    pub tag: TimeTag,
}

/// History of a single command
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct CommandHistory {
    /// Name of the command
    pub name: String,
    /// Entries for the command
    pub entries: Vec<CommandHistoryEntry>,
}

impl CommandHistory {
    /// Get latest timestamp
    fn latest_timestamp(&self) -> Option<DateTime<Utc>> {
        self.entries.last().map(|e| e.timestamp)
    }

    /// Recreate tags based on the timestamps
    pub fn recreate_tags(&mut self) {
        if let Some(latest_timestamp) = self.latest_timestamp() {
            for entry in self.entries.iter_mut() {
                let delta = latest_timestamp.signed_duration_since(entry.timestamp);
                if delta.num_hours() < 1 {
                    let min: u8 = (entry.timestamp.time().minute() / 5 * 5)
                        .try_into()
                        .unwrap_or(0);
                    entry.tag = TimeTag::Minute(min);
                } else if delta.num_hours() < 25 {
                    let hour: u8 = entry.timestamp.time().hour().try_into().unwrap_or(0);
                    entry.tag = TimeTag::Hour(hour);
                } else {
                    let day: u8 = entry
                        .timestamp
                        .naive_utc()
                        .weekday()
                        .num_days_from_monday()
                        .try_into()
                        .unwrap_or(0);
                    entry.tag = TimeTag::Day(day);
                }
            }
        }
    }

    /// Rotate the history to keep only the last n entries:
    /// - 1 per day for 7 days,
    /// - 1 per hour for 24 hours,
    /// - 1 per 5 minutes for 60 minutes.
    /// This is a simple way to keep a history of the last week at a
    /// reasonable size..
    /// It's not perfect and naive, but it's good enough for a start.
    pub fn rotate(&mut self) {
        self.entries.dedup_by(|_left, _right| false);
    }
}

/// History of commands
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct History {
    /// Vector of each command's history
    pub commands: Vec<CommandHistory>,
}

impl History {
    /// Recreate tags based on the timestamps
    pub fn recreate_tags(&mut self) {
        for command in self.commands.iter_mut() {
            command.recreate_tags();
        }
    }
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
    pub fn update(&mut self, results: Vec<CommandResult>) {
        for result in results {
            let command_history = self
                .commands
                .iter_mut()
                .find(|c| c.name == result.config.name);
            let entry = CommandHistoryEntry {
                result: match result.result {
                    Ok(output) => Ok(Some(output)),
                    Err(CommandError::ReturnedError(e)) => Err(HistoryError::CommandError {
                        stdout: String::from_utf8_lossy(&e.output.stdout).to_string(),
                        stderr: String::from_utf8_lossy(&e.output.stderr).to_string(),
                    }),
                    Err(CommandError::TimedOut(_)) => Err(HistoryError::Timeout),
                    Err(e) => Err(HistoryError::Other {
                        message: e.to_string(),
                    }),
                },
                timestamp: chrono::Utc::now(),
                tag: TimeTag::Minute(0),
            };
            match command_history {
                Some(command_history) => {
                    command_history.entries.push(entry);
                }
                None => {
                    let command_history = CommandHistory {
                        name: result.config.name.clone(),
                        entries: vec![entry],
                    };
                    self.commands.push(command_history);
                }
            }
        }
    }
}

/* tests */
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::CommandConfig;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_load_save_history() {
        let history_file = NamedTempFile::new().unwrap();
        let history_file_path = history_file.path().to_str().unwrap().to_string();
        let history = History {
            commands: vec![CommandHistory {
                name: "test".to_string(),
                entries: vec![CommandHistoryEntry {
                    result: Ok(Some(CommandOutput {
                        status: 0,
                        stdout: "stdout".to_string(),
                        stderr: "stderr".to_string(),
                    })),
                    timestamp: chrono::Utc::now(),
                    tag: TimeTag::Minute(0),
                }],
            }],
        };

        history.save(&history_file_path).await.unwrap();

        let loaded_history = History::load(&history_file_path).await.unwrap();
        assert_eq!(history, loaded_history);
    }

    #[test]
    fn test_purge_from_results() {
        let mut history = History {
            commands: vec![
                CommandHistory {
                    name: "test".to_string(),
                    entries: vec![],
                },
                CommandHistory {
                    name: "test2".to_string(),
                    entries: vec![],
                },
                CommandHistory {
                    name: "test3".to_string(),
                    entries: vec![],
                },
                CommandHistory {
                    name: "test4".to_string(),
                    entries: vec![],
                },
            ],
        };
        history.purge_from_results(&vec![
            CommandResult {
                config: CommandConfig {
                    name: "test2".to_string(),
                    timeout: 0,
                    run: "test2".to_string(),
                },
                result: Ok(CommandOutput {
                    status: 0,
                    stdout: "".to_string(),
                    stderr: "".to_string(),
                }),
            },
            CommandResult {
                config: CommandConfig {
                    name: "test3".to_string(),
                    timeout: 0,
                    run: "test3".to_string(),
                },
                result: Ok(CommandOutput {
                    status: 0,
                    stdout: "".to_string(),
                    stderr: "".to_string(),
                }),
            },
        ]);
        assert_eq!(
            history,
            History {
                commands: vec![
                    CommandHistory {
                        name: "test2".to_string(),
                        entries: vec![],
                    },
                    CommandHistory {
                        name: "test3".to_string(),
                        entries: vec![],
                    },
                ]
            }
        );
    }
}
