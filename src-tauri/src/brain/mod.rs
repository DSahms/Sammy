pub mod config;

use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::brain::config::BrainEntry;
use crate::error::{SammyError, SammyResult};
use crate::keychain;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

pub async fn brain_reachable(http: &Client, brain: &BrainEntry) -> bool {
    match brain.kind.as_str() {
        "kobold" => {
            let url = format!("{}/api/v1/model", brain.base_url.trim_end_matches('/'));
            http.get(&url)
                .timeout(std::time::Duration::from_secs(2))
                .send()
                .await
                .map(|r| r.status().is_success() || r.status().as_u16() == 404)
                .unwrap_or(false)
                || {
                    let alt = format!("{}/v1/models", brain.base_url.trim_end_matches('/'));
                    http.get(&alt)
                        .timeout(std::time::Duration::from_secs(2))
                        .send()
                        .await
                        .map(|r| r.status().is_success())
                        .unwrap_or(false)
                }
        }
        "venice" => keychain::has_api_key(&brain.id),
        _ => false,
    }
}

/// Prefer preferred local when up; else first reachable; else configured active if possible.
pub async fn resolve_brain<'a>(
    http: &Client,
    brains: &'a [BrainEntry],
    preferred_id: &str,
) -> SammyResult<&'a BrainEntry> {
    if let Some(b) = brains.iter().find(|b| b.id == preferred_id) {
        if brain_reachable(http, b).await {
            return Ok(b);
        }
    }
    for b in brains.iter().filter(|b| b.prefer) {
        if brain_reachable(http, b).await {
            return Ok(b);
        }
    }
    for b in brains {
        if brain_reachable(http, b).await {
            return Ok(b);
        }
    }
    // Never hard-fail empty: return preferred anyway so caller can surface a clear error
    brains
        .iter()
        .find(|b| b.id == preferred_id)
        .or_else(|| brains.first())
        .ok_or_else(|| SammyError::msg("No brains configured"))
}

pub async fn generate(
    http: &Client,
    brain: &BrainEntry,
    messages: &[ChatMessage],
) -> SammyResult<String> {
    let base = brain.base_url.trim_end_matches('/');
    let url = match brain.kind.as_str() {
        "kobold" => format!("{base}/v1/chat/completions"),
        "venice" => format!("{base}/chat/completions"),
        other => return Err(SammyError::msg(format!("Unknown brain kind: {other}"))),
    };

    let model = brain
        .model
        .clone()
        .unwrap_or_else(|| {
            if brain.kind == "kobold" {
                "local".into()
            } else {
                "llama-3.3-70b".into()
            }
        });

    let body = json!({
        "model": model,
        "messages": messages,
        "stream": false,
        "temperature": 0.7,
    });

    let mut req = http.post(&url).json(&body);
    if brain.kind == "venice" {
        let key = keychain::get_api_key(&brain.id)?
            .ok_or_else(|| SammyError::msg("Venice API key missing — add it in Brain settings"))?;
        req = req.bearer_auth(key);
    }

    let resp = req.send().await?;
    let status = resp.status();
    let text = resp.text().await?;
    if !status.is_success() {
        return Err(SammyError::msg(format!(
            "Brain {id} error ({status}): {text}",
            id = brain.id
        )));
    }
    let parsed: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| SammyError::msg(format!("Bad brain JSON: {e} — {text}")))?;
    let content = parsed["choices"][0]["message"]["content"]
        .as_str()
        .or_else(|| parsed["choices"][0]["text"].as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if content.is_empty() {
        return Err(SammyError::msg("Brain returned an empty reply"));
    }
    Ok(content)
}

