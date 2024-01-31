use clap::parser::ValuesRef;
use serde_derive::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
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
pub fn load(
    yaml_files: ValuesRef<'_, String>,
) -> Result<Vec<CommandConfig>, Box<dyn std::error::Error>> {
    let mut commands: Vec<CommandConfig> = Vec::new();
    for file in yaml_files {
        let file_contents = fs::read_to_string(file)?;
        let file_commands: Vec<CommandConfig> = serde_yaml::from_str(&file_contents)?;
        commands.extend(file_commands);
    }
    Ok(commands)
}
