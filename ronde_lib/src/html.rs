use crate::history::{CommandHistory, CommandHistoryEntry, History, TimeTag};
use crate::summary::Summary;
use chrono::{DateTime, Utc};
use maud::{html, Markup, PreEscaped, Render, DOCTYPE};

/// Render a header
fn header(summary: &Summary) -> Markup {
    let status = if summary.nb_err == 0 { "✔" } else { "✘" };
    html! {
        (DOCTYPE)
        meta charset="utf-8";
        meta name="viewport" content="width=device-width, initial-scale=1";
        style { (PreEscaped(include_str!("style.css"))) }
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
            TimeTag::Day(_) => html! { "D" },
        }
    }
}

/// Summary of a command history entry
struct CommandHistoryEntrySummary {
    timestamp: DateTime<Utc>,
    on_error: bool,
    tag: TimeTag,
}
impl CommandHistoryEntrySummary {
    fn from_entry(entry: &CommandHistoryEntry) -> Self {
        CommandHistoryEntrySummary {
            timestamp: entry.timestamp,
            on_error: entry.result.is_err(),
            tag: entry.tag.clone(),
        }
    }
}
impl Render for CommandHistoryEntrySummary {
    fn render(&self) -> Markup {
        let klass = if self.on_error { "bean err" } else { "bean ok" };
        let title = self.timestamp.to_rfc2822();
        html! {
            div class=(klass) title=(title) {
                (self.tag)
            }
        }
    }
}

/// Render a CommandResult
impl Render for CommandHistory {
    fn render(&self) -> Markup {
        html! {
            div class="command" {
                h2 { (self.name) }
                div class="bar" {
                    @for entry in self.entries.iter() {
                        (CommandHistoryEntrySummary::from_entry(&entry))
                    }
                }
            }
        }
        /*
        match &self.result {
            Ok(output) => {
                html! {
                    details class="ok" {
                        summary { (self.config.name) }
                        pre { (PreEscaped(output.stdout)) }
                        pre { (PreEscaped(output.stderr)) }
                    }
                }
            }
            Err(e) => {
                html! {
                    details class="err" {
                        summary { (self.config.name) }
                        p { (e) }
                    }
                }
            }
        }
        */
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
                    (format!("⚠ {} command{} failed ⚠", self.nb_err, plural))
                }
            }
        }
    }
}

/// Render a History
impl Render for History {
    fn render(&self) -> Markup {
        html! {
            @for history_item in self.commands.iter() {
                (history_item)
            }
        }
    }
}

/// Purpose: Generate HTML from the results of a ronde run.
pub fn generate(summary: Summary, history: &History) -> String {
    let markup = html! {
        (header(&summary))
        (summary)
        @for history_item in history.commands.iter() {
            (history_item)
        }
    };
    markup.into_string()
}
