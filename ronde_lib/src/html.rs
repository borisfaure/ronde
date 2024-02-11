use crate::history::{CommandHistory, CommandHistoryEntry, History, HistoryError, TimeTag};
use crate::runner::CommandOutput;
use crate::summary::Summary;
use maud::{html, Markup, PreEscaped, Render, DOCTYPE};

/// Render a header
fn header(summary: &Summary) -> Markup {
    let status = if summary.nb_err == 0 { "✔" } else { "✘" };
    html! {
        (DOCTYPE)
        meta charset="utf-8";
        meta name="viewport" content="width=device-width, initial-scale=1";
        meta http-equiv="Content-Security-Policy" content="script-src 'nonce-ronde'";
        style { (PreEscaped(include_str!("style.css"))) }
        //script nonce="ronde" { (PreEscaped(include_str!("main.js"))) }
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
            div class="hidden" id=(gen_id(self.idx, self.top_idx)) {
                h3 {
                    (self.entry.timestamp.to_rfc2822())
                }
            }
        }
    }
}

struct HistoryEntryEnumeratedSummary<'a> {
    idx: usize,
    top_idx: usize,
    is_error: bool,
    timestamp: &'a chrono::DateTime<chrono::Utc>,
    tag: &'a TimeTag,
}
/// Render a HistoryEntryEnumeratedSummary
impl Render for HistoryEntryEnumeratedSummary<'_> {
    fn render(&self) -> Markup {
        let klass = if self.is_error { "bean err" } else { "bean ok" };
        let title = self.timestamp.to_rfc2822();
        let toggle = gen_id(self.idx, self.top_idx);
        html! {
            div class=(klass) title=(title) data-toggle=(toggle) {
                (self.tag)
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
                            is_error: entry.result.is_err(),
                            timestamp: &entry.timestamp,
                            tag: &entry.tag,
                            })
                    }
                }
                div class="details" {
                    @for (idx,entry) in self.history_item.entries.iter().enumerate() {
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
