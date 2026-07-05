use serde::Serialize;
use serde_json::Value;
use std::{error::Error, fs, path::PathBuf};

const SESSION_KIND: &str = "omp-session-compaction-window";
const PARAGRAPH: &str = " ¶\n";

#[derive(Serialize)]
struct ExportMetadata {
    kind: &'static str,
    source_session: String,
    compaction_index: usize,
    start_line: usize,
    end_line: usize,
    event_count: usize,
    message_count: usize,
    user_messages: usize,
    assistant_messages: usize,
    tool_results: usize,
    serialized_chars: usize,
    eventized_chars: usize,
    eventized_turns: usize,
    source_json_chars: usize,
    tokens_before: Option<u64>,
    compaction_timestamp: Option<String>,
    note: &'static str,
}

pub fn run_export_session(args: &[String]) -> Result<(), Box<dyn Error>> {
    let session = required_arg(args, "--session")?;
    let out = PathBuf::from(required_arg(args, "--out")?);
    let compaction_index = required_arg(args, "--compaction-index")?.parse::<usize>()?;

    let text = fs::read_to_string(session)?;
    let mut events = Vec::new();
    for (line_index, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(line)
            .map_err(|err| format!("failed to parse {} line {}: {err}", session, line_index + 1))?;
        events.push((line_index + 1, line.len(), value));
    }

    let compactions: Vec<usize> = events
        .iter()
        .enumerate()
        .filter_map(|(index, (_, _, event))| {
            (event_type(event) == Some("compaction")).then_some(index)
        })
        .collect();
    let compaction_event_index = *compactions
        .get(compaction_index)
        .ok_or_else(|| format!("compaction index {compaction_index} not found"))?;
    let start_event_index = compaction_index
        .checked_sub(1)
        .and_then(|previous| compactions.get(previous).map(|index| index + 1))
        .unwrap_or(0);

    let window = &events[start_event_index..compaction_event_index];
    let compaction = &events[compaction_event_index].2;
    let mut serialized = String::new();
    let mut eventized = String::new();
    let mut eventized_turns = 0usize;
    let mut message_count = 0usize;
    let mut user_messages = 0usize;
    let mut assistant_messages = 0usize;
    let mut tool_results = 0usize;
    let source_json_chars = window.iter().map(|(_, len, _)| *len).sum::<usize>();

    for (_, _, event) in window {
        if event_type(event) != Some("message") {
            continue;
        }
        let Some(message) = event.get("message") else {
            continue;
        };
        message_count += 1;
        match message.get("role").and_then(Value::as_str) {
            Some("user") => {
                user_messages += 1;
                push_section(
                    &mut serialized,
                    "# User",
                    &content_to_text(message.get("content")),
                );
                eventized_turns += push_dialogue_turn(
                    &mut eventized,
                    "U",
                    &dialogue_content_to_text(message.get("content")),
                );
            }
            Some("assistant") => {
                assistant_messages += 1;
                push_section(
                    &mut serialized,
                    "# Assistant",
                    &assistant_content_to_text(message.get("content")),
                );
                eventized_turns += push_dialogue_turn(
                    &mut eventized,
                    "A",
                    &assistant_dialogue_content_to_text(message.get("content")),
                );
            }
            Some("toolResult") => {
                tool_results += 1;
                push_tool_result(&mut serialized, message);
            }
            Some(other) => {
                push_section(
                    &mut serialized,
                    &format!("# {other}"),
                    &content_to_text(message.get("content")),
                );
            }
            None => {}
        }
    }

    fs::create_dir_all(&out)?;
    let serialized = redact_public_fixture(&normalize_archive_text(&serialized));
    let eventized = redact_public_fixture(&normalize_archive_text(&eventized));
    fs::write(out.join("serialized.txt"), serialized.as_bytes())?;
    fs::write(out.join("eventized.txt"), eventized.as_bytes())?;
    let source_session = PathBuf::from(session)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("session.jsonl")
        .to_string();
    let metadata = ExportMetadata {
        kind: SESSION_KIND,
        source_session,
        compaction_index,
        start_line: events[start_event_index].0,
        end_line: events[compaction_event_index].0,
        event_count: window.len(),
        message_count,
        user_messages,
        assistant_messages,
        tool_results,
        serialized_chars: serialized.chars().count(),
        eventized_chars: eventized.chars().count(),
        eventized_turns,
        source_json_chars,
        tokens_before: compaction.get("tokensBefore").and_then(Value::as_u64),
        compaction_timestamp: compaction
            .get("timestamp")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        note: "Real OMP session compaction window; previous compressed archive is not folded into this source fixture.",
    };
    fs::write(
        out.join("metadata.json"),
        format!("{}\n", serde_json::to_string_pretty(&metadata)?),
    )?;

    println!(
        "exported {} messages from lines {}-{} to {} ({} chars)",
        message_count,
        metadata.start_line,
        metadata.end_line,
        out.display(),
        metadata.serialized_chars
    );
    Ok(())
}

