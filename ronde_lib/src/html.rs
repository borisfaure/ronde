use crate::error::RondeError;
use crate::history::{CommandHistory, CommandHistoryEntry, History, HistoryError, TimeTag};
use crate::runner::CommandOutput;
use crate::summary::Summary;
use maud::{html, Markup, PreEscaped, Render, DOCTYPE};
use serde_derive::Serialize;
use std::path::PathBuf;
use tokio::fs;

/// Render a header
fn header(summary: &Summary) -> Markup {
    let status = if summary.nb_err == 0 { "✔" } else { "✘" };
    html! {
        (DOCTYPE)
        meta charset="utf-8";
        meta name="viewport" content="width=device-width, initial-scale=1";
        meta http-equiv="Content-Security-Policy" content="script-src 'nonce-ronde'";
        link rel="stylesheet" href="style.css";
        script src="main.js" {};
        title {
            (format!("{} {}/{}",
                 status, summary.nb_ok, summary.nb_ok + summary.nb_err))
        }
    }
}

impl Render for TimeTag {
    fn render(&self) -> Markup {
        match self {
            TimeTag::Minute(m) => html! { (format!("{:02}",m)) },
            TimeTag::Hour(h) => html! { (format!("{:02}",h)) },
            TimeTag::Day(0) => html! { "Mo" },
            TimeTag::Day(1) => html! { "Tu" },
            TimeTag::Day(2) => html! { "We" },
            TimeTag::Day(3) => html! { "Th" },
            TimeTag::Day(4) => html! { "Fr" },
            TimeTag::Day(5) => html! { "Sa" },
            TimeTag::Day(_) => html! { "Su" },
        }
    }
}

impl Render for HistoryError {
    fn render(&self) -> Markup {
        match self {
            HistoryError::Timeout { timeout } => html! {
                b { (format!("Timeout {}s", timeout)) }
            },
            HistoryError::CommandError {
                exit,
                stdout,
                stderr,
            } => {
                html! {
                    div class="error" {
                        p { (format!("Exit code: {}", exit)) }
                        p { "stdout" }
                        pre { (PreEscaped(stdout)) }
                        p { "stderr" }
                        pre { (PreEscaped(stderr)) }
                    }
                }
            }
            HistoryError::Other { message } => html! {
                pre {
                    (message)
                }
            },
        }
    }
}

impl Render for CommandOutput {
    fn render(&self) -> Markup {
        html! {
            div class="output" {

                p { "stdout" }
                pre { (PreEscaped(&self.stdout)) }
                p { "stderr" }
                pre { (PreEscaped(&self.stderr)) }
            }
        }
    }
}

fn gen_id(idx: usize, top_idx: usize) -> String {
    format!("entry_{}_{}", top_idx, idx)
}

struct HistoryEntryEnumeratedDetails<'a> {
    idx: usize,
    top_idx: usize,
    entry: &'a CommandHistoryEntry,
}
impl Render for HistoryEntryEnumeratedDetails<'_> {
    fn render(&self) -> Markup {
        html! {
            div class="details hidden" id=(gen_id(self.idx, self.top_idx)) {
                h3 {
                    (self.entry.timestamp.to_rfc2822())
                }
                p { "Command" }
                pre { (self.entry.command) }
                @match &self.entry.result {
                    Ok(output) => {
                        (output)
                    }
                    Err(err) => {
                        (err)
                    }
                }
            }
        }
    }
}

struct HistoryEntryEnumeratedSummary<'a> {
    idx: usize,
    top_idx: usize,
    is_error: bool,
    have_details: bool,
    timestamp: &'a chrono::DateTime<chrono::Utc>,
    tag: &'a TimeTag,
}
/// Render a HistoryEntryEnumeratedSummary
impl Render for HistoryEntryEnumeratedSummary<'_> {
    fn render(&self) -> Markup {
        let class_tag = match self.tag {
            TimeTag::Minute(_) => "minute",
            TimeTag::Hour(_) => "hour",
            TimeTag::Day(_) => "day",
        };
        let class_err = if self.is_error { "err" } else { "ok" };
        let klass = format!("bean {} {}", class_tag, class_err);
        let title = self.timestamp.to_rfc2822();
        if self.have_details {
            let toggle = gen_id(self.idx, self.top_idx);
            html! {
                div class=(klass) title=(title) data-toggle=(toggle) {
                    (self.tag)
                }
            }
        } else {
            html! {
                div class=(klass) title=(title) {
                    (self.tag)
                }
            }
        }
    }
}

struct CommandHistoryEnumareted<'a> {
    idx: usize,
    history_item: &'a CommandHistory,
}
/// Render a CommandResult
impl Render for CommandHistoryEnumareted<'_> {
    fn render(&self) -> Markup {
        html! {
            div class="command" {
                h2 { (self.history_item.name) }
                div class="bar" {
                    @for (idx,entry) in self.history_item.entries.iter().enumerate() {
                        (HistoryEntryEnumeratedSummary {
                            idx,
                            top_idx: self.idx,
                            have_details: idx == self.history_item.entries.len() - 1 || entry.result.is_err(),
                            is_error: entry.result.is_err(),
                            timestamp: &entry.timestamp,
                            tag: &entry.tag,
                            })
                    }
                }
                div class="details_container" {
                    @for (idx,entry) in self.history_item.entries.iter().enumerate() {
                        @if idx == self.history_item.entries.len() - 1 || entry.result.is_err() {
                        (HistoryEntryEnumeratedDetails {
                            idx,
                            top_idx: self.idx,
                            entry,
                        })
                        }
                    }
                }
            }
        }
    }
}

/// Render a Summary
impl Render for Summary {
    fn render(&self) -> Markup {
        if self.nb_err == 0 {
            html! {
                h1 class="ok" {
                    "✔ All Systems Operational"
                }
            }
        } else {
            let plural = if self.nb_err == 1 { "" } else { "s" };
            html! {
                h1 class="err" {
                    (format!("⚠ {} command{} failed", self.nb_err, plural))
                }
            }
        }
    }
}

/// Render a History
impl Render for History {
    fn render(&self) -> Markup {
        html! {
            @for (idx, history_item) in self.commands.iter().enumerate() {
                (CommandHistoryEnumareted {
                    idx,
                    history_item,
                })
            }
        }
    }
}

/// Purpose: Generate HTML from the results of a ronde run.
pub fn generate(summary: Summary, history: &History) -> String {
    let markup = html! {
        (header(&summary))
        (summary)
        (history)
    };
    markup.into_string()
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

impl MainJson {
    /// Create a new MainJson
    fn new(summary: Summary, history: &History, title: String) -> MainJson {
        let commands = history
            .commands
            .iter()
            .map(|command| CommandHistorySummary {
                name: command.name.clone(),
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

/// Generate the main.json file into the output directory.
pub fn generate_main(
    summary: Summary,
    history: &History,
    name: String,
) -> Result<String, RondeError> {
    let main = MainJson::new(summary, history, name);
    Ok(serde_json::to_string(&main)?)
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
