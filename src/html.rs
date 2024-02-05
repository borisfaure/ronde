use crate::history::{CommandHistory, History};
use crate::summary::Summary;
use maud::{html, Markup, PreEscaped, Render, DOCTYPE};

/// Render a header
fn header(summary: &Summary) -> Markup {
    let status = if summary.nb_err == 0 { "✔" } else { "✘" };
    html! {
        (DOCTYPE)
        meta charset="utf-8";
        style { (PreEscaped(include_str!("style.css"))) }
        title {
            (format!("{} {}/{}",
                 status, summary.nb_ok, summary.nb_ok + summary.nb_err))
        }
    }
}

/// Render a CommandResult
impl Render for CommandHistory {
    fn render(&self) -> Markup {
        html! {
            (self.name)
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
