use thiserror::Error;

#[derive(Error, Debug)]
pub enum RondeError {
    /// IO Error
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    /// SerdeYaml Error
    #[error("Serde YAML Error: {0}")]
    SerdeYamlError(#[from] serde_yaml::Error),
}
