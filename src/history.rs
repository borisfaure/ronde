use crate::runner::{CommandError, CommandOutput, CommandResult};
use chrono::{DateTime, Datelike, Timelike, Utc};
use serde_derive::{Deserialize, Serialize};
use thiserror::Error;
use tokio::fs;

/// History Result
#[derive(Error, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HistoryError {
    /// Timeout
    Timeout,
    /// Command error
    CommandError {
        exit: i32,
        stdout: String,
        stderr: String,
    },
    /// Other error
    Other { message: String },
}
impl std::fmt::Display for HistoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HistoryError::Timeout => write!(f, "Timeout"),
            HistoryError::CommandError {
                exit,
                stdout,
                stderr,
            } => {
                write!(
                    f,
                    "Command error: exit: {}, stdout: {}, stderr: {}",
                    exit, stdout, stderr
                )
            }
            HistoryError::Other { message } => write!(f, "Other error: {}", message),
        }
    }
}

/// How a command result is aggregated
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum TimeTag {
    /// Aggregated over a day
    Day(u8), // 0-6
    /// Aggregated over an hour
    Hour(u8), // 0-23
    /// Single entry
    Minute(u8), // 0-59
}

/// History entry for a single command
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CommandHistoryEntry {
    /// Result of the command
    #[serde(with = "serde_yaml::with::singleton_map_recursive")]
    pub result: Result<Option<CommandOutput>, HistoryError>,
    /// Timestamp when the command was run
    pub timestamp: DateTime<Utc>,
    /// Tag for the time aggregation
    pub tag: TimeTag,
}
impl CommandHistoryEntry {
    /// Merge in an newer entry
    fn merge_in(&mut self, newer: &mut Self) {
        // if the newer entry is an error, use it
        if let Err(e) = &newer.result {
            self.result = Err((*e).clone());
            self.timestamp = newer.timestamp;
        } else if self.result.is_err() {
            // do nothing if the newer entry is ok and the older is not
        } else {
            *self = newer.clone();
        }
    }
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
    /// The goal is to aggregate the results over time:
    /// - 1 per 5 minutes for 60 minutes.
    /// - 1 per hour for 24 hours,
    /// - 1 per day for 7 days,
    /// This is a naive way to aggregate the results over time.
    ///
    /// - If the latest entry is less than an hour old, the tag is the minute
    ///   of the timestamp.
    ///   For example, if the latest entry is at 12:34, the tag is 30.
    /// - If the latest entry is less than a day old, the tag is the hour
    ///   of the timestamp.
    ///   For example, if the latest entry is at 12:34, the tag is 12.
    ///   If the latest entry is at 23:34, the tag is 23.
    /// - If the latest entry is more than a day old, the tag is the day
    ///   of the timestamp.
    ///   For example, if the latest entry is on Monday, the tag is 0.
    ///   If the latest entry is on Sunday, the tag is 6.
    pub fn recreate_tags(&mut self) {
        if let Some(latest_timestamp) = self.latest_timestamp() {
            let last_day = latest_timestamp.date_naive()
                - chrono::Duration::hours(25)
                - chrono::Duration::days(7);
            self.entries.retain_mut(|entry| {
                let delta = latest_timestamp.signed_duration_since(entry.timestamp);
                if delta.num_hours() < 1 {
                    let min: u8 = (entry.timestamp.time().minute() / 5 * 5)
                        .try_into()
                        .unwrap_or(0);
                    entry.tag = TimeTag::Minute(min);
                } else if delta.num_hours() < 1 + 24 {
                    let hour: u8 = entry.timestamp.time().hour().try_into().unwrap_or(0);
                    entry.tag = TimeTag::Hour(hour);
                } else {
                    let date = entry.timestamp.date_naive();
                    if date < last_day {
                        return false;
                    }

                    let day: u8 = date
                        .weekday()
                        .num_days_from_monday()
                        .try_into()
                        .unwrap_or(8);
                    entry.tag = TimeTag::Day(day);
                }
                true
            });
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
        self.entries
            .dedup_by(|left, right| match (&left.tag, &right.tag) {
                (TimeTag::Day(l), TimeTag::Day(r)) if r == l => {
                    right.merge_in(left);
                    true // remove the left entry
                }
                (TimeTag::Hour(l), TimeTag::Hour(r)) if r == l => {
                    right.merge_in(left);
                    true // remove the left entry
                }
                (TimeTag::Minute(l), TimeTag::Minute(r)) if r == l => {
                    right.merge_in(left);
                    true // remove the left entry
                }
                _ => false,
            });
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

    /// Rotate the history
    /// See `CommandHistory::rotate` for more details
    pub fn rotate(&mut self) {
        for command in self.commands.iter_mut() {
            command.rotate();
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
                        exit: e.output.status.code().unwrap_or(-1i32),
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
                        exit: 0,
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
                    exit: 0,
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
                    exit: 0,
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

    #[test]
    fn test_recreate_tags() {
        fn ch_ok(d: &str) -> CommandHistoryEntry {
            CommandHistoryEntry {
                result: Ok(Some(CommandOutput {
                    exit: 0,
                    stdout: "".to_string(),
                    stderr: "".to_string(),
                })),
                timestamp: chrono::DateTime::parse_from_rfc2822(d).unwrap().to_utc(),
                tag: TimeTag::Minute(0),
            }
        }
        let mut history = CommandHistory {
            name: "test".to_string(),
            entries: vec![],
        };
        let test_set = vec![
            ("Tue, 30 Jan 2024 01:41:22 GMT", TimeTag::Day(1)),
            ("Wed, 31 Jan 2024 01:41:22 GMT", TimeTag::Day(2)),
            ("Thu, 01 Feb 2024 01:41:22 GMT", TimeTag::Day(3)),
            ("Fri, 02 Feb 2024 01:41:22 GMT", TimeTag::Day(4)),
            ("Sat, 03 Feb 2024 01:41:22 GMT", TimeTag::Day(5)),
            ("Sun, 04 Feb 2024 01:41:22 GMT", TimeTag::Day(6)),
            ("Mon, 05 Feb 2024 01:41:22 GMT", TimeTag::Day(0)),
            ("Tue, 06 Feb 2024 01:41:22 GMT", TimeTag::Day(1)),
            ("Tue, 06 Feb 2024 18:49:41 GMT", TimeTag::Day(1)),
            ("Tue, 06 Feb 2024 18:49:42 GMT", TimeTag::Day(1)),
            ("Tue, 06 Feb 2024 18:49:43 GMT", TimeTag::Day(1)),
            ("Tue, 06 Feb 2024 18:49:44 GMT", TimeTag::Hour(18)),
            ("Tue, 06 Feb 2024 19:49:44 GMT", TimeTag::Hour(19)),
            ("Tue, 06 Feb 2024 20:41:22 GMT", TimeTag::Hour(20)),
            ("Tue, 06 Feb 2024 21:11:22 GMT", TimeTag::Hour(21)),
            ("Tue, 06 Feb 2024 21:41:22 GMT", TimeTag::Hour(21)),
            ("Tue, 06 Feb 2024 22:41:22 GMT", TimeTag::Hour(22)),
            ("Tue, 06 Feb 2024 23:41:22 GMT", TimeTag::Hour(23)),
            ("Wed, 07 Feb 2024 00:00:00 GMT", TimeTag::Hour(00)),
            ("Wed, 07 Feb 2024 01:41:22 GMT", TimeTag::Hour(01)),
            ("Wed, 07 Feb 2024 07:19:22 GMT", TimeTag::Hour(07)),
            ("Wed, 07 Feb 2024 10:04:22 GMT", TimeTag::Hour(10)),
            ("Wed, 07 Feb 2024 17:14:22 GMT", TimeTag::Hour(17)),
            ("Wed, 07 Feb 2024 17:19:22 GMT", TimeTag::Hour(17)),
            ("Wed, 07 Feb 2024 18:04:22 GMT", TimeTag::Hour(18)),
            ("Wed, 07 Feb 2024 18:09:22 GMT", TimeTag::Hour(18)),
            ("Wed, 07 Feb 2024 18:34:22 GMT", TimeTag::Hour(18)),
            ("Wed, 07 Feb 2024 18:39:22 GMT", TimeTag::Hour(18)),
            ("Wed, 07 Feb 2024 18:44:21 GMT", TimeTag::Hour(18)),
            ("Wed, 07 Feb 2024 18:49:42 GMT", TimeTag::Hour(18)),
            ("Wed, 07 Feb 2024 18:49:43 GMT", TimeTag::Hour(18)),
            ("Wed, 07 Feb 2024 18:49:44 GMT", TimeTag::Minute(45)),
            ("Wed, 07 Feb 2024 18:54:22 GMT", TimeTag::Minute(50)),
            ("Wed, 07 Feb 2024 18:59:22 GMT", TimeTag::Minute(55)),
            ("Wed, 07 Feb 2024 19:04:22 GMT", TimeTag::Minute(0)),
            ("Wed, 07 Feb 2024 19:09:22 GMT", TimeTag::Minute(5)),
            ("Wed, 07 Feb 2024 19:14:22 GMT", TimeTag::Minute(10)),
            ("Wed, 07 Feb 2024 19:19:22 GMT", TimeTag::Minute(15)),
            ("Wed, 07 Feb 2024 19:24:22 GMT", TimeTag::Minute(20)),
            ("Wed, 07 Feb 2024 19:29:22 GMT", TimeTag::Minute(25)),
            ("Wed, 07 Feb 2024 19:34:22 GMT", TimeTag::Minute(30)),
            ("Wed, 07 Feb 2024 19:39:22 GMT", TimeTag::Minute(35)),
            ("Wed, 07 Feb 2024 19:44:21 GMT", TimeTag::Minute(40)),
            ("Wed, 07 Feb 2024 19:49:43 GMT", TimeTag::Minute(45)),
        ];
        for (datetime, _) in test_set.iter() {
            history.entries.push(ch_ok(datetime));
        }
        history.recreate_tags();
        for (datetime, tag) in test_set.into_iter().rev() {
            assert_eq!(
                history.entries.pop().unwrap().tag,
                tag,
                "timestamp: {}",
                datetime
            );
        }
    }

    #[test]
    fn test_recreate_tags_removes_too_old() {
        fn ch_ok(d: &str) -> CommandHistoryEntry {
            CommandHistoryEntry {
                result: Ok(Some(CommandOutput {
                    exit: 0,
                    stdout: "".to_string(),
                    stderr: "".to_string(),
                })),
                timestamp: chrono::DateTime::parse_from_rfc2822(d).unwrap().to_utc(),
                tag: TimeTag::Minute(0),
            }
        }
        let mut history = CommandHistory {
            name: "test".to_string(),
            entries: vec![],
        };
        let test_set = vec![
            "Mon, 29 Jan 2024 23:41:22 GMT",
            "Tue, 30 Jan 2024 01:41:22 GMT",
            "Tue, 30 Jan 2024 18:49:41 GMT",
            "Tue, 30 Jan 2024 18:49:42 GMT",
            "Tue, 30 Jan 2024 18:49:43 GMT",
            "Wed, 07 Feb 2024 19:49:43 GMT",
        ];
        for datetime in test_set.iter() {
            history.entries.push(ch_ok(datetime));
        }
        history.recreate_tags();

        let expected = vec![
            "Tue, 30 Jan 2024 01:41:22 GMT",
            "Tue, 30 Jan 2024 18:49:41 GMT",
            "Tue, 30 Jan 2024 18:49:42 GMT",
            "Tue, 30 Jan 2024 18:49:43 GMT",
            "Wed, 07 Feb 2024 19:49:43 GMT",
        ];
        assert_eq!(history.entries.len(), expected.len(),);
        for datetime in expected.iter().rev() {
            assert_eq!(
                history.entries.pop().unwrap().timestamp,
                chrono::DateTime::parse_from_rfc2822(datetime)
                    .unwrap()
                    .to_utc(),
            );
        }
    }
    #[test]
    fn test_rotate() {
        fn ch_ok(d: &str) -> CommandHistoryEntry {
            CommandHistoryEntry {
                result: Ok(Some(CommandOutput {
                    exit: 0,
                    stdout: format!("ok_stdout_{}", d),
                    stderr: format!("ok_stderr_{}", d),
                })),
                timestamp: chrono::DateTime::parse_from_rfc2822(d).unwrap().to_utc(),
                tag: TimeTag::Minute(0),
            }
        }
        fn ch_err(d: &str) -> CommandHistoryEntry {
            CommandHistoryEntry {
                result: Err(HistoryError::CommandError {
                    exit: -1i32,
                    stdout: format!("err_stdout_{}", d),
                    stderr: format!("err_stderr_{}", d),
                }),
                timestamp: chrono::DateTime::parse_from_rfc2822(d).unwrap().to_utc(),
                tag: TimeTag::Minute(0),
            }
        }
        struct TestCase {
            datetime: &'static str,
            is_ok: bool,
            keep: bool,
            tag: TimeTag, // expected tag for readabiliy
        }
        fn d(u: u8) -> TimeTag {
            TimeTag::Day(u)
        }
        fn h(u: u8) -> TimeTag {
            TimeTag::Hour(u)
        }
        fn m(u: u8) -> TimeTag {
            TimeTag::Minute(u)
        }
        fn t(datetime: &'static str, is_ok: bool, keep: bool, tag: TimeTag) -> TestCase {
            TestCase {
                datetime,
                is_ok,
                keep,
                tag,
            }
        }
        let test_set = vec![
            /* datetime,                      is_ok, keep, tag */
            t("Tue, 30 Jan 2024 00:40:00 GMT", true, false, d(1)),
            t("Tue, 30 Jan 2024 01:41:22 GMT", false, true, d(1)),
            t("Wed, 31 Jan 2024 01:22:22 GMT", true, true, d(2)),
            t("Thu, 01 Feb 2024 01:33:33 GMT", true, true, d(3)),
            t("Fri, 02 Feb 2024 01:44:44 GMT", true, true, d(4)),
            t("Sat, 03 Feb 2024 01:55:55 GMT", true, true, d(5)),
            t("Sun, 04 Feb 2024 01:06:06 GMT", true, true, d(6)),
            t("Mon, 05 Feb 2024 01:00:00 GMT", true, true, d(0)),
            t("Tue, 06 Feb 2024 01:41:22 GMT", true, false, d(1)),
            t("Tue, 06 Feb 2024 18:49:41 GMT", true, false, d(1)),
            t("Tue, 06 Feb 2024 18:49:42 GMT", true, false, d(1)),
            t("Tue, 06 Feb 2024 18:49:43 GMT", true, true, d(1)),
            t("Tue, 06 Feb 2024 18:49:44 GMT", true, true, h(18)),
            t("Tue, 06 Feb 2024 19:49:44 GMT", true, true, h(19)),
            t("Tue, 06 Feb 2024 20:41:22 GMT", true, true, h(20)),
            t("Tue, 06 Feb 2024 21:11:31 GMT", true, false, h(21)),
            t("Tue, 06 Feb 2024 21:41:40 GMT", true, true, h(21)),
            t("Tue, 06 Feb 2024 22:41:59 GMT", true, true, h(22)),
            t("Tue, 06 Feb 2024 23:41:08 GMT", true, true, h(23)),
            t("Wed, 07 Feb 2024 00:00:00 GMT", true, true, h(00)),
            t("Wed, 07 Feb 2024 01:41:22 GMT", true, true, h(01)),
            t("Wed, 07 Feb 2024 07:19:22 GMT", true, true, h(07)),
            t("Wed, 07 Feb 2024 10:04:22 GMT", true, true, h(10)),
            t("Wed, 07 Feb 2024 17:14:22 GMT", true, false, h(17)),
            t("Wed, 07 Feb 2024 17:19:22 GMT", true, true, h(17)),
            t("Wed, 07 Feb 2024 18:04:22 GMT", true, false, h(18)),
            t("Wed, 07 Feb 2024 18:09:22 GMT", true, false, h(18)),
            t("Wed, 07 Feb 2024 18:34:22 GMT", true, false, h(18)),
            t("Wed, 07 Feb 2024 18:39:22 GMT", true, false, h(18)),
            t("Wed, 07 Feb 2024 18:44:21 GMT", true, false, h(18)),
            t("Wed, 07 Feb 2024 18:49:42 GMT", true, false, h(18)),
            t("Wed, 07 Feb 2024 18:49:43 GMT", true, true, h(18)),
            t("Wed, 07 Feb 2024 18:49:44 GMT", true, true, m(45)),
            t("Wed, 07 Feb 2024 18:54:22 GMT", true, true, m(50)),
            t("Wed, 07 Feb 2024 18:55:11 GMT", false, false, m(55)),
            t("Wed, 07 Feb 2024 18:56:33 GMT", true, false, m(55)),
            t("Wed, 07 Feb 2024 18:57:55 GMT", false, true, m(55)),
            t("Wed, 07 Feb 2024 19:04:04 GMT", true, true, m(0)),
            t("Wed, 07 Feb 2024 19:09:22 GMT", true, true, m(5)),
            t("Wed, 07 Feb 2024 19:14:22 GMT", true, true, m(10)),
            t("Wed, 07 Feb 2024 19:18:22 GMT", true, false, m(15)),
            t("Wed, 07 Feb 2024 19:19:22 GMT", false, true, m(15)),
            t("Wed, 07 Feb 2024 19:24:22 GMT", true, true, m(20)),
            t("Wed, 07 Feb 2024 19:29:22 GMT", true, true, m(25)),
            t("Wed, 07 Feb 2024 19:32:55 GMT", true, false, m(30)),
            t("Wed, 07 Feb 2024 19:34:22 GMT", true, true, m(30)),
            t("Wed, 07 Feb 2024 19:39:22 GMT", true, true, m(35)),
            t("Wed, 07 Feb 2024 19:44:21 GMT", true, true, m(40)),
            t("Wed, 07 Feb 2024 19:48:21 GMT", false, true, m(45)),
            t("Wed, 07 Feb 2024 19:49:43 GMT", true, false, m(45)),
        ];
        let mut history = CommandHistory {
            name: "test".to_string(),
            entries: vec![],
        };
        for tc in test_set.iter() {
            if tc.is_ok {
                history.entries.push(ch_ok(tc.datetime));
            } else {
                history.entries.push(ch_err(tc.datetime));
            }
        }
        history.recreate_tags();
        for (idx, tc) in test_set.iter().enumerate() {
            assert_eq!(
                history.entries[idx].tag, tc.tag,
                "index[{}]: {}",
                idx, tc.datetime
            );
        }
        history.rotate();

        for tc in test_set.into_iter().rev() {
            if !tc.keep {
                println!("skipping: {}", tc.datetime);
                continue;
            }
            let che = history.entries.pop().unwrap();
            assert_eq!(
                che.timestamp,
                chrono::DateTime::parse_from_rfc2822(tc.datetime)
                    .unwrap()
                    .to_utc(),
                "timestamp: {} vs {}",
                tc.datetime,
                che.timestamp
            );
            if tc.is_ok {
                assert_eq!(
                    che.result,
                    Ok(Some(CommandOutput {
                        exit: 0,
                        stdout: format!("ok_stdout_{}", tc.datetime),
                        stderr: format!("ok_stderr_{}", tc.datetime),
                    }))
                );
            } else {
                assert_eq!(
                    che.result,
                    Err(HistoryError::CommandError {
                        exit: -1i32,
                        stdout: format!("err_stdout_{}", tc.datetime),
                        stderr: format!("err_stderr_{}", tc.datetime),
                    })
                );
            }
        }
    }
}
