use clap::{Arg, Command as ClapCommand};
use futures::future::join_all;
use std::path::PathBuf;

use ronde_lib::config::Config;
use ronde_lib::history::History;
use ronde_lib::html;
use ronde_lib::notification::check_and_send_notifications;
use ronde_lib::runner;
use ronde_lib::summary::Summary;

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
                .short('c')
                .long("config")
                .required(true)
                .help("YAML Config file describing the services to monitor"),
        )
}

#[tokio::main]
/// Main function
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = build_cli().get_matches();

    let yaml_file = matches.get_one::<String>("ConfigFile").unwrap();
    let config = Config::load(yaml_file).await?;

    let results = join_all(config.commands.into_iter().map(runner::execute_command)).await;

    let mut history = History::load(&config.history_file).await?;

    history.purge_from_results(&results);
    let summary = Summary::from_results(&results);
    history.update(results);
    history.recreate_tags();
    history.rotate();

    let html = html::generate(summary, &history);
    let mut output_path = PathBuf::from(config.output_dir);
    output_path.push("index.html");
    tokio::fs::write(output_path.as_path(), html).await?;

    if let Some(ref nconfig) = config.notifications {
        check_and_send_notifications(nconfig, &history).await?;
    }

    history.save(&config.history_file).await?;
    Ok(())
}
