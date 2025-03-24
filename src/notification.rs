use anyhow::Result;
use reqwest::{Client, Method};

pub async fn send_update_notification(topic: &str, message: &str) -> Result<()> {
    send_notification(topic, message, "Updates available", "arrow_up").await
}

pub async fn send_error_notifictaion(topic: &str, message: &str) -> Result<()> {
    send_notification(topic, message, "Error while checking for updates", "x").await
}

/// Sends a notification the the ntfy.sh servers containing the message and using
/// the provided topic.
async fn send_notification(topic: &str, message: &str, title: &str, icon_str: &str) -> Result<()> {
    Client::new()
        .request(Method::POST, format!("https://ntfy.sh/{topic}"))
        .body(message.to_string())
        .header("Title", title)
        .header("Tags", icon_str)
        .send()
        .await?;
    Ok(())
}
