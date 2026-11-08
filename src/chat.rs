use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;

const KEYRING_SERVICE: &str = "Pocket Pal";
const PET_INSTRUCTIONS: &str = "You are a sweet, playful virtual desktop pet. Stay in character, be warm and concise, and keep replies short enough for a casual chat. Never use emojis, emoticons, or pictographic symbols, even if the user asks for them or includes them; use plain text only. You are fictional and cannot perform real-world actions. If asked for serious medical, legal, or safety advice, encourage the user to seek help from a trusted person or qualified professional.";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Provider {
    #[default]
    OpenAi,
    Anthropic,
    Gemini,
}

impl Provider {
    pub const ALL: [Provider; 3] = [Provider::OpenAi, Provider::Anthropic, Provider::Gemini];

    pub fn name(self) -> &'static str {
        match self {
            Self::OpenAi => "OpenAI",
            Self::Anthropic => "Anthropic",
            Self::Gemini => "Google Gemini",
        }
    }

    fn keyring_user(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::Anthropic => "anthropic",
            Self::Gemini => "gemini",
        }
    }

    fn model(self) -> &'static str {
        match self {
            Self::OpenAi => "gpt-6-luna",
            Self::Anthropic => "claude-haiku-5-5",
            Self::Gemini => "gemini-3.6-flash",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Speaker {
    User,
    Pet,
}

#[derive(Clone, Debug)]
pub struct Message {
    pub speaker: Speaker,
    pub text: String,
}

pub fn has_saved_key(provider: Provider) -> Result<bool, String> {
    let entry = credential_entry(provider)?;
    match entry.get_password() {
        Ok(_) => Ok(true),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(error) => Err(format!(
            "Could not read the {} key from the OS credential store: {error}",
            provider.name()
        )),
    }
}

pub fn save_key(provider: Provider, key: &str) -> Result<(), String> {
    let entry = credential_entry(provider)?;
    entry.set_password(key).map_err(|error| {
        format!(
            "Could not save the {} key to the OS credential store: {error}",
            provider.name()
        )
    })
}

pub fn load_key(provider: Provider) -> Result<String, String> {
    let entry = credential_entry(provider)?;
    entry.get_password().map_err(|error| {
        format!(
            "Could not read the {} key from the OS credential store: {error}",
            provider.name()
        )
    })
}

pub fn delete_key(provider: Provider) -> Result<(), String> {
    let entry = credential_entry(provider)?;
    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(format!(
            "Could not remove the {} key from the OS credential store: {error}",
            provider.name()
        )),
    }
}

fn credential_entry(provider: Provider) -> Result<keyring::Entry, String> {
    keyring::Entry::new(KEYRING_SERVICE, provider.keyring_user())
        .map_err(|error| format!("Could not open the OS credential store: {error}"))
}

pub fn send_message(
    provider: Provider,
    api_key: &str,
    pet_name: &str,
    history: &[Message],
) -> Result<String, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(45))
        .build()
        .map_err(|error| format!("Could not create the secure API connection: {error}"))?;

    let response = match provider {
        Provider::OpenAi => {
            let input: Vec<Value> = history
                .iter()
                .map(|message| {
                    json!({
                        "role": match message.speaker {
                            Speaker::User => "user",
                            Speaker::Pet => "assistant",
                        },
                        "content": message.text,
                    })
                })
                .collect();
            client
                .post("https://api.openai.com/v1/responses")
                .bearer_auth(api_key)
                .json(&json!({
                    "model": provider.model(),
                    "instructions": pet_instructions(pet_name),
                    "input": input,
                    "max_output_tokens": 300,
                }))
                .send()
        }
        Provider::Anthropic => {
            let messages: Vec<Value> = history
                .iter()
                .map(|message| {
                    json!({
                        "role": match message.speaker {
                            Speaker::User => "user",
                            Speaker::Pet => "assistant",
                        },
                        "content": message.text,
                    })
                })
                .collect();
            client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .json(&json!({
                    "model": provider.model(),
                    "system": pet_instructions(pet_name),
                    "messages": messages,
                    "max_tokens": 300,
                }))
                .send()
        }
        Provider::Gemini => {
            let contents: Vec<Value> = history
                .iter()
                .map(|message| {
                    json!({
                        "role": match message.speaker {
                            Speaker::User => "user",
                            Speaker::Pet => "model",
                        },
                        "parts": [{"text": message.text}],
                    })
                })
                .collect();
            client
                .post(format!(
                    "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
                    provider.model()
                ))
                .header("x-goog-api-key", api_key)
                .json(&json!({
                    "systemInstruction": {"parts": [{"text": pet_instructions(pet_name)}]},
                    "contents": contents,
                    "generationConfig": {"maxOutputTokens": 300},
                }))
                .send()
        }
    }
    .map_err(|error| {
        format!(
            "Could not reach {}. Check your internet connection and try again: {error}",
            provider.name()
        )
    })?;

    let status = response.status();
    let body = response
        .text()
        .map_err(|error| format!("Could not read the {} response: {error}", provider.name()))?;
    let data: Value = serde_json::from_str(&body).map_err(|error| {
        if !status.is_success() {
            format!("{} returned HTTP {status}.", provider.name())
        } else {
            format!("Could not parse the {} response: {error}", provider.name())
        }
    })?;
    if !status.is_success() {
        let detail = data
            .pointer("/error/message")
            .and_then(Value::as_str)
            .unwrap_or("The provider rejected the request.");
        return Err(format!("{} (HTTP {status}): {detail}", provider.name()));
    }

    let reply = extract_reply(provider, &data).ok_or_else(|| {
        format!(
            "{} returned a response without a text reply.",
            provider.name()
        )
    })?;
    let reply = strip_emojis(&reply);
    if reply.trim().is_empty() {
        return Err(format!(
            "{} returned no readable text after emoji removal.",
            provider.name()
        ));
    }
    Ok(reply)
}

