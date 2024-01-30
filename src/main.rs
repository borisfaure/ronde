use clap::{Arg, Command as ClapCommand};
use serde_derive::Deserialize;
use std::fs;
use std::time::Duration;
use tokio::process::Command;

#[derive(Debug, Deserialize)]
/// Command configuration
struct CommandConfig {
    /// Name of the command
    name: String,
    /// Timeout in seconds
    timeout: u64,
    /// Command to run
    run: String,
}

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
    let mut commands: Vec<CommandConfig> = Vec::new();

    for file in yaml_files {
        let file_contents = fs::read_to_string(file)?;
        let file_commands: Vec<CommandConfig> = serde_yaml::from_str(&file_contents)?;
        commands.extend(file_commands);
    }

    for command in commands {
        let result = execute_command(&command).await?;
        println!(
            "Command '{}' executed with exit code {}",
            command.name, result
        );
    }

    Ok(())
}

/// Execute a command
async fn execute_command(command: &CommandConfig) -> Result<i32, Box<dyn std::error::Error>> {
    let mut process = Command::new("sh")
        .arg("-c")
        .arg(&command.run)
        .kill_on_drop(true)
        .spawn()?;

    let status = tokio::time::timeout(Duration::from_secs(command.timeout), process.wait()).await?;

    match status {
        Ok(exit_status) => Ok(exit_status.code().unwrap_or(0)),
        Err(_) => {
            process.kill().await?;
            Ok(-1) // Timed out
        }
    }
}
