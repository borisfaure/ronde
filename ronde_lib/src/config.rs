use serde_derive::Deserialize;
use tokio::fs;

/// Timeout in seconds
#[derive(Debug, PartialEq, Deserialize)]
pub struct Timeout(pub u16);

impl Default for Timeout {
    fn default() -> Self {
        Timeout(60)
    }
}

#[derive(Debug, PartialEq, Deserialize)]
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

#[derive(Debug, PartialEq, Deserialize)]
/// Configuration
pub struct Config {
    /// File to store history
    pub history_file: String,
    /// List of commands to run
    pub commands: Vec<CommandConfig>,
    /// Output directory
    /// This is where the HTML file will be written
    pub output_dir: String,
}

/// Load configuration from YAML files
pub async fn load(yaml_file: &String) -> Result<Config, Box<dyn std::error::Error>> {
    let file_contents = fs::read_to_string(yaml_file).await?;
    let config: Config = serde_yaml::from_str(&file_contents)?;
    Ok(config)
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
        let config = load(&yaml_file).await.unwrap();
        assert_eq!(
            config,
            Config {
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
                ]
            }
        );
    }
}