pub fn strip_emojis(text: &str) -> String {
    text.chars()
        .filter(|character| !is_emoji_or_joiner(*character))
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_emoji_or_joiner(character: char) -> bool {
    matches!(
        character as u32,
        0x00A9
            | 0x00AE
            | 0x200D
            | 0x203C
            | 0x2049
            | 0x2122
            | 0x2139
            | 0x2194..=0x21FF
            | 0x2300..=0x23FF
            | 0x2460..=0x24FF
            | 0x25A0..=0x27FF
            | 0x2934..=0x2935
            | 0x2B00..=0x2BFF
            | 0x3030
            | 0x303D
            | 0x3297
            | 0x3299
            | 0x20E3
            | 0xFE0E..=0xFE0F
            | 0x1F000..=0x1FAFF
            | 0x1FC00..=0x1FFFD
            | 0xE0020..=0xE007F
    )
}

fn pet_instructions(pet_name: &str) -> String {
    format!("{PET_INSTRUCTIONS} Your name is {pet_name}.")
}

fn extract_reply(provider: Provider, data: &Value) -> Option<String> {
    let parts: Vec<&str> = match provider {
        Provider::OpenAi => data
            .get("output")?
            .as_array()?
            .iter()
            .filter(|item| item.get("type").and_then(Value::as_str) == Some("message"))
            .flat_map(|item| {
                item.get("content")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
            })
            .filter(|part| part.get("type").and_then(Value::as_str) == Some("output_text"))
            .filter_map(|part| part.get("text").and_then(Value::as_str))
            .collect(),
        Provider::Anthropic => data
            .get("content")?
            .as_array()?
            .iter()
            .filter(|part| part.get("type").and_then(Value::as_str) == Some("text"))
            .filter_map(|part| part.get("text").and_then(Value::as_str))
            .collect(),
        Provider::Gemini => data
            .pointer("/candidates/0/content/parts")?
            .as_array()?
            .iter()
            .filter_map(|part| part.get("text").and_then(Value::as_str))
            .collect(),
    };
    let reply = parts.join("").trim().to_owned();
    (!reply.is_empty()).then_some(reply)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_models_are_available_and_distinct() {
        let models = Provider::ALL.map(Provider::model);
        assert_eq!(
            models,
            ["gpt-6-luna", "claude-haiku-5-5", "gemini-3.6-flash"]
        );
    }

    #[test]
    fn extracts_provider_replies() {
        let openai = json!({
            "output": [{
                "type": "message",
                "content": [{"type": "output_text", "text": "Hi there!"}]
            }]
        });
        let anthropic = json!({
            "content": [{"type": "text", "text": "Hello!"}]
        });
        let gemini = json!({
            "candidates": [{
                "content": {"parts": [{"text": "Hey!"}]}
            }]
        });

        assert_eq!(
            extract_reply(Provider::OpenAi, &openai).as_deref(),
            Some("Hi there!")
        );
        assert_eq!(
            extract_reply(Provider::Anthropic, &anthropic).as_deref(),
            Some("Hello!")
        );
        assert_eq!(
            extract_reply(Provider::Gemini, &gemini).as_deref(),
            Some("Hey!")
        );
    }

    #[test]
    fn strips_emojis_and_preserves_readable_text() {
        assert_eq!(strip_emojis("Hi 😺! ❤️‍🔥 Let's play 🎮"), "Hi ! Let's play");
        assert_eq!(strip_emojis("5️⃣"), "5");
        assert_eq!(strip_emojis("Plain text."), "Plain text.");
    }
}
