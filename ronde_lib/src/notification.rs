use crate::config::NotificationConfig;
use crate::history::{CommandHistoryEntry, History};
use thiserror::Error;

#[derive(Debug, Error)]
/// Error type for notifications
pub enum NotificationError {
    /// Reqwest Error
    #[error("Reqwest Error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    /// Error related to sending a notification with Pushover
    #[error("PushoverError: {0}")]
    PushoverError(String),
}

#[derive(Debug, PartialEq)]
/// The type of notification to send.
enum NotificationType {
    /// No notification to send.
    None,
    /// The command has failed. This is the first time it has failed in a row.
    Failure,
    /// The succeeded after a failure.
    BackFromFailure,
}

async fn send_notification(
    config: &NotificationConfig,
    command_name: &str,
    notification_type: NotificationType,
    last_run: Option<&CommandHistoryEntry>,
) -> Result<(), NotificationError> {
    if let Some(ref pushover) = config.pushover {
        let client = reqwest::Client::new();
        let mut title = match notification_type {
            NotificationType::Failure => format!("New Failure of {}", command_name),
            NotificationType::BackFromFailure => format!("Back from failure on {}", command_name),
            NotificationType::None => "None".to_string(),
        };
        let mut details: String = String::new();
        if notification_type == NotificationType::Failure {
            if let Some(last) = last_run {
                details = match last.result {
                    Ok(ref output) => format!(
                        "{}\n>>>STDERR\n{}\n>>>STDOUT\n{}",
                        last.command, &output.stderr, &output.stdout
                    ),
                    Err(ref e) => format!("{}\n{}", last.command, e),
                };
            }
        }
        // Truncate the message to 1024 characters.
        if details.len() > 1024 {
            details.drain(..1024).for_each(drop);
        };
        if title.len() > 255 {
            title.drain(..255).for_each(drop);
        };
        let one = "1".to_string();
        let mut form = vec![
            ("user", &pushover.user),
            ("token", &pushover.token),
            ("monospace", &one),
            ("message", &details),
            ("title", &title),
        ];
        if let Some(ref url) = pushover.url {
            form.push(("url", url));
        }
        let response = client
            .post("https://api.pushover.net/1/messages.json")
            .form(&form)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(NotificationError::PushoverError(format!(
                "Failed to send notification to pushover: {}",
                response.text().await?
            )));
        }
    }
    Ok(())
}

pub async fn check_and_send_notifications(
    config: &NotificationConfig,
    history: &History,
) -> Result<(), NotificationError> {
    for command_history in &history.commands {
        let ntype = if command_history.is_new_error() {
            NotificationType::Failure
        } else if command_history.is_back_from_failure() {
            NotificationType::BackFromFailure
        } else {
            NotificationType::None
        };
        if ntype != NotificationType::None {
            send_notification(
                config,
                &command_history.name,
                ntype,
                command_history.entries.last(),
            )
            .await?;
        }
    }
    Ok(())
}
