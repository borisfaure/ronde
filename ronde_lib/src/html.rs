use crate::history::{CommandHistory, CommandHistoryEntry, History, HistoryError, TimeTag};
use crate::summary::Summary;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde_derive::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug, Serialize, PartialEq)]
pub struct CommandHistoryEntryDetails {
    #[serde(rename = "i")]
    pub is_error: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "x")]
    pub exit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "t")]
    pub timeout: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "o")]
    pub stdout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "e")]
    pub stderr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "m")]
    pub message: Option<String>,
    #[serde(rename = "c")]
    pub command: String,
}
impl CommandHistoryEntryDetails {
    /// Create a new CommandHistoryEntryDetails
    pub fn new(entry: &CommandHistoryEntry) -> CommandHistoryEntryDetails {
        let (is_error, exit, timeout, stdout, stderr, message) = match &entry.result {
            Ok(output) => (
                false,
                Some(output.exit),
                None,
                Some(output.stdout.clone()),
                Some(output.stderr.clone()),
                None,
            ),
            Err(HistoryError::Timeout { timeout }) => {
                (true, None, Some(*timeout), None, None, None)
            }
            Err(HistoryError::CommandError {
                exit,
                stdout,
                stderr,
            }) => (
                true,
                Some(*exit),
                None,
                Some(stdout.clone()),
                Some(stderr.clone()),
                None,
            ),
            Err(HistoryError::Other { message }) => {
                (true, None, None, None, None, Some(message.clone()))
            }
        };
        CommandHistoryEntryDetails {
            is_error,
            exit,
            timeout,
            stdout,
            stderr,
            message,
            command: entry.command.clone(),
        }
    }
}

/// History details of a command
#[derive(Debug, Serialize)]
struct CommandHistoryDetails {
    h: HashMap<String, CommandHistoryEntryDetails>,
}

impl CommandHistoryDetails {
    /// Create a new CommandHistoryDetails
    fn new(history: &CommandHistory) -> CommandHistoryDetails {
        let mut h = HashMap::new();
        for entry in history.entries.iter() {
            let details = CommandHistoryEntryDetails::new(entry);
            h.insert(entry.timestamp.to_rfc2822(), details);
        }
        CommandHistoryDetails { h }
    }
}

/// Write a static file into the output directory if it does not exist or if
/// the size is different.
async fn write_static_file(
    output_dir: &str,
    filename: &str,
    content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut output_path = PathBuf::from(output_dir);
    output_path.push(filename);
    let path = output_path.as_path();
    match fs::metadata(path).await {
        Ok(metadata) if metadata.len() != content.len() as u64 => {
            fs::write(path, content).await?;
        }
        Err(_) => {
            fs::write(path, content).await?;
        }
        _ => {}
    }
    Ok(())
}

/// CommandHistoryEntrySummary
#[derive(Debug, Serialize)]
struct CommandHistoryEntrySummary {
    #[serde(rename = "t")]
    timestamp: String,
    #[serde(rename = "v")]
    tag_value: String,
    #[serde(rename = "k")]
    tag_kind: String,
    #[serde(rename = "e")]
    is_error: bool,
}
impl CommandHistoryEntrySummary {
    /// Create a new CommandHistoryEntrySummary
    fn new(entry: &CommandHistoryEntry) -> CommandHistoryEntrySummary {
        let (tag_kind, tag_value) = match entry.tag {
            TimeTag::Minute(m) => ("m".to_string(), format!("{:02}", m)),
            TimeTag::Hour(h) => ("h".to_string(), format!("{:02}", h)),
            TimeTag::Day(d) => match d {
                0 => ("d".to_string(), "Mo".to_string()),
                1 => ("d".to_string(), "Tu".to_string()),
                2 => ("d".to_string(), "We".to_string()),
                3 => ("d".to_string(), "Th".to_string()),
                4 => ("d".to_string(), "Fr".to_string()),
                5 => ("d".to_string(), "Sa".to_string()),
                _ => ("d".to_string(), "Su".to_string()),
            },
        };
        CommandHistoryEntrySummary {
            timestamp: entry.timestamp.to_rfc2822(),
            tag_value,
            tag_kind,
            is_error: entry.result.is_err(),
        }
    }
}

/// CommandHistorySummary
#[derive(Debug, Serialize)]
struct CommandHistorySummary {
    #[serde(rename = "n")]
    name: String,
    #[serde(rename = "i")]
    id: String,
    #[serde(rename = "e")]
    entries: Vec<CommandHistoryEntrySummary>,
}

/// Main JSON structure
#[derive(Debug, Serialize)]
struct MainJson {
    #[serde(rename = "s")]
    summary: Summary,
    #[serde(rename = "c")]
    commands: Vec<CommandHistorySummary>,
    #[serde(rename = "t")]
    title: String,
}

/// Generate an id from a name
/// The id is suitable as an HTML id & a filename
fn generate_id(name: &String) -> String {
    URL_SAFE_NO_PAD.encode(blake3::hash(name.as_bytes()).as_bytes())
}

impl MainJson {
    /// Create a new MainJson
    fn new(summary: Summary, history: &History, title: String) -> MainJson {
        let commands = history
            .commands
            .iter()
            .map(|command| CommandHistorySummary {
                name: command.name.clone(),
                id: generate_id(&command.name),
                entries: command
                    .entries
                    .iter()
                    .map(CommandHistoryEntrySummary::new)
                    .collect(),
            })
            .collect();
        MainJson {
            summary,
            commands,
            title,
        }
    }
}

/// Generate auxiliary files into the output directory if they do not exist or
/// it their size is different.
pub async fn generate_auxiliary_files(output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    write_static_file(output_dir, "style.css", include_str!("../assets/style.css")).await?;
    write_static_file(output_dir, "main.js", include_str!("../assets/main.js")).await?;
    write_static_file(
        output_dir,
        "index.html",
        include_str!("../assets/index.html"),
    )
    .await?;
    Ok(())
}

/// Generate JSON files into the output directory
pub async fn generate_json_files(
    output_dir: &str,
    summary: Summary,
    history: &History,
    name: String,
) -> Result<(), Box<dyn std::error::Error>> {
    for command in &history.commands {
        let command_history_details = CommandHistoryDetails::new(command);
        let json = serde_json::to_string(&command_history_details)?;
        let mut output_path = PathBuf::from(output_dir);
        output_path.push(format!("{}.json", generate_id(&command.name)));
        let path = output_path.as_path();
        fs::write(path, json).await?;
    }

    let mut output_path = PathBuf::from(output_dir);
    output_path.push("main.json");
    let path = output_path.as_path();
    let main = MainJson::new(summary, history, name);
    let main_json = serde_json::to_string(&main)?;
    fs::write(path, main_json).await?;
    Ok(())
}
