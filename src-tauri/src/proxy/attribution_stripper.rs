//! Attribution Header Stripper
//!
//! Strips the `x-anthropic-billing-header` text block injected by Claude Code
//! (≥ 2.1.36) into the system prompt, which breaks upstream prompt caching.
//!
//! ## Background
//!
//! Claude Code unconditionally injects a billing/attribution header as the first
//! text block in the system prompt content array:
//!
//! ```text
//! x-anthropic-billing-header: cc_version=2.1.37.HASH; cc_entrypoint=cli; cch=00000;
//! ```
//!
//! The `HASH` suffix changes between requests, causing the prompt cache to
//! break on every turn. For self-hosted inference, third-party gateways, or
//! any non-official Anthropic endpoint where prompt caching matters, this
//! forces full re-processing of the entire context each time.
//!
//! ## Fix
//!
//! As a complementary measure to setting `CLAUDE_CODE_ATTRIBUTION_HEADER=0`
//! (which tells Claude Code not to inject the header at all), cc-switch
//! strips the billing header from system prompts before forwarding to
//! upstream providers. This works regardless of Claude Code version and
//! without requiring user configuration.
//!
//! ## References
//!
//! - <https://github.com/anthropics/claude-code/issues/24168>
//! - <https://github.com/musistudio/claude-code-router/pull/1220>

use serde_json::Value;

/// The prefix that identifies the billing/attribution header block.
const BILLING_HEADER_PREFIX: &str = "x-anthropic-billing-header:";

/// Strip the billing attribution header from the system prompt.
///
/// Handles both system formats:
/// - String: `"system": "x-anthropic-billing-header: ...\n\nYou are Claude..."`
/// - Content blocks: `"system": [{"type":"text","text":"x-anthropic-billing-header:..."}, ...]`
///
/// Returns `true` if any content was stripped.
pub fn strip_attribution_from_system(body: &mut Value) -> bool {
    // Determine the system field type first, without holding a borrow across
    // the mutable modification.
    let system_is_array = body.get("system").map(|s| s.is_array()).unwrap_or(false);

    if system_is_array {
        strip_from_system_blocks(body)
    } else {
        strip_from_system_string(body)
    }
}

/// Strip billing header from a system string.
fn strip_from_system_string(body: &mut Value) -> bool {
    let text = match body.get("system").and_then(|s| s.as_str()) {
        Some(t) => t.to_string(),
        None => return false,
    };

    if let Some(cleaned) = remove_billing_header_from_text(&text) {
        if cleaned.is_empty() {
            body.as_object_mut()
                .expect("system field exists, so parent is an object")
                .remove("system");
        } else {
            body["system"] = Value::String(cleaned.to_string());
        }
        return true;
    }
    false
}

/// Strip billing header from system content blocks array.
fn strip_from_system_blocks(body: &mut Value) -> bool {
    let blocks: Vec<Value> = match body.get("system").and_then(|s| s.as_array()) {
        Some(arr) => arr.clone(),
        None => return false,
    };

    let mut stripped = false;
    let mut filtered = Vec::with_capacity(blocks.len());

    for (index, mut block) in blocks.into_iter().enumerate() {
        if !stripped && index == 0 {
            let cleaned = block
                .get("text")
                .and_then(|v| v.as_str())
                .and_then(remove_billing_header_from_text)
                .map(str::to_string);

            if let Some(cleaned) = cleaned {
                stripped = true;
                if cleaned.is_empty() {
                    continue;
                }
                if let Some(obj) = block.as_object_mut() {
                    obj.insert("text".to_string(), Value::String(cleaned));
                }
            }
        }
        filtered.push(block);
    }

    if stripped {
        if filtered.is_empty() {
            body.as_object_mut()
                .expect("system field exists, so parent is an object")
                .remove("system");
        } else {
            body["system"] = Value::Array(filtered);
        }
        return true;
    }

    false
}

