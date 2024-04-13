use serde_derive::Deserialize;
use snafu::prelude::*;
use std::collections::{HashMap, HashSet};
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
    /// Clears the entire environment map before running the command
    #[serde(default)]
    pub clear_env: bool,
    /// Environment variables to set
    #[serde(default)]
    pub env: Option<HashMap<String, String>>,
    /// Working directory
    pub cwd: Option<String>,
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
#[derive(Debug, Snafu)]
pub enum ConfigError {
    /// IO Error
    #[snafu(display("Unable to read {}: {}", path, source))]
    IoError {
        source: std::io::Error,
        path: String,
    },
    /// SerdeToml Error
    #[snafu(display("Serde TOML Error on {}: {}", path, source))]
    SerdeTomlError {
        source: toml::de::Error,
        path: String,
    },
    /// Command name is not unique
    #[snafu(display("Command name {} is not unique", cmd))]
    NotUniqueCommandName { cmd: String },
}

#[derive(Debug, Default, PartialEq, Deserialize)]
/// Configuration
pub struct Config {
    /// Name of the site to display
    pub name: String,
    /// File to store history
    pub history_file: String,
    /// UID to send notifications and write files
    pub uid: Option<u32>,
    /// GID to send notifications and write files
    pub gid: Option<u32>,
    /// Output directory
    /// This is where the HTML file will be written
    pub output_dir: String,
    /// Notification configuration
    pub notifications: Option<NotificationConfig>,
    /// List of commands to run
    pub commands: Vec<CommandConfig>,
}

impl Config {
    /// Load configuration from TOML files
    pub async fn load(toml_file: &str) -> Result<Self, ConfigError> {
        let file_contents = fs::read_to_string(toml_file).await.context(IoSnafu {
            path: toml_file.to_string(),
        })?;
        let config: Config = toml::from_str(&file_contents).context(SerdeTomlSnafu {
            path: toml_file.to_string(),
        })?;
        config.check_unique_command_names()?;
        Ok(config)
    }

    /// Check that all the command names are unique
    pub fn check_unique_command_names(&self) -> Result<(), ConfigError> {
        let mut names = HashSet::new();
        for command in &self.commands {
            if names.contains(&command.name) {
                return Err(ConfigError::NotUniqueCommandName {
                    cmd: command.name.clone(),
                });
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
            r#"
output_dir = "/var/www/html"
history_file= "/var/lib/ronde/history"
name = "Ronde"
[notifications.pushover]
    token = "token123"
    user = "user123"
[[commands]]
    name = "test"
    timeout = 10
    run = """echo "test""""
    uid = 1000
    gid = 1234
[[commands]]
    name = "ping localhost"
    run = "ping -c 4 localhost"
    clear_env = true
    env.KEY1 = "Value1"
    env.KEY2 = "Value2"
    cwd = "/tmp"
"#
        )
        .unwrap();
        let cfg_file = file.path().to_str().unwrap().to_string();
        let config = Config::load(&cfg_file).await.unwrap();
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
                        ..Default::default()
                    },
                    CommandConfig {
                        name: "ping localhost".to_string(),
                        timeout: Timeout(60),
                        run: "ping -c 4 localhost".to_string(),
                        clear_env: true,
                        env: Some(HashMap::from([
                            ("KEY1".to_string(), "Value1".to_string()),
                            ("KEY2".to_string(), "Value2".to_string())
                        ])),
                        cwd: Some("/tmp".to_string()),
                        ..Default::default()
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
