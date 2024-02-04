use clap::{Arg, Command as ClapCommand};
use futures::future::join_all;

/// Module to load configuration
mod config;
/// Module to store history
mod history;
/// Module to generate HTML output
mod html;
/// Module to run commands
mod runner;

/// Build a Command
fn build_cli() -> ClapCommand {
    ClapCommand::new("ronde")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Boris Faure <boris.faure@gmail.com>")
        .about("Monitor your servers and services with alerting and a simple status page")
        .arg(
            Arg::new("ConfigFile")
                .value_name("YamlConfigFile")
                .num_args(1)
                .required(true)
                .help("YAML Config file describing the services to monitor"),
        )
        .arg(
            Arg::new("HistoryFile")
                .value_name("HistoryFile")
                .num_args(1)
                .short('h')
                .long("history")
                .required(true)
                .help("History file"),
        )
        .arg(
            Arg::new("OutputFile")
                .value_name("HtmlOutputFile")
                .num_args(1)
                .short('o')
                .long("output")
                .required(true)
                .help("HTML Output file"),
        )
}

#[tokio::main]
/// Main function
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = build_cli().get_matches();

    let yaml_file = matches.get_one::<String>("ConfigFile").unwrap();
    let commands = config::load(yaml_file).await?;

    let results = join_all(commands.into_iter().map(runner::execute_command)).await;

    let history_file = matches.get_one::<String>("HistoryFile").unwrap();
    let _history = history::load(&history_file).await?;

    let html = html::generate(&results);
    let output_file = matches.get_one::<String>("OutputFile").unwrap();
    std::fs::write(output_file, html)?;

    Ok(())
}
