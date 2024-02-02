use crate::runner::CommandResult;
use maud::{html, Markup, PreEscaped, Render, DOCTYPE};
use std::vec::Vec;

/// A basic header with a dynamic `page_title`.
fn header(page_title: &str) -> Markup {
    html! {
        (DOCTYPE)
        meta charset="utf-8";
        title { (page_title) }
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

/// Purpose: Generate HTML from the results of a ronde run.
pub fn generate(results: &Vec<CommandResult>) -> String {
    let markup = html! {
        (header("Ronde status report"))
        h1 { "Ronde status report" }
            @for result in results {
                (result)
            }
    };
    markup.into_string()
}
