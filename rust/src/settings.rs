// caveman — settings.json reader/writer (replaces the inline `node -e` blocks
// in install.sh / uninstall.sh, so the installer never needs Node).
//
// JSONC-tolerant on read: strips // and /* */ comments and trailing commas
// before parsing. Writes plain JSON with 2-space indent + trailing newline
// (matching the old JSON.stringify(settings, null, 2) + "\n" behavior — which
// also dropped comments on write, so no fidelity is lost).

use serde_json::{json, Value};
use std::path::Path;

/// Strip JSONC comments and trailing commas, respecting string literals.
fn strip_jsonc(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    let n = bytes.len();
    let mut in_str = false;
    let mut escaped = false;

    while i < n {
        let c = bytes[i];
        if in_str {
            out.push(c as char);
            if escaped {
                escaped = false;
            } else if c == b'\\' {
                escaped = true;
            } else if c == b'"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        match c {
            b'"' => {
                in_str = true;
                out.push('"');
                i += 1;
            }
            b'/' if i + 1 < n && bytes[i + 1] == b'/' => {
                // line comment
                i += 2;
                while i < n && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if i + 1 < n && bytes[i + 1] == b'*' => {
                // block comment
                i += 2;
                while i + 1 < n && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i += 2;
            }
            _ => {
                out.push(c as char);
                i += 1;
            }
        }
    }

    // Drop trailing commas: `,` followed by optional whitespace then } or ].
    let mut cleaned = String::with_capacity(out.len());
    let ob = out.as_bytes();
    let mut j = 0;
    let m = ob.len();
    while j < m {
        if ob[j] == b',' {
            let mut k = j + 1;
            while k < m && (ob[k] as char).is_whitespace() {
                k += 1;
            }
            if k < m && (ob[k] == b'}' || ob[k] == b']') {
                // skip this comma
                j += 1;
                continue;
            }
        }
        cleaned.push(ob[j] as char);
        j += 1;
    }
    cleaned
}

/// Read settings.json, returning a JSON object Value. Returns `{}` if the file
/// is missing or unparseable (after comment stripping).
pub fn read_settings(path: &Path) -> Value {
    let raw = match std::fs::read_to_string(path) {
        Ok(r) => r,
        Err(_) => return json!({}),
    };
    let stripped = strip_jsonc(&raw);
    match serde_json::from_str::<Value>(&stripped) {
        Ok(v) if v.is_object() => v,
        _ => json!({}),
    }
}

pub fn write_settings(path: &Path, settings: &Value) -> std::io::Result<()> {
    let mut s = serde_json::to_string_pretty(settings)?;
    s.push('\n');
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(path, s)
}

/// True if settings.json has a non-null `statusLine`.
pub fn has_statusline(path: &Path) -> bool {
    has_statusline_value(&read_settings(path))
}

/// True if an already-loaded settings object has a non-null `statusLine`.
pub fn has_statusline_value(settings: &Value) -> bool {
    settings
        .get("statusLine")
        .map(|v| !v.is_null())
        .unwrap_or(false)
}

/// True if a hook entry's command string contains "caveman".
fn entry_is_caveman(entry: &Value) -> bool {
    entry
        .get("hooks")
        .and_then(|h| h.as_array())
        .map(|arr| {
            arr.iter().any(|h| {
                h.get("command")
                    .and_then(|c| c.as_str())
                    .map(|c| c.contains("caveman"))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

/// True if the given hook event array already has a caveman entry.
pub fn has_caveman_hook(settings: &Value, event: &str) -> bool {
    settings
        .get("hooks")
        .and_then(|h| h.get(event))
        .and_then(|e| e.as_array())
        .map(|arr| arr.iter().any(entry_is_caveman))
        .unwrap_or(false)
}

/// Append a caveman hook entry to `event` if one is not already present.
/// Returns true if it added an entry.
pub fn add_caveman_hook(
    settings: &mut Value,
    event: &str,
    command: &str,
    status_message: &str,
) -> bool {
    if has_caveman_hook(settings, event) {
        return false;
    }
    if !settings.get("hooks").map(|h| h.is_object()).unwrap_or(false) {
        settings["hooks"] = json!({});
    }
    let hooks = settings.get_mut("hooks").unwrap();
    if !hooks.get(event).map(|e| e.is_array()).unwrap_or(false) {
        hooks[event] = json!([]);
    }
    let arr = hooks.get_mut(event).unwrap().as_array_mut().unwrap();
    arr.push(json!({
        "hooks": [{
            "type": "command",
            "command": command,
            "timeout": 5,
            "statusMessage": status_message
        }]
    }));
    true
}

/// Remove caveman hook entries from the given events. Returns the count removed.
/// Drops empty event arrays and an empty `hooks` object, matching the old
/// uninstall behavior.
pub fn remove_caveman_hooks(settings: &mut Value, events: &[&str]) -> usize {
    let mut removed = 0;
    let hooks = match settings.get_mut("hooks").and_then(|h| h.as_object_mut()) {
        Some(h) => h,
        None => return 0,
    };
    for event in events {
        if let Some(arr) = hooks.get_mut(*event).and_then(|e| e.as_array_mut()) {
            let before = arr.len();
            arr.retain(|e| !entry_is_caveman(e));
            removed += before - arr.len();
            if arr.is_empty() {
                hooks.remove(*event);
            }
        }
    }
    let empty = hooks.is_empty();
    if empty {
        settings.as_object_mut().unwrap().remove("hooks");
    }
    removed
}

/// Current statusLine command string, if any.
pub fn statusline_command(settings: &Value) -> String {
    match settings.get("statusLine") {
        Some(Value::String(s)) => s.clone(),
        Some(v) => v
            .get("command")
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_line_and_block_comments_and_trailing_commas() {
        let src = r#"{
            // line comment
            "a": 1, /* block */
            "b": [1, 2,],
        }"#;
        let v: Value = serde_json::from_str(&strip_jsonc(src)).unwrap();
        assert_eq!(v["a"], 1);
        assert_eq!(v["b"], json!([1, 2]));
    }

    #[test]
    fn keeps_comment_like_text_inside_strings() {
        let src = r#"{"url": "http://x/y", "c": "a // b /* c */ d"}"#;
        let v: Value = serde_json::from_str(&strip_jsonc(src)).unwrap();
        assert_eq!(v["url"], "http://x/y");
        assert_eq!(v["c"], "a // b /* c */ d");
    }

    #[test]
    fn add_hook_is_idempotent() {
        let mut s = json!({});
        assert!(add_caveman_hook(&mut s, "SessionStart", "x caveman y", "msg"));
        assert!(!add_caveman_hook(&mut s, "SessionStart", "x caveman y", "msg"));
        assert!(has_caveman_hook(&s, "SessionStart"));
        assert_eq!(s["hooks"]["SessionStart"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn add_hook_preserves_unrelated_entries() {
        let mut s = json!({
            "hooks": { "SessionStart": [ { "hooks": [ { "type": "command", "command": "echo hi" } ] } ] }
        });
        add_caveman_hook(&mut s, "SessionStart", "caveman activate", "msg");
        let arr = s["hooks"]["SessionStart"].as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn remove_hooks_drops_only_caveman_and_empties() {
        let mut s = json!({
            "hooks": {
                "SessionStart": [
                    { "hooks": [ { "type": "command", "command": "echo hi" } ] },
                    { "hooks": [ { "type": "command", "command": "caveman activate" } ] }
                ],
                "UserPromptSubmit": [
                    { "hooks": [ { "type": "command", "command": "caveman mode-tracker" } ] }
                ]
            }
        });
        let removed = remove_caveman_hooks(&mut s, &["SessionStart", "UserPromptSubmit"]);
        assert_eq!(removed, 2);
        assert_eq!(s["hooks"]["SessionStart"].as_array().unwrap().len(), 1);
        // UserPromptSubmit emptied → key dropped
        assert!(s["hooks"].get("UserPromptSubmit").is_none());
    }
}
