use serde_derive::Deserialize;
use std::fs;
use std::time::Duration;
use tokio::process::Command;

#[derive(Debug, Deserialize)]
struct CommandConfig {
    name: String,
    timeout: u64,
    run: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <yaml-file>", args[0]);
        std::process::exit(1);
    }

    let yaml_file_path = &args[1];

    let file_contents = fs::read_to_string(yaml_file_path)?;
    let commands: Vec<CommandConfig> = serde_yaml::from_str(&file_contents)?;

    for command in commands {
        let result = execute_command(&command).await?;
        println!(
            "Command '{}' executed with exit code {}",
            command.name, result
        );
    }

    Ok(())
}

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