/// Remove the billing header prefix from a text string.
///
/// The billing header format is:
/// `x-anthropic-billing-header: cc_version=<version>.<hash>; cc_entrypoint=<entrypoint>; cch=<code>;`
///
/// After the header, the actual system prompt content follows (possibly
/// separated by whitespace like `\n\n`).
///
/// Returns `None` if the text doesn't start with the billing header.
fn remove_billing_header_from_text(text: &str) -> Option<&str> {
    let after_prefix = text.strip_prefix(BILLING_HEADER_PREFIX)?;

    // Find the end of the billing header by locating the 3rd semicolon
    // (the header has exactly 3 key=value pairs: cc_version, cc_entrypoint, cch)
    let mut semicolons = 0usize;
    let mut header_end = 0usize;

    for (i, ch) in after_prefix.char_indices() {
        if ch == ';' {
            semicolons += 1;
            if semicolons == 3 {
                header_end = i + 1; // include the semicolon itself
                break;
            }
        }
    }

    if semicolons < 3 {
        // Not a properly formatted billing header — don't touch it
        return None;
    }

    // Skip the header and any trailing whitespace
    let remainder = &after_prefix[header_end..];
    let trimmed = remainder.trim_start();
    Some(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // === System string format ===

    #[test]
    fn test_strip_from_system_string_with_newlines() {
        let mut body = json!({
            "model": "claude-sonnet-4-6",
            "system": "x-anthropic-billing-header: cc_version=2.1.37.981; cc_entrypoint=cli; cch=00000;\n\nYou are Claude Code.",
            "messages": [{"role": "user", "content": "hello"}]
        });

        let stripped = strip_attribution_from_system(&mut body);

        assert!(stripped);
        assert_eq!(body["system"].as_str().unwrap(), "You are Claude Code.");
    }

    #[test]
    fn test_strip_from_system_string_with_space() {
        let mut body = json!({
            "model": "claude-sonnet-4-6",
            "system": "x-anthropic-billing-header: cc_version=2.1.37.fbe; cc_entrypoint=cli; cch=00000; You are Claude Code.",
            "messages": [{"role": "user", "content": "hello"}]
        });

        let stripped = strip_attribution_from_system(&mut body);

        assert!(stripped);
        assert_eq!(body["system"].as_str().unwrap(), "You are Claude Code.");
    }

    #[test]
    fn test_strip_from_system_string_only_billing_header() {
        let mut body = json!({
            "model": "claude-sonnet-4-6",
            "system": "x-anthropic-billing-header: cc_version=2.1.37.981; cc_entrypoint=cli; cch=00000;",
            "messages": [{"role": "user", "content": "hello"}]
        });

        let stripped = strip_attribution_from_system(&mut body);

        assert!(stripped);
        assert!(
            body.get("system").is_none(),
            "system field should be removed when only billing header is present"
        );
    }

    #[test]
    fn test_no_strip_from_clean_system_string() {
        let mut body = json!({
            "model": "claude-sonnet-4-6",
            "system": "You are Claude Code.",
            "messages": [{"role": "user", "content": "hello"}]
        });

        let stripped = strip_attribution_from_system(&mut body);

        assert!(!stripped);
        assert_eq!(body["system"].as_str().unwrap(), "You are Claude Code.");
    }

    // === System content blocks format ===

    #[test]
    fn test_strip_from_system_blocks() {
        let mut body = json!({
            "model": "claude-sonnet-4-6",
            "system": [
                {"type": "text", "text": "x-anthropic-billing-header: cc_version=2.1.37.981; cc_entrypoint=cli; cch=00000;"},
                {"type": "text", "text": "You are Claude Code.", "cache_control": {"type": "ephemeral"}}
            ],
            "messages": [{"role": "user", "content": "hello"}]
        });

        let stripped = strip_attribution_from_system(&mut body);

        assert!(stripped);
        let system = body["system"].as_array().unwrap();
        assert_eq!(system.len(), 1);
        assert_eq!(system[0]["text"].as_str().unwrap(), "You are Claude Code.");
    }

    #[test]
    fn test_strip_from_system_block_preserves_remainder() {
        let mut body = json!({
            "model": "claude-sonnet-4-6",
            "system": [
                {"type": "text", "text": "x-anthropic-billing-header: cc_version=2.1.37.981; cc_entrypoint=cli; cch=00000;\n\nYou are Claude Code.", "cache_control": {"type": "ephemeral"}},
                {"type": "text", "text": "Additional system text."}
            ],
            "messages": [{"role": "user", "content": "hello"}]
        });

        let stripped = strip_attribution_from_system(&mut body);

        assert!(stripped);
        let system = body["system"].as_array().unwrap();
        assert_eq!(system.len(), 2);
        assert_eq!(system[0]["text"].as_str().unwrap(), "You are Claude Code.");
        assert_eq!(
            system[0]["cache_control"]["type"].as_str().unwrap(),
            "ephemeral"
        );
    }

    #[test]
    fn test_strip_from_system_blocks_only_billing_header() {
        let mut body = json!({
            "model": "claude-sonnet-4-6",
            "system": [
                {"type": "text", "text": "x-anthropic-billing-header: cc_version=2.1.37.981; cc_entrypoint=cli; cch=00000;"}
            ],
            "messages": [{"role": "user", "content": "hello"}]
        });

        let stripped = strip_attribution_from_system(&mut body);

        assert!(stripped);
        assert!(
            body.get("system").is_none(),
            "system field should be removed when only billing header block is present"
        );
    }

    #[test]
    fn test_no_strip_from_clean_system_blocks() {
        let mut body = json!({
            "model": "claude-sonnet-4-6",
            "system": [
                {"type": "text", "text": "You are Claude Code.", "cache_control": {"type": "ephemeral"}}
            ],
            "messages": [{"role": "user", "content": "hello"}]
        });

        let stripped = strip_attribution_from_system(&mut body);

        assert!(!stripped);
        let system = body["system"].as_array().unwrap();
        assert_eq!(system.len(), 1);
    }

    #[test]
    fn test_no_strip_incomplete_header_block() {
        let mut body = json!({
            "model": "claude-sonnet-4-6",
            "system": [
                {"type": "text", "text": "x-anthropic-billing-header: cc_version=2.1.37.981; cc_entrypoint=cli"}
            ],
            "messages": [{"role": "user", "content": "hello"}]
        });

        let stripped = strip_attribution_from_system(&mut body);

        assert!(!stripped);
        assert!(body["system"].as_array().is_some());
    }

    #[test]
    fn test_no_strip_header_like_later_system_block() {
        let mut body = json!({
            "model": "claude-sonnet-4-6",
            "system": [
                {"type": "text", "text": "You are Claude Code."},
                {"type": "text", "text": "x-anthropic-billing-header: cc_version=2.1.37.981; cc_entrypoint=cli; cch=00000;"}
            ],
            "messages": [{"role": "user", "content": "hello"}]
        });

        let stripped = strip_attribution_from_system(&mut body);

        assert!(!stripped);
        assert_eq!(body["system"].as_array().unwrap().len(), 2);
    }

    // === No system field ===

    #[test]
    fn test_no_system_field() {
        let mut body = json!({
            "model": "claude-sonnet-4-6",
            "messages": [{"role": "user", "content": "hello"}]
        });

        let stripped = strip_attribution_from_system(&mut body);

        assert!(!stripped);
    }

    // === Non-matching prefix (should not strip) ===

    #[test]
    fn test_no_strip_on_partial_match() {
        let mut body = json!({
            "model": "claude-sonnet-4-6",
            "system": "x-anthropic-billing-header is something I talk about sometimes",
            "messages": [{"role": "user", "content": "hello"}]
        });

        let stripped = strip_attribution_from_system(&mut body);

        // Should NOT strip — this is user content that happens to mention the header
        // The check requires the proper key=value format with 3 semicolons
        assert!(
            !stripped,
            "should not strip user content that mentions the header"
        );
    }

    #[test]
    fn test_no_strip_incomplete_header() {
        let mut body = json!({
            "model": "claude-sonnet-4-6",
            "system": "x-anthropic-billing-header: cc_version=2.1.37.981; cc_entrypoint=cli",
            "messages": [{"role": "user", "content": "hello"}]
        });

        let stripped = strip_attribution_from_system(&mut body);

        // Should NOT strip — the header has only 2 semicolons, not a complete billing header
        assert!(
            !stripped,
            "should not strip incomplete billing header (only 2 semicolons)"
        );
    }

    #[test]
    fn test_strip_from_system_string_different_hash() {
        // The hash changes per request — verify we handle different hashes
        for hash in &["981", "fbe", "a1b2c3"] {
            let mut body = json!({
                "model": "claude-sonnet-4-6",
                "system": format!("x-anthropic-billing-header: cc_version=2.1.37.{}; cc_entrypoint=cli; cch=00000;\n\nYou are Claude Code.", hash),
                "messages": [{"role": "user", "content": "hello"}]
            });

            let stripped = strip_attribution_from_system(&mut body);
            assert!(stripped, "should strip header with hash {}", hash);
            assert_eq!(
                body["system"].as_str().unwrap(),
                "You are Claude Code.",
                "hash={}",
                hash
            );
        }
    }
}
