use crate::runner::CommandResult;
use maud::{html, Markup, PreEscaped, Render, DOCTYPE};
use std::vec::Vec;

/// A basic header with a dynamic `page_title`.
fn header(summary: &Summary) -> Markup {
    html! {
        (DOCTYPE)
        meta charset="utf-8";
        title {
        (format!("Ronde status - {}/{}",
                        summary.nb_ok, summary.nb_ok + summary.nb_err))
        }
    }
}

impl Render for CommandResult {
    fn render(&self) -> Markup {
        html! {
            details {
                summary { (self.config.name) }
                @match &self.result {
                    Ok(output) => {
                        pre { (PreEscaped(String::from_utf8_lossy(&output.stdout))) }
                        pre { (PreEscaped(String::from_utf8_lossy(&output.stderr))) }
                    }
                    Err(e) => {
                        p { (e) }
                    }
                }
            }
        }
    }
}

/// Summary of the command results
struct Summary {
    nb_ok: u32,
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

/// Purpose: Generate HTML from the results of a ronde run.
pub fn generate(results: &Vec<CommandResult>) -> String {
    let summary = summary(results);
    let markup = html! {
        (header(&summary))
        h1 {
            (format!("Ronde status - {}/{}",
                        summary.nb_ok, summary.nb_ok + summary.nb_err))
        }
        @for result in results {
            (result)
        }
    };
    markup.into_string()
}
