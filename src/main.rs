use clap::{Arg, Command as ClapCommand};

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
                .num_args(1..)
                .required(true)
                .help("YAML Config file describing the services to monitor"),
        )
}

#[tokio::main]
/// Main function
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = build_cli().get_matches();

    let yaml_files = matches.get_many::<String>("ConfigFile").unwrap();
    let commands = config::load(yaml_files)?;

    for command in commands {
        let result = runner::execute_command(&command).await?;
        println!(
            "Command '{}' executed with exit code {}, stdout: '{}', stderr: '{}'",
            command.name,
            result.status.code().unwrap_or(255),
            result.stdout.iter().map(|&c| c as char).collect::<String>(),
            result.stderr.iter().map(|&c| c as char).collect::<String>()
        );
    }

    Ok(())
}
