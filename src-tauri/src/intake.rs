use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::{SammyError, SammyResult};
use crate::vault::UnlockedVault;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntakeQuestion {
    pub id: String,
    pub domain: String,
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntakeState {
    pub path: String,
    pub step_index: usize,
    pub completed: bool,
    pub current: Option<IntakeQuestion>,
    pub total: usize,
}

fn quick_questions() -> Vec<IntakeQuestion> {
    vec![
        IntakeQuestion {
            id: "q_name".into(),
            domain: "preferences".into(),
            prompt: "What should I call you?".into(),
        },
        IntakeQuestion {
            id: "q_people".into(),
            domain: "people".into(),
            prompt: "Who are the people that matter most right now? (names + how they relate)".into(),
        },
        IntakeQuestion {
            id: "q_dates".into(),
            domain: "dates".into(),
            prompt: "Any birthdays, anniversaries, or hard dates I should never miss?".into(),
        },
        IntakeQuestion {
            id: "q_speak".into(),
            domain: "preferences".into(),
            prompt: "How do you want me to talk to you — direct, soft, short, detailed?".into(),
        },
    ]
}

fn deep_questions() -> Vec<IntakeQuestion> {
    let mut q = quick_questions();
    q.extend([
        IntakeQuestion {
            id: "q_health".into(),
            domain: "health".into(),
            prompt: "Any optional health or routine facts you want me to know? (Skip if you'd rather not.)".into(),
        },
        IntakeQuestion {
            id: "q_triggers".into(),
            domain: "triggers".into(),
            prompt: "Anything I must never do or bring up carelessly? (User-stated only — you control this.)".into(),
        },
        IntakeQuestion {
            id: "q_work".into(),
            domain: "work".into(),
            prompt: "What's the shape of your work / obligations week — what should I respect?".into(),
        },
        IntakeQuestion {
            id: "q_why".into(),
            domain: "preferences".into(),
            prompt: "When you skip plans, is there a pattern I should learn to ask about gently?".into(),
        },
    ]);
    q
}

fn questions_for(path: &str) -> Vec<IntakeQuestion> {
    match path {
        "B" => quick_questions(),
        "C" => deep_questions(), // C includes B
        _ => Vec::new(),
    }
}

pub fn get_state(vault: &UnlockedVault) -> SammyResult<IntakeState> {
    let (path, step_index, completed, _answers): (String, i64, i64, String) =
        vault.db.conn.query_row(
            "SELECT path, step_index, completed, answers_json FROM intake_progress WHERE id = 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )?;
    let qs = questions_for(&path);
    let step = step_index as usize;
    let done = completed != 0;
    let current = if !done && step < qs.len() {
        Some(qs[step].clone())
    } else {
        None
    };
    Ok(IntakeState {
        path,
        step_index: step,
        completed: done,
        current,
        total: qs.len(),
    })
}

pub fn start(vault: &UnlockedVault, path: &str) -> SammyResult<IntakeState> {
    let path = path.to_uppercase();
    if !matches!(path.as_str(), "A" | "B" | "C") {
        return Err(SammyError::msg("Path must be A, B, or C"));
    }
    if path == "A" {
        vault.db.conn.execute(
            "UPDATE intake_progress SET path = 'A', step_index = 0, completed = 1, answers_json = '{}' WHERE id = 1",
            [],
        )?;
        return get_state(vault);
    }
    vault.db.conn.execute(
        "UPDATE intake_progress SET path = ?1, step_index = 0, completed = 0, answers_json = '{}' WHERE id = 1",
        params![path],
    )?;
    get_state(vault)
}

pub fn skip_to_chat(vault: &UnlockedVault) -> SammyResult<IntakeState> {
    start(vault, "A")
}

pub fn answer(vault: &UnlockedVault, text: &str) -> SammyResult<IntakeState> {
    let text = text.trim();
    if text.is_empty() {
        return Err(SammyError::msg("Empty answer"));
    }
    let (path, step_index, completed, answers_raw): (String, i64, i64, String) =
        vault.db.conn.query_row(
            "SELECT path, step_index, completed, answers_json FROM intake_progress WHERE id = 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )?;
    if completed != 0 {
        return get_state(vault);
    }
    let qs = questions_for(&path);
    let step = step_index as usize;
    if step >= qs.len() {
        vault.db.conn.execute(
            "UPDATE intake_progress SET completed = 1 WHERE id = 1",
            [],
        )?;
        return get_state(vault);
    }
    let q = &qs[step];
    let mut answers: Value = serde_json::from_str(&answers_raw).unwrap_or_else(|_| json!({}));
    answers[&q.id] = json!(text);

    // Confirm-before-store is UI-side; answering intake is explicit consent to store.
    let now = Utc::now().to_rfc3339();
    let fact_id = Uuid::new_v4().to_string();
    vault.db.conn.execute(
        "INSERT INTO facts (id, domain, label, value, source, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 'intake', ?5, ?5)",
        params![fact_id, q.domain, q.prompt, text, now],
    )?;

    let next = step + 1;
    let done = next >= qs.len();
    vault.db.conn.execute(
        "UPDATE intake_progress SET step_index = ?1, completed = ?2, answers_json = ?3 WHERE id = 1",
        params![next as i64, if done { 1 } else { 0 }, answers.to_string()],
    )?;
    get_state(vault)
}
