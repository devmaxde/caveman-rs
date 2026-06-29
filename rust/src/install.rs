// caveman — native installer/uninstaller. Replaces install.sh + uninstall.sh's
// inline `node -e` settings.json editing. No Node, ever.
//
// `install` copies the running binary into <CLAUDE_CONFIG_DIR>/hooks/caveman and
// wires the SessionStart + UserPromptSubmit hooks and the statusline badge.

use crate::config;
use crate::settings;
use include_dir::{include_dir, Dir};
use std::path::Path;

const SESSION_EVENTS: &[&str] = &["SessionStart", "UserPromptSubmit"];

// The skills + agents are baked INTO the binary at build time. A standalone
// `caveman install` extracts them — no repo checkout needed at install time.
static SKILLS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/../skills");
static AGENTS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/../agents");

fn quote(p: &Path) -> String {
    format!("\"{}\"", p.display())
}

/// Recursively write an embedded directory to disk.
fn extract_dir(dir: &Dir, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for file in dir.files() {
        let name = file.path().file_name().unwrap();
        std::fs::write(dst.join(name), file.contents())?;
    }
    for sub in dir.dirs() {
        let name = sub.path().file_name().unwrap();
        if name == "__pycache__" {
            continue;
        }
        extract_dir(sub, &dst.join(name))?;
    }
    Ok(())
}

/// Extract every embedded `skills/<name>/` (with a SKILL.md) into
/// `$CLAUDE_CONFIG_DIR/skills/` so each becomes a registered `/<name>` command —
/// this is what stops Claude Code from printing "Unknown command: /caveman".
/// Also extracts the embedded cavecrew agents. Returns the registered names.
fn install_skills(claude_dir: &Path) -> Vec<String> {
    let mut registered = Vec::new();
    let skills_dst = claude_dir.join("skills");
    for sub in SKILLS_DIR.dirs() {
        if sub.get_file(sub.path().join("SKILL.md")).is_none() {
            continue;
        }
        let name = sub.path().file_name().unwrap().to_string_lossy().to_string();
        let dst = skills_dst.join(&name);
        let _ = std::fs::remove_dir_all(&dst); // idempotent refresh
        if extract_dir(sub, &dst).is_ok() {
            registered.push(name);
        }
    }
    // cavecrew subagents
    let agents_dst = claude_dir.join("agents");
    let _ = std::fs::create_dir_all(&agents_dst);
    for file in AGENTS_DIR.files() {
        if file.path().extension().map(|x| x == "md").unwrap_or(false) {
            let name = file.path().file_name().unwrap();
            let _ = std::fs::write(agents_dst.join(name), file.contents());
        }
    }
    registered.sort();
    registered
}

/// True for skill/agent names this installer owns (safe to remove on uninstall).
fn is_caveman_skill(name: &str) -> bool {
    name == "caveman" || name.starts_with("caveman-") || name == "cavecrew"
}

/// The embedded `caveman/SKILL.md` — used by `activate` as a guaranteed-correct
/// fallback when no SKILL.md is found on disk.
pub fn embedded_caveman_skill() -> Option<&'static str> {
    SKILLS_DIR
        .get_file("caveman/SKILL.md")
        .and_then(|f| f.contents_utf8())
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = std::fs::metadata(path) {
        let mut perms = meta.permissions();
        perms.set_mode(0o755);
        let _ = std::fs::set_permissions(path, perms);
    }
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) {}

