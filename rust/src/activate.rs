// caveman — SessionStart activation hook (port of caveman-activate.js)
//
// 1. Writes flag file at $CLAUDE_CONFIG_DIR/.caveman-active
// 2. Emits the caveman ruleset (filtered to the active level) as hidden stdout
// 3. Detects missing statusline config and emits a setup nudge

use crate::config;
use crate::settings;
use regex::Regex;
use std::io::Write;
use std::path::PathBuf;

const INDEPENDENT_MODES: &[&str] = &["commit", "review", "compress"];

/// Locate SKILL.md relative to the running binary:
///   plugin install: <plugin_root>/hooks/caveman -> <plugin_root>/skills/caveman/SKILL.md
/// Standalone hook install: file absent -> fall back to the embedded ruleset.
fn skill_md_path() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let hooks_dir = exe.parent()?;
    let plugin_root = hooks_dir.parent()?;
    Some(plugin_root.join("skills").join("caveman").join("SKILL.md"))
}

fn filter_skill(skill: &str, mode_label: &str) -> String {
    // Strip YAML frontmatter (first ---...--- block).
    let fm = Regex::new(r"(?s)^---.*?---\s*").unwrap();
    let body = fm.replace(skill, "");

    let table_row = Regex::new(r"^\|\s*\*\*(\S+?)\*\*\s*\|").unwrap();
    let example = Regex::new(r"^- (\S+?):\s").unwrap();

    let mut out: Vec<String> = Vec::new();
    for line in body.split('\n') {
        if let Some(c) = table_row.captures(line) {
            if &c[1] == mode_label {
                out.push(line.to_string());
            }
            continue;
        }
        if let Some(c) = example.captures(line) {
            if &c[1] == mode_label {
                out.push(line.to_string());
            }
            continue;
        }
        out.push(line.to_string());
    }
    out.join("\n")
}

fn fallback_ruleset(mode_label: &str) -> String {
    format!(
        "CAVEMAN MODE ACTIVE — level: {ml}\n\n\
Respond terse like smart caveman. All technical substance stay. Only fluff die.\n\n\
## Persistence\n\n\
ACTIVE EVERY RESPONSE. No revert after many turns. No filler drift. Still active if unsure. Off only: \"stop caveman\" / \"normal mode\".\n\n\
Current level: **{ml}**. Switch: `/caveman lite|full|ultra`.\n\n\
## Rules\n\n\
Drop: articles (a/an/the), filler (just/really/basically/actually/simply), pleasantries (sure/certainly/of course/happy to), hedging. \
Fragments OK. Short synonyms (big not extensive, fix not \"implement a solution for\"). Technical terms exact. Code blocks unchanged. Errors quoted exact.\n\n\
Pattern: `[thing] [action] [reason]. [next step].`\n\n\
Not: \"Sure! I'd be happy to help you with that. The issue you're experiencing is likely caused by...\"\n\
Yes: \"Bug in auth middleware. Token expiry check use `<` not `<=`. Fix:\"\n\n\
## Auto-Clarity\n\n\
Drop caveman for: security warnings, irreversible action confirmations, multi-step sequences where fragment order risks misread, user asks to clarify or repeats question. Resume caveman after clear part done.\n\n\
## Boundaries\n\n\
Code/commits/PRs: write normal. \"stop caveman\" or \"normal mode\": revert. Level persist until changed or session end.",
        ml = mode_label
    )
}

pub fn run() {
    let claude_dir = config::claude_dir();
    let flag_path = claude_dir.join(".caveman-active");
    let settings_path = claude_dir.join("settings.json");

    let mode = config::get_default_mode();

    // "off" — skip activation entirely, remove flag.
    if mode == "off" {
        let _ = std::fs::remove_file(&flag_path);
        print!("OK");
        let _ = std::io::stdout().flush();
        return;
    }

    // 1. Write flag (symlink-safe)
    config::safe_write_flag(&flag_path, &mode);

    // Independent modes — short activation line, behavior lives in their skill.
    if INDEPENDENT_MODES.contains(&mode.as_str()) {
        print!(
            "CAVEMAN MODE ACTIVE — level: {m}. Behavior defined by /caveman-{m} skill.",
            m = mode
        );
        let _ = std::io::stdout().flush();
        return;
    }

    let mode_label = if mode == "wenyan" {
        "wenyan-full".to_string()
    } else {
        mode.clone()
    };

    // 2. Build the ruleset output from SKILL.md on disk (plugin layout or the
    //    install-extracted copy), else the SKILL.md baked into this binary, else
    //    the minimal hardcoded fallback.
    let skill_content = skill_md_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .or_else(|| crate::install::embedded_caveman_skill().map(|s| s.to_string()));
    let mut output = match skill_content {
        Some(skill) => {
            let filtered = filter_skill(&skill, &mode_label);
            format!("CAVEMAN MODE ACTIVE — level: {}\n\n{}", mode_label, filtered)
        }
        None => fallback_ruleset(&mode_label),
    };

    // 3. Statusline-missing nudge.
    if !settings::has_statusline(&settings_path) {
        let exe = std::env::current_exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "caveman".to_string());
        let command = format!("\"{}\" statusline", exe);
        let snippet = format!(
            "\"statusLine\": {{ \"type\": \"command\", \"command\": {} }}",
            serde_json::to_string(&command).unwrap_or_else(|_| "\"caveman statusline\"".into())
        );
        output.push_str(&format!(
            "\n\nSTATUSLINE SETUP NEEDED: The caveman plugin includes a statusline badge showing active mode \
(e.g. [CAVEMAN], [CAVEMAN:ULTRA]). It is not configured yet. \
To enable, add this to {}: {} \
Proactively offer to set this up for the user on first interaction.",
            settings_path.display(),
            snippet
        ));
    }

    print!("{}", output);
    let _ = std::io::stdout().flush();
}
