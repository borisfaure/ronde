use crate::config::CommandConfig;
use std::process::Output;
use std::time::Duration;
use tokio::process::Command;

/// Execute a command
pub async fn execute_command(command: &CommandConfig) -> Result<Output, std::io::Error> {
    let child = Command::new("sh")
        .arg("-c")
        .arg(&command.run)
        .kill_on_drop(true)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    tokio::time::timeout(
        Duration::from_secs(command.timeout),
        child.wait_with_output(),
    )
    .await?
}
