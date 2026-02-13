//! Problem naming pipeline for screenshots.
//!
//! Entry point: [`suggest_problem_name`] — async, returns a filesystem-safe name.
//! Future steps (not implemented yet): decode image → OCR (ocrs) → LLM (Ollama)
//! with kebab-case summarization; on any failure or missing model, fall back to
//! a timestamp-based name so the app never blocks or crashes.

use chrono::Utc;

/// Fallback name when OCR/LLM are unavailable or fail. Filesystem-safe, no I/O.
/// Format: `problem-YYYYMMDD-HHMMSS` (no colons for portability).
fn problem_name_timestamp() -> String {
    Utc::now().format("problem-%Y%m%d-%H%M%S").to_string()
}

/// Suggests a problem name from screenshot image data.
/// Today: returns timestamp fallback. Later: OCR → LLM (Ollama) → fallback on failure.
/// Caller can await this without blocking; future OCR/Ollama work will be async here.
pub(crate) async fn suggest_problem_name(image_base64: String) -> String {
    // Phase 1 (future): decode base64 → image buffer
    // Phase 2 (future): ocrs::ocr(...) on image → extracted text
    // Phase 3 (future): call Ollama to summarize text → kebab-case, max 5 words
    // On any error or missing model: fall back to timestamp
    let _ = image_base64; // use when OCR is implemented
    problem_name_timestamp()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggest_problem_name_returns_timestamp_format() {
        let name = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(suggest_problem_name(String::new()));
        assert!(
            name.starts_with("problem-"),
            "expected prefix 'problem-', got '{}'",
            name
        );
        // Format: problem-YYYYMMDD-HHMMSS (no colons for portability)
        assert!(
            name.len() >= 22,
            "expected at least 'problem-YYYYMMDD-HHMMSS' length"
        );
        assert!(
            name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'),
            "name must be filesystem-safe (alphanumeric and hyphen only), got '{}'",
            name
        );
    }
}
