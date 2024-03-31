use thiserror::Error;

#[derive(Error, Debug)]
pub enum RondeError {
    /// Config Error
    #[error("Config Error: {0}")]
    ConfigError(#[from] crate::config::ConfigError),
    /// Html Error
    #[error("Html Error: {0}")]
    HtmlError(#[from] crate::html::HtmlError),
    /// Notification Error
    #[error("Notification Error: {0}")]
    NotificationError(#[from] crate::notification::NotificationError),
    /// History Error
    #[error("History Error: {0}")]
    HistoryError(#[from] crate::history::HistoryError),
}
