use clap::{Arg, Command as ClapCommand};
use futures::future::join_all;

/// Module to load configuration
mod config;
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
}

#[tokio::main]
/// Main function
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = build_cli().get_matches();

    let yaml_file = matches.get_one::<String>("ConfigFile").unwrap();
    let commands = config::load(yaml_file)?;

    let results = join_all(commands.into_iter().map(|c| runner::execute_command(c))).await;

    for result in results {
        match result.result {
            Ok(output) => {
                println!(
                    "Command '{}' executed with exit code {}, stdout: '{}', stderr: '{}'",
                    result.config.name,
                    output.status.code().unwrap_or(255),
                    output.stdout.iter().map(|&c| c as char).collect::<String>(),
                    output.stderr.iter().map(|&c| c as char).collect::<String>()
                );
            }
            Err(e) => {
                println!("Command '{}' failed: {}", result.config.name, e);
            }
        }
    }

    Ok(())
}
