use rusqlite::params;

use crate::error::SammyResult;
use crate::vault::UnlockedVault;

pub fn chunk_text(text: &str, size: usize, overlap: usize) -> Vec<String> {
    let normalized = text
        .replace("\r\n", "\n")
        .lines()
        .map(|l| l.trim_end())
        .collect::<Vec<_>>()
        .join("\n");
    let text = normalized.trim();
    if text.is_empty() {
        return vec![];
    }
    if text.len() <= size {
        return vec![text.to_string()];
    }
    let chars: Vec<char> = text.chars().collect();
    let mut chunks = Vec::new();
    let mut start = 0;
    while start < chars.len() {
        let mut end = (start + size).min(chars.len());
        if end < chars.len() {
            let window = &chars[start..end];
            if let Some(rel) = window.iter().rposition(|&c| c == '\n') {
                if rel > size / 3 {
                    end = start + rel + 1;
                }
            } else if let Some(rel) = window.iter().rposition(|&c| c == '.' || c == '!' || c == '?')
            {
                if rel > size / 3 {
                    end = start + rel + 1;
                }
            }
        }
        let piece: String = chars[start..end].iter().collect();
        let piece = piece.trim();
        if !piece.is_empty() {
            chunks.push(piece.to_string());
        }
        if end >= chars.len() {
            break;
        }
        start = end.saturating_sub(overlap);
        if start >= end {
            start = end;
        }
    }
    chunks
}

pub fn build_context(
    vault: &UnlockedVault,
    question: &str,
    doc_ids: Option<&[String]>,
    budget: usize,
) -> SammyResult<String> {
    let ids: Vec<String> = if let Some(ids) = doc_ids {
        ids.to_vec()
    } else {
        let mut stmt = vault
            .db
            .conn
            .prepare("SELECT id FROM sources WHERE pinned = 1")?;
        let rows = stmt.query_map([], |r| r.get(0))?;
        let mut v = Vec::new();
        for r in rows {
            v.push(r?);
        }
        if v.is_empty() {
            let mut stmt = vault.db.conn.prepare("SELECT id FROM sources")?;
            let rows = stmt.query_map([], |r| r.get(0))?;
            for r in rows {
                v.push(r?);
            }
        }
        v
    };

    if ids.is_empty() {
        return Ok(String::new());
    }

    let mut docs: Vec<(String, String, Vec<(i64, String)>)> = Vec::new();
    for id in &ids {
        let filename: String = vault.db.conn.query_row(
            "SELECT filename FROM sources WHERE id = ?1",
            params![id],
            |r| r.get(0),
        )?;
        let mut stmt = vault.db.conn.prepare(
            "SELECT chunk_index, text FROM source_chunks WHERE doc_id = ?1 ORDER BY chunk_index ASC",
        )?;
        let rows = stmt.query_map(params![id], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
        })?;
        let mut chunks = Vec::new();
        for r in rows {
            chunks.push(r?);
        }
        let full = chunks
            .iter()
            .map(|(_, t)| t.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");
        docs.push((filename, full, chunks));
    }

    let total: usize = docs.iter().map(|(_, t, _)| t.len()).sum();
    if total <= budget {
        let mut out = String::from(
            "User documents (full text — prefer these over pretrained guesses):\n\n",
        );
        for (i, (name, text, _)) in docs.iter().enumerate() {
            out.push_str(&format!("### [{i}] {name}\n{text}\n\n"));
        }
        return Ok(out);
    }

    // Large library or life-history doc: sequential chunks beat keyword guessing.
    if docs.len() == 1 || is_broad_question(question) {
        return Ok(pack_sequential(&docs, budget));
    }

    // Multiple docs: fair share per file, then sequential within each.
    let per_doc = (budget / docs.len()).max(2_000);
    let mut parts: Vec<(String, String)> = Vec::new();
    for (name, _full, chunks) in &docs {
        let mut used = 0usize;
        let mut body = String::new();
        for (_idx, text) in chunks {
            if used + text.len() > per_doc {
                break;
            }
            if !body.is_empty() {
                body.push_str("\n\n");
            }
            body.push_str(text);
            used += text.len();
        }
        if body.is_empty() && !chunks.is_empty() {
            let take: String = chunks[0].1.chars().take(per_doc).collect();
            body = take;
        }
        parts.push((name.clone(), body));
    }
    Ok(format_excerpts(&parts, "retrieved excerpts"))
}

fn is_broad_question(q: &str) -> bool {
    let q = q.to_lowercase();
    [
        "review",
        "summarize",
        "summary",
        "this document",
        "my history",
        "know about",
        "tell me about",
        "what do you know",
        "read this",
        "life history",
        "childhood",
        "about me",
        "my story",
        "through 6th",
        "sixth grade",
    ]
    .iter()
    .any(|p| q.contains(p))
}

fn pack_sequential(docs: &[(String, String, Vec<(i64, String)>)], budget: usize) -> String {
    let per = if docs.len() == 1 {
        budget
    } else {
        (budget / docs.len()).max(2_000)
    };
    let mut parts: Vec<(String, String)> = Vec::new();
    for (name, full, chunks) in docs {
        let mut used = 0usize;
        let mut body = String::new();
        for (_idx, text) in chunks {
            if used + text.len() > per {
                break;
            }
            if !body.is_empty() {
                body.push_str("\n\n");
            }
            body.push_str(text);
            used += text.len();
        }
        if body.is_empty() {
            body = full.chars().take(per).collect();
        }
        parts.push((name.clone(), body));
    }
    format_excerpts(
        &parts,
        if docs.len() == 1 {
            "document excerpts (read in order — prefer over pretrained guesses)"
        } else {
            "document excerpts"
        },
    )
}

fn format_excerpts(parts: &[(String, String)], label: &str) -> String {
    let mut out = format!("User documents ({label}):\n\n");
    for (i, (name, text)) in parts.iter().enumerate() {
        out.push_str(&format!("### [{i}] {name}\n{text}\n\n"));
    }
    out
}