pub fn run_install(args: &[String]) -> i32 {
    let force = args.iter().any(|a| a == "--force" || a == "-f");

    let claude_dir = config::claude_dir();
    let hooks_dir = claude_dir.join("hooks");
    let settings_path = claude_dir.join("settings.json");

    if let Err(e) = std::fs::create_dir_all(&hooks_dir) {
        eprintln!("caveman: cannot create {}: {}", hooks_dir.display(), e);
        return 1;
    }

    // 1. Copy the running binary into the hooks dir (unless we're already it).
    let target_bin = hooks_dir.join("caveman");
    let current = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("caveman: cannot resolve own path: {}", e);
            return 1;
        }
    };
    let same = std::fs::canonicalize(&current).ok() == std::fs::canonicalize(&target_bin).ok()
        && target_bin.exists();
    if !same {
        if target_bin.exists() && !force {
            // Refresh anyway so the wired binary matches the one being run.
        }
        if let Err(e) = std::fs::copy(&current, &target_bin) {
            eprintln!("caveman: cannot install binary to {}: {}", target_bin.display(), e);
            return 1;
        }
        make_executable(&target_bin);
        println!("  Installed binary: {}", target_bin.display());
    } else {
        println!("  Using installed binary: {}", target_bin.display());
    }

    // 2. Back up + load settings.json.
    if settings_path.exists() {
        let _ = std::fs::copy(&settings_path, settings_path.with_extension("json.bak"));
    }
    let mut settings = settings::read_settings(&settings_path);

    // 3. Wire hooks.
    let activate_cmd = format!("{} activate", quote(&target_bin));
    let tracker_cmd = format!("{} mode-tracker", quote(&target_bin));
    let statusline_cmd = format!("{} statusline", quote(&target_bin));

    if settings::add_caveman_hook(&mut settings, "SessionStart", &activate_cmd, "Loading caveman mode...") {
        println!("  Wired SessionStart hook.");
    } else {
        println!("  SessionStart hook already present.");
    }
    if settings::add_caveman_hook(&mut settings, "UserPromptSubmit", &tracker_cmd, "Tracking caveman mode...") {
        println!("  Wired UserPromptSubmit hook.");
    } else {
        println!("  UserPromptSubmit hook already present.");
    }

    // 4. Statusline.
    if !settings::has_statusline_value(&settings) {
        settings["statusLine"] = serde_json::json!({
            "type": "command",
            "command": statusline_cmd
        });
        println!("  Statusline badge configured.");
    } else {
        let cmd = settings::statusline_command(&settings);
        if cmd.contains(&target_bin.to_string_lossy().to_string()) {
            println!("  Statusline badge already configured.");
        } else {
            println!("  NOTE: Existing statusline detected — caveman badge NOT added.");
            println!("        Add `{}` to your statusline manually to show the badge.", statusline_cmd);
        }
    }

    if let Err(e) = settings::write_settings(&settings_path, &settings) {
        eprintln!("caveman: cannot write {}: {}", settings_path.display(), e);
        return 1;
    }
    println!("  Hooks wired in {}", settings_path.display());

    // 5. Register slash-command skills (extracted from the embedded copy baked
    //    into this binary) so Claude Code recognizes `/caveman` etc. Without
    //    this the prompt hook still works but Claude prints "Unknown command".
    //    This also puts SKILL.md where `activate` looks, so installs emit the
    //    real ruleset, not the fallback.
    let registered = install_skills(&claude_dir);
    if registered.is_empty() {
        println!("  NOTE: no embedded skills to register (unexpected).");
    } else {
        println!(
            "  Registered {} slash-command skill(s) in {}: {}",
            registered.len(),
            claude_dir.join("skills").display(),
            registered
                .iter()
                .map(|n| format!("/{}", n))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    println!("\nDone! Restart Claude Code to activate.");
    0
}

pub fn run_uninstall(_args: &[String]) -> i32 {
    let claude_dir = config::claude_dir();
    let hooks_dir = claude_dir.join("hooks");
    let settings_path = claude_dir.join("settings.json");
    let target_bin = hooks_dir.join("caveman");
    let flag_file = claude_dir.join(".caveman-active");

    println!("Uninstalling caveman hooks...");

    // 1. Remove caveman entries from settings.json.
    if settings_path.exists() {
        let _ = std::fs::copy(&settings_path, settings_path.with_extension("json.bak"));
        let mut settings = settings::read_settings(&settings_path);

        let removed = settings::remove_caveman_hooks(&mut settings, SESSION_EVENTS);

        // Remove statusLine if it references the managed binary.
        let cmd = settings::statusline_command(&settings);
        if cmd.contains(&target_bin.to_string_lossy().to_string()) || cmd.contains("caveman") {
            if let Some(obj) = settings.as_object_mut() {
                obj.remove("statusLine");
            }
            println!("  Removed caveman statusLine from settings.json");
        }

        if let Err(e) = settings::write_settings(&settings_path, &settings) {
            eprintln!("caveman: cannot write {}: {}", settings_path.display(), e);
        } else {
            println!("  Removed {} caveman hook entries from settings.json", removed);
        }

        let bak = settings_path.with_extension("json.bak");
        if bak.exists() {
            let _ = std::fs::remove_file(&bak);
            println!("  Removed: {}", bak.display());
        }
    }

    // 2. Remove the installed binary.
    if target_bin.exists() {
        let _ = std::fs::remove_file(&target_bin);
        println!("  Removed: {}", target_bin.display());
    }

    // 3. Remove registered slash-command skills + cavecrew agents we installed.
    let mut removed_skills = 0;
    if let Ok(rd) = std::fs::read_dir(claude_dir.join("skills")) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if e.path().is_dir() && is_caveman_skill(&name) && std::fs::remove_dir_all(e.path()).is_ok()
            {
                removed_skills += 1;
            }
        }
    }
    if removed_skills > 0 {
        println!("  Removed {} caveman skill(s) from {}", removed_skills, claude_dir.join("skills").display());
    }
    if let Ok(rd) = std::fs::read_dir(claude_dir.join("agents")) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with("cavecrew-") && name.ends_with(".md") {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }

    // 4. Remove flag file.
    if flag_file.exists() {
        let _ = std::fs::remove_file(&flag_file);
        println!("  Removed: {}", flag_file.display());
    }

    println!("\nDone! Restart Claude Code to complete the uninstall.");
    0
}
