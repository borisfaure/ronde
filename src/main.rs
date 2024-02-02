use clap::{Arg, Command as ClapCommand};
use futures::future::join_all;

/// Module to load configuration
mod config;
/// Module to generate HTML output
mod html;
/// Module to run commands
mod runner;

/// Build a Command
fn build_cli() -> ClapCommand {
    ClapCommand::new("ronde")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Boris Faure <boris.faure@gmail.com>")
        .about("Keep an eye on your services")
        .arg(
            Arg::new("ConfigFile")
                .value_name("YamlConfigFile")
                .num_args(1)
                .required(true)
                .help("YAML Config file describing the services to monitor"),
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
    let commands = config::load(yaml_file)?;

    let results = join_all(commands.into_iter().map(|c| runner::execute_command(c))).await;

    let html = html::generate(&results);
    let output_file = matches.get_one::<String>("OutputFile").unwrap();
    std::fs::write(output_file, html)?;

    Ok(())
}
