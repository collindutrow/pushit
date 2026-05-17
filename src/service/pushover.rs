use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::service::{Message, Service};

const ENDPOINT: &str = "https://api.pushover.net/1/messages.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushoverConfig {
    pub token: String,
    pub user_key: String,
}

impl Service for PushoverConfig {
    fn send(&self, msg: &Message) -> Result<()> {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .build()
            .into();

        let priority_str = msg.priority.map(|p| p.to_string());

        let mut form: Vec<(&str, &str)> = vec![
            ("token", self.token.as_str()),
            ("user", self.user_key.as_str()),
            ("message", msg.body.as_str()),
        ];
        if let Some(v) = msg.title.as_deref() {
            form.push(("title", v));
        }
        if let Some(v) = priority_str.as_deref() {
            form.push(("priority", v));
        }
        if let Some(v) = msg.sound.as_deref() {
            form.push(("sound", v));
        }
        if let Some(v) = msg.device.as_deref() {
            form.push(("device", v));
        }
        if let Some(v) = msg.url.as_deref() {
            form.push(("url", v));
        }
        if let Some(v) = msg.url_title.as_deref() {
            form.push(("url_title", v));
        }

        let mut response = agent.post(ENDPOINT).send_form(form)?;
        let status = response.status().as_u16();
        let body = response.body_mut().read_to_string()?;

        if (200..300).contains(&status) {
            return Ok(());
        }

        let msg = parse_pushover_error(&body)
            .unwrap_or_else(|| format!("pushover returned HTTP {status}: {body}"));
        Err(Error::Service(msg))
    }
}

fn parse_pushover_error(body: &str) -> Option<String> {
    #[derive(Deserialize)]
    struct PushoverError {
        errors: Option<Vec<String>>,
    }
    let parsed: PushoverError = serde_json::from_str(body).ok()?;
    let errors = parsed.errors?;
    if errors.is_empty() {
        None
    } else {
        Some(errors.join("; "))
    }
}
