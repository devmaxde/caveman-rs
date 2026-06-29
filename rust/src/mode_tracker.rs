// caveman — UserPromptSubmit hook (port of caveman-mode-tracker.js).
// Inspects user input for /caveman commands + natural-language triggers and
// writes the active mode to the flag file. Emits per-turn reinforcement.

use crate::config;
use crate::stats;
use regex::Regex;
use serde_json::{json, Value};
use std::io::{Read, Write};

const INDEPENDENT_MODES: &[&str] = &["commit", "review", "compress"];

fn emit(v: &Value) {
    print!("{}", v);
    let _ = std::io::stdout().flush();
}

pub fn run() {
    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        return;
    }
    // Silent-fail on any parse error, matching the JS try/catch.
    let _ = handle(&input);
}

fn handle(input: &str) -> Option<()> {
    let data: Value = serde_json::from_str(input).ok()?;
    let prompt = data
        .get("prompt")
        .and_then(|p| p.as_str())
        .unwrap_or("")
        .trim()
        .to_lowercase();

    let claude_dir = config::claude_dir();
    let flag_path = claude_dir.join(".caveman-active");

    let re_act1 = Regex::new(r"(?i)\b(activate|enable|turn on|start|talk like)\b.*\bcaveman\b").unwrap();
    let re_act2 = Regex::new(r"(?i)\bcaveman\b.*\b(mode|activate|enable|turn on|start)\b").unwrap();
    let re_brief = Regex::new(r"(?i)\b(less tokens|fewer tokens|be brief|be terse|shorter answers)\b").unwrap();
    let re_stop_guard = Regex::new(r"(?i)\b(stop|disable|turn off|deactivate)\b").unwrap();

    // 1. Natural-language activation.
    if (re_act1.is_match(&prompt) || re_act2.is_match(&prompt) || re_brief.is_match(&prompt))
        && !re_stop_guard.is_match(&prompt)
    {
        let mode = config::get_default_mode();
        if mode != "off" {
            config::safe_write_flag(&flag_path, &mode);
        }
    }

    // 2. /caveman-stats [--share] [--all] [--since N] — block + inject output.
    let re_stats = Regex::new(r"^/caveman(?::caveman)?-stats(?:\s+(.*))?$").unwrap();
    if let Some(caps) = re_stats.captures(&prompt) {
        let tail: Vec<String> = caps
            .get(1)
            .map(|m| m.as_str().trim())
            .unwrap_or("")
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        let mut argv: Vec<String> = Vec::new();
        if let Some(tp) = data.get("transcript_path").and_then(|v| v.as_str()) {
            argv.push("--session-file".to_string());
            argv.push(tp.to_string());
        }
        if tail.iter().any(|a| a == "--share") {
            argv.push("--share".to_string());
        }
        if tail.iter().any(|a| a == "--all") {
            argv.push("--all".to_string());
        }
        if let Some(i) = tail.iter().position(|a| a == "--since") {
            if let Some(v) = tail.get(i + 1) {
                argv.push("--since".to_string());
                argv.push(v.clone());
            }
        }
        match stats::run_capture(&argv) {
            Ok(out) => emit(&json!({ "decision": "block", "reason": out.trim() })),
            Err(_) => emit(&json!({
                "decision": "block",
                "reason": "caveman-stats: could not compute stats."
            })),
        }
        return Some(());
    }

    // 3. /caveman commands.
    if prompt.starts_with("/caveman") {
        let parts: Vec<&str> = prompt.split_whitespace().collect();
        let cmd = parts.first().copied().unwrap_or("");
        let arg = parts.get(1).copied().unwrap_or("");

        let mut mode: Option<String> = None;
        if cmd == "/caveman-commit" {
            mode = Some("commit".to_string());
        } else if cmd == "/caveman-review" {
            mode = Some("review".to_string());
        } else if cmd == "/caveman-compress" || cmd == "/caveman:caveman-compress" {
            mode = Some("compress".to_string());
        } else if cmd == "/caveman" || cmd == "/caveman:caveman" {
            if arg.is_empty() {
                mode = Some(config::get_default_mode());
            } else if arg == "off" || arg == "stop" || arg == "disable" {
                mode = Some("off".to_string());
            } else if arg == "wenyan-full" {
                mode = Some("wenyan".to_string());
            } else if config::is_valid_mode(arg) && !INDEPENDENT_MODES.contains(&arg) {
                mode = Some(arg.to_string());
            }
        }

        match mode.as_deref() {
            Some("off") => {
                let _ = std::fs::remove_file(&flag_path);
            }
            Some(m) => config::safe_write_flag(&flag_path, m),
            None => {}
        }
    }

    // 4. Deactivation — natural language and slash commands.
    let re_deact1 = Regex::new(r"(?i)\b(stop|disable|deactivate|turn off)\b.*\bcaveman\b").unwrap();
    let re_deact2 = Regex::new(r"(?i)\bcaveman\b.*\b(stop|disable|deactivate|turn off)\b").unwrap();
    let re_normal = Regex::new(r"(?i)\bnormal mode\b").unwrap();
    if re_deact1.is_match(&prompt) || re_deact2.is_match(&prompt) || re_normal.is_match(&prompt) {
        let _ = std::fs::remove_file(&flag_path);
    }

    // 5. Per-turn reinforcement.
    if let Some(active) = config::read_flag(&flag_path) {
        if !INDEPENDENT_MODES.contains(&active.as_str()) {
            emit(&json!({
                "hookSpecificOutput": {
                    "hookEventName": "UserPromptSubmit",
                    "additionalContext": format!(
                        "CAVEMAN MODE ACTIVE ({}). Drop articles/filler/pleasantries/hedging. Fragments OK. Code/commits/security: write normal.",
                        active
                    )
                }
            }));
        }
    }

    Some(())
}
