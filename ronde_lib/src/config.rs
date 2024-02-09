use serde_derive::{Deserialize, Serialize};
use tokio::fs;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
/// Command configuration
pub struct CommandConfig {
    /// Name of the command
    pub name: String,
    /// Timeout in seconds
    pub timeout: u64,
    /// Command to run
    pub run: String,
}

/// Load configuration from YAML files
pub async fn load(yaml_file: &String) -> Result<Vec<CommandConfig>, Box<dyn std::error::Error>> {
    let file_contents = fs::read_to_string(yaml_file).await?;
    let commands: Vec<CommandConfig> = serde_yaml::from_str(&file_contents)?;
    Ok(commands)
}
