use thiserror::Error;

#[derive(Error, Debug)]
pub enum RondeError {
    /// IO Error
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    /// SerdeYaml Error
    #[error("Serde YAML Error: {0}")]
    SerdeYamlError(#[from] serde_yaml::Error),
    /// SerdeJson Error
    #[error("Serde JSON Error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    /// Config Error
    #[error("Config Error: {0}")]
    ConfigError(#[from] crate::config::ConfigError),
}
