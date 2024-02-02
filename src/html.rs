use crate::runner::CommandResult;
use maud::{html, Markup, PreEscaped, Render, DOCTYPE};
use std::vec::Vec;

/// Render a header
fn header(summary: &Summary) -> Markup {
    let status = if summary.nb_err == 0 { "✔" } else { "✘" };
    html! {
        (DOCTYPE)
        meta charset="utf-8";
        title {
        (format!("{} {}/{}",
                 status,
                        summary.nb_ok, summary.nb_ok + summary.nb_err))
        }
    }
}

/// Render a CommandResult
impl Render for CommandResult {
    fn render(&self) -> Markup {
        match &self.result {
            Ok(output) => {
                html! {
                    details class="ok" {
                        summary { (self.config.name) }
                        pre { (PreEscaped(String::from_utf8_lossy(&output.stdout))) }
                        pre { (PreEscaped(String::from_utf8_lossy(&output.stderr))) }
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
    }
}

/// Summary of the command results
struct Summary {
    /// Number of successful commands
    nb_ok: u32,
    /// Number of failed commands
    nb_err: u32,
}

/// Get Summary of the command results
fn summary(results: &Vec<CommandResult>) -> Summary {
    let mut nb_ok = 0;
    let mut nb_err = 0;
    for result in results {
        match &result.result {
            Ok(_) => nb_ok += 1,
            Err(_) => nb_err += 1,
        }
    }
    Summary { nb_ok, nb_err }
}

/// Render a Summary
impl Render for Summary {
    fn render(&self) -> Markup {
        if self.nb_err == 0 {
            html! {
                h1 class="ok" {
                    (format!("{} commands succeeded", self.nb_ok))
                }
            }
        } else {
            let plural = if self.nb_err == 1 { "" } else { "s" };
            html! {
                h1 class="err" {
                    (format!("{} command{} failed", self.nb_err, plural))
                }
            }
        }
    }
}

/// Purpose: Generate HTML from the results of a ronde run.
pub fn generate(results: &Vec<CommandResult>) -> String {
    let summary = summary(results);
    let markup = html! {
        (header(&summary))
        (summary)
        @for result in results {
            (result)
        }
    };
    markup.into_string()
}
