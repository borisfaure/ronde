use crate::config::NotificationConfig;
use crate::history::History;

async fn send_notification(
    config: &NotificationConfig,
    message: String,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(ref pushover) = config.pushover {
        let client = reqwest::Client::new();
        let response = client
            .post("https://api.pushover.net/1/messages.json")
            .form(&[
                ("user", &pushover.user),
                ("token", &pushover.token),
                ("message", &message),
            ])
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(format!("Failed to send notification: {}", response.text().await?).into());
        }
    }
    Ok(())
}

pub async fn check_and_send_notifications(
    config: &NotificationConfig,
    history: &History,
) -> Result<(), Box<dyn std::error::Error>> {
    for command_history in &history.commands {
        if command_history.is_new_error() {
            let message = format!("Command '{}' failed", command_history.name);
            send_notification(config, message).await?;
        }
    }
    Ok(())
}
