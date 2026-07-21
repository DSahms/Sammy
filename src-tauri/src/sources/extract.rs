use std::path::Path;

use crate::error::{SammyError, SammyResult};

pub fn extract_text(path: &Path, raw: &[u8]) -> SammyResult<String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "txt" | "md" | "csv" => Ok(String::from_utf8_lossy(raw).to_string()),
        "html" | "htm" => Ok(strip_html(&String::from_utf8_lossy(raw))),
        "docx" => extract_docx(raw),
        "pdf" => extract_pdf(raw),
        "" => {
            // No extension — try UTF-8 text
            let s = String::from_utf8_lossy(raw).to_string();
            if s.chars().filter(|c| c.is_control() && *c != '\n' && *c != '\r' && *c != '\t').count()
                > s.len() / 20
            {
                Err(SammyError::msg("Unsupported file type"))
            } else {
                Ok(s)
            }
        }
        other => Err(SammyError::msg(format!(
            "Unsupported file type: .{other} (use .md .txt .csv .pdf .docx .html)"
        ))),
    }
}

fn strip_html(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut in_script = false;
    let lower = html.to_lowercase();
    let bytes: Vec<char> = html.chars().collect();
    let lbytes: Vec<char> = lower.chars().collect();
    let mut i = 0;
    while i < bytes.len() {
        if !in_tag && i + 7 < lbytes.len() && lbytes[i..].starts_with(&['<', 's', 'c', 'r', 'i', 'p', 't']) {
            in_script = true;
        }
        if in_script && i + 8 < lbytes.len() && lbytes[i..].starts_with(&['<', '/', 's', 'c', 'r', 'i', 'p', 't']) {
            in_script = false;
        }
        let c = bytes[i];
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
            out.push(' ');
        } else if !in_tag && !in_script {
            out.push(c);
        }
        i += 1;
    }
    collapse_ws(&out)
}

fn collapse_ws(s: &str) -> String {
    let mut out = String::new();
    let mut prev_space = false;
    for c in s.chars() {
        if c.is_whitespace() {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(c);
            prev_space = false;
        }
    }
    out.trim().to_string()
}

fn extract_docx(raw: &[u8]) -> SammyResult<String> {
    let cursor = std::io::Cursor::new(raw);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| SammyError::msg(format!("docx zip: {e}")))?;
    let mut file = archive
        .by_name("word/document.xml")
        .map_err(|_| SammyError::msg("Invalid .docx (missing document.xml)"))?;
    let mut xml = String::new();
    std::io::Read::read_to_string(&mut file, &mut xml)
        .map_err(|e| SammyError::msg(format!("docx read: {e}")))?;
    // Keep paragraph breaks: </w:p> → newline; strip other tags
    let with_breaks = xml.replace("</w:p>", "\n").replace("</w:tr>", "\n");
    Ok(strip_html(&with_breaks))
}

fn extract_pdf(raw: &[u8]) -> SammyResult<String> {
    let text = pdf_extract::extract_text_from_mem(raw)
        .map_err(|e| SammyError::msg(format!("PDF: {e}")))?;
    let t = text.trim().to_string();
    if t.is_empty() {
        return Err(SammyError::msg(
            "Could not read text from this PDF (scanned image?)",
        ));
    }
    Ok(t)
}