fn required_arg<'a>(args: &'a [String], name: &str) -> Result<&'a str, Box<dyn Error>> {
    args.windows(2)
        .find_map(|window| (window[0] == name).then_some(window[1].as_str()))
        .ok_or_else(|| format!("missing required argument {name}").into())
}

fn redact_public_fixture(text: &str) -> String {
    let mut redacted = text.replace("/home/cagedbird", "~");
    for key in [
        "OPENAI_API_KEY",
        "ANTHROPIC_API_KEY",
        "GEMINI_API_KEY",
        "API_KEY",
        "TOKEN",
    ] {
        redacted = redact_assignment(&redact_colon_value(&redacted, key), key);
    }
    redact_sk_tokens(&redacted)
}

fn redact_assignment(text: &str, key: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for line in text.lines() {
        if let Some(index) = line.find(key) {
            let before = &line[..index + key.len()];
            let after = &line[index + key.len()..];
            if let Some(rest) = after.strip_prefix('=') {
                let suffix = rest
                    .find(char::is_whitespace)
                    .map(|suffix_index| &rest[suffix_index..])
                    .unwrap_or("");
                out.push_str(before);
                out.push_str("=<redacted>");
                out.push_str(suffix);
            } else {
                out.push_str(line);
            }
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    out.trim_end().to_string()
}

fn redact_colon_value(text: &str, key: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for line in text.lines() {
        out.push_str(&redact_colon_value_in_line(line, key));
        out.push('\n');
    }
    out.trim_end().to_string()
}

fn redact_colon_value_in_line(line: &str, key: &str) -> String {
    let Some(key_index) = line.find(key) else {
        return line.to_string();
    };
    let after_key = &line[key_index + key.len()..];
    let Some(colon_offset) = after_key.find(':') else {
        return line.to_string();
    };
    let value_start_search = key_index + key.len() + colon_offset + 1;
    let rest = &line[value_start_search..];
    let quote_offset = rest
        .char_indices()
        .find_map(|(offset, ch)| match ch {
            ' ' | '\t' => None,
            '\'' | '"' => Some(offset),
            _ => Some(usize::MAX),
        })
        .filter(|offset| *offset != usize::MAX);
    let Some(quote_offset) = quote_offset else {
        return line.to_string();
    };
    let quote_index = value_start_search + quote_offset;
    let quote = line.as_bytes()[quote_index] as char;
    let after_quote = &line[quote_index + 1..];
    let Some(end_quote_offset) = after_quote.find(quote) else {
        return line.to_string();
    };
    let end_quote_index = quote_index + 1 + end_quote_offset;
    format!(
        "{}{}<redacted>{}",
        &line[..quote_index + 1],
        "",
        &line[end_quote_index..]
    )
}

fn redact_sk_tokens(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.char_indices().peekable();
    while let Some((index, ch)) = chars.next() {
        if ch == 's' && text[index..].starts_with("sk-") {
            let start = index;
            let mut end = index + 3;
            while let Some((next_index, next_ch)) = chars.peek().copied() {
                if next_ch.is_ascii_alphanumeric() || next_ch == '_' || next_ch == '-' {
                    end = next_index + next_ch.len_utf8();
                    chars.next();
                } else {
                    break;
                }
            }
            if end.saturating_sub(start) > 10 {
                out.push_str("sk-<redacted>");
            } else {
                out.push_str(&text[start..end]);
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn event_type(event: &Value) -> Option<&str> {
    event.get("type").and_then(Value::as_str)
}

fn push_section(out: &mut String, heading: &str, body: &str) {
    if body.trim().is_empty() {
        return;
    }
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(heading);
    out.push_str(PARAGRAPH);
    out.push_str(body.trim());
    out.push('\n');
}

fn push_tool_result(out: &mut String, message: &Value) {
    let call_id = message
        .get("toolCallId")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let body = content_to_text(message.get("content"));
    if body.trim().is_empty() {
        return;
    }
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str("# Tool result ");
    out.push_str(call_id);
    out.push_str(PARAGRAPH);
    out.push_str("<out>\n");
    out.push_str(body.trim());
    out.push_str("\n</out>\n");
}

fn assistant_content_to_text(content: Option<&Value>) -> String {
    let Some(Value::Array(parts)) = content else {
        return scalar_content_to_text(content);
    };
    let mut blocks = Vec::new();
    for part in parts {
        match part.get("type").and_then(Value::as_str) {
            Some("thinking") => {
                if let Some(text) = part.get("thinking").and_then(Value::as_str) {
                    blocks.push(format!("[thinking]\n{}\n[/thinking]", text.trim()));
                }
            }
            Some("text") => {
                if let Some(text) = part.get("text").and_then(Value::as_str) {
                    blocks.push(text.trim().to_string());
                }
            }
            Some("toolCall") => blocks.push(tool_call_to_text(part)),
            Some(kind) => blocks.push(format!("[unsupported assistant content: {kind}]")),
            None => blocks.push(part.to_string()),
        }
    }
    blocks
        .into_iter()
        .filter(|block| !block.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn push_dialogue_turn(out: &mut String, speaker: &str, body: &str) -> usize {
    let body = compact_dialogue_body(body);
    if body.is_empty() {
        return 0;
    }
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(speaker);
    out.push_str(": ");
    out.push_str(&body);
    out.push('\n');
    1
}

fn assistant_dialogue_content_to_text(content: Option<&Value>) -> String {
    let Some(Value::Array(parts)) = content else {
        return dialogue_text(&scalar_content_to_text(content));
    };
    parts
        .iter()
        .filter_map(|part| match part.get("type").and_then(Value::as_str) {
            Some("text") => part.get("text").and_then(Value::as_str).map(dialogue_text),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn dialogue_content_to_text(content: Option<&Value>) -> String {
    let Some(Value::Array(parts)) = content else {
        return dialogue_text(&scalar_content_to_text(content));
    };
    parts
        .iter()
        .filter_map(|part| match part.get("type").and_then(Value::as_str) {
            Some("text") => part.get("text").and_then(Value::as_str).map(dialogue_text),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn dialogue_text(text: &str) -> String {
    strip_code_blocks(text)
        .lines()
        .filter_map(summarize_dialogue_line)
        .collect::<Vec<_>>()
        .join(" ")
}

fn strip_code_blocks(text: &str) -> String {
    let mut out = String::new();
    let mut in_code = false;
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            if in_code {
                in_code = false;
            } else {
                in_code = true;
            }
            continue;
        }
        if in_code {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }

    out
}

fn summarize_dialogue_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty()
        || trimmed == "<out>"
        || trimmed == "</out>"
        || trimmed.starts_with("[shaken ")
        || trimmed.starts_with("[raw output:")
    {
        return None;
    }
    if trimmed.len() > 500 && looks_machine_generated(trimmed) {
        return None;
    }
    Some(strip_markdown_inline(&strip_markdown_prefix(trimmed)))
}

fn strip_markdown_prefix(line: &str) -> String {
    let mut text = line.trim_start();
    while let Some(rest) = text.strip_prefix('#') {
        text = rest.trim_start();
    }
    if let Some(rest) = text.strip_prefix('>') {
        text = rest.trim_start();
    }
    for marker in ["- ", "* ", "+ "] {
        if let Some(rest) = text.strip_prefix(marker) {
            return rest.trim_start().to_string();
        }
    }
    let bytes = text.as_bytes();
    if bytes.len() > 2 && bytes[0].is_ascii_digit() {
        if let Some(dot_index) = text.find('.') {
            if text[..dot_index].bytes().all(|byte| byte.is_ascii_digit()) {
                return text[dot_index + 1..].trim_start().to_string();
            }
        }
    }
    text.to_string()
}

fn strip_markdown_inline(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '*' | '`' => {}
            '[' => {
                let mut label = String::new();
                let mut found_close = false;
                while let Some(next) = chars.next() {
                    if next == ']' {
                        found_close = true;
                        break;
                    }
                    label.push(next);
                }
                if found_close && chars.peek() == Some(&'(') {
                    chars.next();
                    let mut url = String::new();
                    for next in chars.by_ref() {
                        if next == ')' {
                            break;
                        }
                        url.push(next);
                    }
                    out.push_str(&label);
                    if !url.is_empty() {
                        out.push(' ');
                        out.push_str(&url);
                    }
                } else {
                    out.push('[');
                    out.push_str(&label);
                }
            }
            '|' => out.push(' '),
            _ => out.push(ch),
        }
    }
    out
}

fn compact_dialogue_body(body: &str) -> String {
    let mut out = String::new();
    let mut pending_space = false;
    for c in body.chars() {
        if c.is_whitespace() {
            pending_space = true;
            continue;
        }
        if pending_space && !out.is_empty() {
            out.push(' ');
        }
        pending_space = false;
        out.push(c);
    }
    out
}

fn looks_machine_generated(line: &str) -> bool {
    line.starts_with('{')
        || line.starts_with('[')
        || line.contains("\\n")
        || line.contains("\"type\":")
        || line.contains("function ")
        || line.contains("const ")
        || line.contains("=>")
}

fn content_to_text(content: Option<&Value>) -> String {
    let Some(Value::Array(parts)) = content else {
        return scalar_content_to_text(content);
    };
    parts
        .iter()
        .filter_map(|part| match part.get("type").and_then(Value::as_str) {
            Some("text") => part
                .get("text")
                .and_then(Value::as_str)
                .map(ToString::to_string),
            Some("thinking") => part
                .get("thinking")
                .and_then(Value::as_str)
                .map(ToString::to_string),
            Some("toolCall") => Some(tool_call_to_text(part)),
            Some(kind) => Some(format!("[unsupported content: {kind}]")),
            None => Some(part.to_string()),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn scalar_content_to_text(content: Option<&Value>) -> String {
    match content {
        Some(Value::String(text)) => text.to_string(),
        Some(Value::Null) | None => String::new(),
        Some(value) => value.to_string(),
    }
}

fn tool_call_to_text(part: &Value) -> String {
    let id = part.get("id").and_then(Value::as_str).unwrap_or("unknown");
    let name = part.get("name").and_then(Value::as_str).unwrap_or("tool");
    let arguments = part
        .get("arguments")
        .map(|value| serde_json::to_string(value).unwrap_or_else(|_| value.to_string()))
        .unwrap_or_else(|| "{}".to_string());
    format!("# Tool call {name} {id}\n{arguments}")
}

fn normalize_archive_text(text: &str) -> String {
    text.lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}