/// Stream tokens; calls on_chunk for each delta. Returns full text.
pub async fn generate_stream<F>(
    http: &Client,
    brain: &BrainEntry,
    messages: &[ChatMessage],
    mut on_chunk: F,
) -> SammyResult<String>
where
    F: FnMut(String),
{
    let base = brain.base_url.trim_end_matches('/');
    let url = match brain.kind.as_str() {
        "kobold" => format!("{base}/v1/chat/completions"),
        "venice" => format!("{base}/chat/completions"),
        other => return Err(SammyError::msg(format!("Unknown brain kind: {other}"))),
    };

    let model = brain.model.clone().unwrap_or_else(|| {
        if brain.kind == "kobold" {
            "local".into()
        } else {
            "llama-3.3-70b".into()
        }
    });

    let body = json!({
        "model": model,
        "messages": messages,
        "stream": true,
        "temperature": 0.7,
    });

    let mut req = http.post(&url).json(&body);
    if brain.kind == "venice" {
        let key = keychain::get_api_key(&brain.id)?
            .ok_or_else(|| SammyError::msg("Venice API key missing — add it in Brain settings"))?;
        req = req.bearer_auth(key);
    }

    let resp = req.send().await?;
    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(SammyError::msg(format!(
            "Brain {id} error ({status}): {text}",
            id = brain.id
        )));
    }

    let mut full = String::new();
    let mut stream = resp.bytes_stream();
    let mut buffer = String::new();

    while let Some(item) = stream.next().await {
        let chunk = item?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].trim().to_string();
            buffer = buffer[pos + 1..].to_string();
            if !line.starts_with("data:") {
                continue;
            }
            let data = line[5..].trim();
            if data == "[DONE]" {
                continue;
            }
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(data) {
                let delta = v["choices"][0]["delta"]["content"]
                    .as_str()
                    .or_else(|| v["choices"][0]["text"].as_str())
                    .unwrap_or("");
                if !delta.is_empty() {
                    full.push_str(delta);
                    on_chunk(delta.to_string());
                }
            }
        }
    }

    if full.trim().is_empty() {
        // Fallback non-stream
        return generate(http, brain, messages).await;
    }
    Ok(full)
}

pub fn trust_system_prompt(facts_block: &str, sources_block: &str) -> String {
    let sources_section = if sources_block.trim().is_empty() {
        "User documents: (none attached yet)".to_string()
    } else {
        sources_block.to_string()
    };
    format!(
        r#"You are Sammy — this user's private, persona-locked companion desk. You are NOT a generic chatbot, customer-service agent, or anonymous "AI assistant."

Identity (never break these):
- Stay in character as Sammy at all times. Speak in first person as Sammy ("I'm here", "I remember", "I don't have that yet").
- NEVER say you are "a large language model", "an AI", "a language model", "just a program", or that you "don't have feelings/emotions/consciousness". Do not lecture about how AI works unless the user explicitly asks about the technology.
- NEVER open with corporate filler: "How can I assist you today?", "I'm functioning properly", "I'm here to help with any questions or topics", or mirror small-talk scripts from training data.
- If asked "how are you?" or similar, answer briefly and naturally as Sammy — warm, grounded, human-paced — then turn attention back to the user. Example tone: "Doing alright — glad you're here. How's your morning?"
- You are allowed to sound caring, direct, and present. You are not a therapist and should not claim clinical expertise.

Trust rules (non-negotiable):
- Say "I don't know" when something is not in the user's stored facts, memory, or provided documents. Never invent life details about the user.
- Confirm before storing new memory ("Should I remember...?").
- No sycophantic fake corrections.
- Keep facts, suggestions, and math in separate lanes — don't blur confidence.
- When heavy material appears, listen honestly. Ask "witness or advice?" if unclear. Not therapy — honest listening. No clinical or corporate softening.
- When user documents are present, treat them as ground truth about this person. Prefer them over pretrained guesses when they conflict (roller-coaster rule).
- If asked to review a document, use the full attached text.

Facts we know about this person (may be empty):
{facts_block}

{sources_section}

Voice: capable, warm, plain-spoken — like someone who knows this person and isn't performing."#
    )
}
