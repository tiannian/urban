use anyhow::Result;
use reqwest::Client;
use serde::Serialize;

const TELEGRAM_API_BASE: &str = "https://api.telegram.org";

/// Client for sending messages via Telegram Bot API.
pub struct TelegramBot {
    client: Client,
    api_key: String,
    chat_id: String,
}

#[derive(Serialize)]
struct SendMessageRequest {
    chat_id: String,
    text: String,
}

impl TelegramBot {
    /// Creates a new `TelegramBot` with the given API key and chat ID.
    pub fn new(api_key: String, chat_id: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            chat_id,
        }
    }

    /// Sends a text message to the configured chat.
    pub async fn push_message(&self, text: &str) -> Result<()> {
        let url = format!("{}/bot{}/sendMessage", TELEGRAM_API_BASE, self.api_key);
        let body = SendMessageRequest {
            chat_id: self.chat_id.clone(),
            text: text.to_string(),
        };
        self.client
            .post(&url)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}
