use crate::config::CommandConfig;
use serde_derive::{Deserialize, Serialize};
use std::process::Output;
use std::time::Duration;
use thiserror::Error;
use tokio::process::Command;

/// Command output
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CommandOutput {
    /// status code
    pub exit: i32,
    /// stdout
    pub stdout: String,
    /// stderr
    pub stderr: String,
}

impl From<Output> for CommandOutput {
    fn from(output: Output) -> Self {
        CommandOutput {
            exit: output.status.code().unwrap_or(-1i32),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        }
    }
}

impl std::fmt::Display for CommandOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Command Output: code: {}, stdout: {}, stderr: {}",
            self.exit, self.stdout, self.stderr
        )
    }
}

/// Command returned an error
#[derive(Error, Debug)]
pub struct ReturnedError {
    pub output: Output,
}
impl std::fmt::Display for ReturnedError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Command error: code: {}, stdout: {}, stderr: {}",
            self.output.status.code().unwrap_or(-1i32),
            String::from_utf8_lossy(&self.output.stdout),
            String::from_utf8_lossy(&self.output.stderr)
        )
    }
}

/// Command error
#[derive(Error, Debug)]
pub enum CommandError {
    /// Timeout
    #[error("TimedOut")]
    TimedOut(#[from] tokio::time::error::Elapsed),
    /// Command error
    #[error("Command error: {0}")]
    Command(#[from] std::io::Error),
    /// Returned error
    #[error("Returned error: {0}")]
    ReturnedError(#[from] ReturnedError),
}

/// Command result
#[derive(Debug)]
pub struct CommandResult {
    /// Command configuration
    pub config: CommandConfig,
    /// Result of the command
    pub result: Result<CommandOutput, CommandError>,
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
    pub fn ok(config: CommandConfig, output: CommandOutput) -> CommandResult {
        CommandResult {
            config,
            result: Ok(output),
        }
    }
}

/// Execute a command
pub async fn execute_command(config: CommandConfig) -> CommandResult {
    let mut cmd = Command::new("sh");
    let mut cmd = cmd
        .arg("-c")
        .arg(&config.run)
        .kill_on_drop(true)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    if let Some(uid) = config.uid {
        cmd = cmd.uid(uid);
    }
    if let Some(gid) = config.gid {
        cmd = cmd.gid(gid);
    }
    if let Some(cwd) = &config.cwd {
        cmd = cmd.current_dir(cwd);
    }
    if config.clear_env {
        cmd = cmd.env_clear();
    }
    if let Some(env) = &config.env {
        cmd = cmd.envs(env.iter());
    }

    match cmd.spawn() {
        Ok(child) => {
            let output = tokio::time::timeout(
                Duration::from_secs(config.timeout.0 as u64),
                child.wait_with_output(),
            )
            .await;
            match output {
                Ok(Ok(output)) if output.status.success() => {
                    CommandResult::ok(config, output.into())
                }
                Ok(Ok(output)) => CommandResult::error(config, ReturnedError { output }.into()),

                Ok(Err(e)) => CommandResult::error(config, e.into()),
                Err(e) => CommandResult::error(config, e.into()),
            }
        }
        Err(e) => CommandResult::error(config, e.into()),
    }
}
