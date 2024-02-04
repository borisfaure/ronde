use crate::config::CommandConfig;
use std::process::Output;
use std::time::Duration;
use thiserror::Error;
use tokio::process::Command;

/// Command error
#[derive(Error, Debug)]
pub enum CommandError {
    /// Timeout
    #[error("TimedOut")]
    TimedOut(#[from] tokio::time::error::Elapsed),
    /// Command error
    #[error("Command error: {0}")]
    Command(#[from] std::io::Error),
}

/// Command result
#[derive(Debug)]
pub struct CommandResult {
    /// Command configuration
    pub config: CommandConfig,
    /// Result of the command
    pub result: Result<Output, CommandError>,
}

impl CommandResult {
    /// Create a new CommandResult with an Err result
    pub fn error(config: CommandConfig, error: CommandError) -> CommandResult {
        CommandResult {
            config,
            result: Err(error),
        }
    }
    /// Create a new CommandResult with an Ok result
    pub fn ok(config: CommandConfig, output: Output) -> CommandResult {
        CommandResult {
            config,
            result: Ok(output),
        }
    }
}

/// Execute a command
pub async fn execute_command(config: CommandConfig) -> CommandResult {
    let child = Command::new("sh")
        .arg("-c")
        .arg(&config.run)
        .kill_on_drop(true)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn();
    match child {
        Ok(child) => {
            let output = tokio::time::timeout(
                Duration::from_secs(config.timeout),
                child.wait_with_output(),
            )
            .await;
            match output {
                Ok(Ok(output)) => CommandResult::ok(config, output),
                Ok(Err(e)) => CommandResult::error(config, e.into()),
                Err(e) => CommandResult::error(config, e.into()),
            }
        }
        Err(e) => CommandResult::error(config, e.into()),
    }
}
