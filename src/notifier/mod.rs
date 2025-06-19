use log::{error, info};
use reqwest::Client;
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct NotifierService {
    discord_webhook_url: String,
    user_address: String,
    client: Client,
}

#[derive(serde::Serialize)]
struct DiscordMessage {
    content: String,
}

impl NotifierService {
    pub fn new(discord_webhook_url: String, user_address: String) -> Self {
        Self {
            discord_webhook_url,
            user_address,
            client: Client::new(),
        }
    }

    pub async fn notify<T: Serialize>(&self, event: &str, data: &T) {
        let result = async {
            let data_json = serde_json::to_string_pretty(data)?;
            let explorer_link = format!("https://hypurrscan.io/address/{}", self.user_address);

            let message = format!(
                "**Event: {}**\n```json\n{}\n```\n🔗 [View on Hypurrscan]({})",
                event, data_json, explorer_link
            );

            let payload = DiscordMessage { content: message };

            let response = self
                .client
                .post(&self.discord_webhook_url)
                .json(&payload)
                .send()
                .await?;

            if !response.status().is_success() {
                let error_text = response.text().await?;
                return Err(format!("Discord webhook failed: {}", error_text).into());
            }

            info!("notification sent for event: {}", event);
            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        }
        .await;

        if let Err(e) = result {
            error!("notification failed for '{}': {}", event, e);
        }
    }

    pub async fn notify_text(&self, event: &str, message: &str) {
        let result = async {
            let explorer_link = format!("https://hypurrscan.io/address/{}", self.user_address);

            let content = format!(
                "**Event: {}**\n{}\n🔗 [View on Hypurrscan]({})",
                event, message, explorer_link
            );

            let payload = DiscordMessage { content };

            let response = self
                .client
                .post(&self.discord_webhook_url)
                .json(&payload)
                .send()
                .await?;

            if !response.status().is_success() {
                let error_text = response.text().await?;
                return Err(format!("Discord webhook failed: {}", error_text).into());
            }

            info!("text notification sent for event: {}", event);
            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        }
        .await;

        if let Err(e) = result {
            error!("text notification failed for '{}': {}", event, e);
        }
    }
}
