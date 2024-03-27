use crate::error::RondeError;
use serde_derive::Deserialize;
use std::collections::HashSet;
use thiserror::Error;
use tokio::fs;

/// Timeout in seconds
#[derive(Debug, PartialEq, Deserialize)]
pub struct Timeout(pub u16);

impl Default for Timeout {
    fn default() -> Self {
        Timeout(60)
    }
}

#[derive(Debug, Default, PartialEq, Deserialize)]
/// Command configuration
pub struct CommandConfig {
    /// Name of the command
    pub name: String,
    /// Timeout in seconds
    #[serde(default)]
    pub timeout: Timeout,
    /// Command to run
    pub run: String,
    /// UID to use to run the command
    pub uid: Option<u32>,
    /// GID to use to run the command
    pub gid: Option<u32>,
}

#[derive(Debug, Default, PartialEq, Deserialize)]
/// Pushover configuration
pub struct PushoverConfig {
    /// User key
    pub user: String,
    /// API token
    pub token: String,
    /// Optional url to link to
    pub url: Option<String>,
}

#[derive(Debug, Default, PartialEq, Deserialize)]
/// Notification configuration
pub struct NotificationConfig {
    /// Pushover configuration
    pub pushover: Option<PushoverConfig>,
    /// Notify on success after failure
    #[serde(default)]
    pub notify_on_success_after_failure: bool,
}

/// Error type for configuration
#[derive(Error, Debug)]
pub enum ConfigError {
    /// IO Error
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    /// SerdeYaml Error
    #[error("Serde YAML Error: {0}")]
    SerdeYamlError(#[from] serde_yaml::Error),
    /// Command name is not unique
    #[error("Command name {0} is not unique")]
    NotUniqueCommandName(String),
}

#[derive(Debug, Default, PartialEq, Deserialize)]
/// Configuration
pub struct Config {
    /// Name of the site to display
    pub name: String,
    /// File to store history
    pub history_file: String,
    /// List of commands to run
    pub commands: Vec<CommandConfig>,
    /// UID to send notifications and write files
    pub uid: Option<u32>,
    /// GID to send notifications and write files
    pub gid: Option<u32>,
    /// Output directory
    /// This is where the HTML file will be written
    pub output_dir: String,
    /// Notification configuration
    pub notifications: Option<NotificationConfig>,
}

impl Config {
    /// Load configuration from YAML files
    pub async fn load(yaml_file: &String) -> Result<Self, RondeError> {
        let file_contents = fs::read_to_string(yaml_file).await?;
        let config: Config = serde_yaml::from_str(&file_contents)?;
        config.check_unique_command_names()?;
        Ok(config)
    }

    /// Check that all the command names are unique
    pub fn check_unique_command_names(&self) -> Result<(), ConfigError> {
        let mut names = HashSet::new();
        for command in &self.commands {
            if names.contains(&command.name) {
                return Err(ConfigError::NotUniqueCommandName(command.name.clone()));
            }
            names.insert(command.name.clone());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_load() {
        let mut file = NamedTempFile::new().unwrap();
        write!(
            file,
            r#"---
output_dir: "/var/www/html"
history_file: "/var/lib/ronde/history"
name: "Ronde"
notifications:
  pushover:
    token: "token123"
    user: "user123"
commands:
  - name: "test"
    timeout: 10
    run: echo "test"
    uid: 1000
    gid: 1234
  - name: "ping localhost"
    run: ping -c 4 localhost
"#
        )
        .unwrap();
        let yaml_file = file.path().to_str().unwrap().to_string();
        let config = Config::load(&yaml_file).await.unwrap();
        assert_eq!(
            config,
            Config {
                notifications: Some(NotificationConfig {
                    pushover: Some(PushoverConfig {
                        user: "user123".to_string(),
                        token: "token123".to_string(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                name: "Ronde".to_string(),
                output_dir: "/var/www/html".to_string(),
                history_file: "/var/lib/ronde/history".to_string(),
                commands: vec![
                    CommandConfig {
                        name: "test".to_string(),
                        timeout: Timeout(10),
                        run: "echo \"test\"".to_string(),
                        uid: Some(1000),
                        gid: Some(1234),
                    },
                    CommandConfig {
                        name: "ping localhost".to_string(),
                        timeout: Timeout(60),
                        run: "ping -c 4 localhost".to_string(),
                        uid: None,
                        gid: None,
                    }
                ],
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_check_unique_command_names() {
        let config = Config {
            name: "Ronde".to_string(),
            notifications: None,
            output_dir: "/var/www/html".to_string(),
            history_file: "/var/lib/ronde/history".to_string(),
            commands: vec![
                CommandConfig {
                    name: "ping localhost".to_string(),
                    run: "ping -c 4 localhost".to_string(),
                    ..Default::default()
                },
                CommandConfig {
                    name: "ping localhost".to_string(),
                    run: "ping -c 4 localhost".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        assert!(config.check_unique_command_names().is_err());
    }
}
