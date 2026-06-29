// caveman init — drop the always-on caveman rule into a target repo for every
// IDE agent we support (port of src/tools/caveman-init.js). Idempotent.
//
// Rule body is embedded at compile time from the single source of truth so it
// can never drift from src/rules/caveman-activate.md.

use std::path::{Path, PathBuf};

const RULE_BODY_RAW: &str = include_str!("../../src/rules/caveman-activate.md");
const SENTINEL: &str = "Respond terse like smart caveman";

struct Agent {
    id: &'static str,
    file: Option<&'static str>,
    frontmatter: &'static str,
    mode: Mode,
    description: Option<&'static str>,
}

#[derive(PartialEq)]
enum Mode {
    Replace,
    Append,
    Openclaw,
}

fn agents() -> Vec<Agent> {
    vec![
        Agent { id: "cursor", file: Some(".cursor/rules/caveman.mdc"),
            frontmatter: "---\ndescription: \"Caveman mode — terse communication, ~75% fewer tokens, full technical accuracy\"\nalwaysApply: true\n---\n\n",
            mode: Mode::Replace, description: None },
        Agent { id: "windsurf", file: Some(".windsurf/rules/caveman.md"),
            frontmatter: "---\ntrigger: always_on\n---\n\n", mode: Mode::Replace, description: None },
        Agent { id: "cline", file: Some(".clinerules/caveman.md"),
            frontmatter: "", mode: Mode::Replace, description: None },
        Agent { id: "copilot", file: Some(".github/copilot-instructions.md"),
            frontmatter: "", mode: Mode::Append, description: None },
        Agent { id: "opencode", file: Some(".opencode/AGENTS.md"),
            frontmatter: "", mode: Mode::Append, description: None },
        Agent { id: "agents", file: Some("AGENTS.md"),
            frontmatter: "", mode: Mode::Append, description: None },
        Agent { id: "openclaw", file: None, frontmatter: "", mode: Mode::Openclaw,
            description: Some("~/.openclaw/workspace/{skills/caveman/, SOUL.md}") },
    ]
}

struct Opts {
    dry_run: bool,
    force: bool,
    only: Option<String>,
    target: PathBuf,
    help: bool,
}

fn rule_body() -> String {
    format!("{}\n", RULE_BODY_RAW.trim_end())
}

struct Outcome {
    status: String,
    label: char,
    detail: Option<String>,
}

fn process_agent(agent: &Agent, target_dir: &Path, rb: &str, opts: &Opts) -> Outcome {
    if agent.mode == Mode::Openclaw {
        // The OpenClaw helper was a separate Node module (bin/lib/openclaw.js)
        // that is not part of this distribution. Native install for OpenClaw is
        // out of scope for the Claude Code-focused port.
        return Outcome {
            status: "unsupported-standalone".to_string(),
            label: 'x',
            detail: Some("~/.openclaw/workspace (run the dedicated OpenClaw installer)".to_string()),
        };
    }

    let file = agent.file.unwrap();
    let full_path = target_dir.join(file);
    let exists = full_path.exists();

    if !exists {
        if !opts.dry_run {
            if let Some(parent) = full_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&full_path, format!("{}{}", agent.frontmatter, rb));
        }
        return Outcome { status: "added".into(), label: '+', detail: None };
    }

    let existing = std::fs::read_to_string(&full_path).unwrap_or_default();
    if existing.contains(SENTINEL) {
        return Outcome { status: "skipped-already-installed".into(), label: '=', detail: None };
    }

    if agent.mode == Mode::Append {
        if !opts.dry_run {
            let sep = if existing.ends_with("\n\n") {
                ""
            } else if existing.ends_with('\n') {
                "\n"
            } else {
                "\n\n"
            };
            let _ = std::fs::write(&full_path, format!("{}{}{}", existing, sep, rb));
        }
        return Outcome { status: "appended".into(), label: '~', detail: None };
    }

    if opts.force {
        if !opts.dry_run {
            let _ = std::fs::write(&full_path, format!("{}{}", agent.frontmatter, rb));
        }
        return Outcome { status: "overwritten".into(), label: '!', detail: None };
    }

    Outcome { status: "skipped-exists".into(), label: '?', detail: None }
}

fn parse_args(argv: &[String]) -> Opts {
    let mut opts = Opts {
        dry_run: false,
        force: false,
        only: None,
        target: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        help: false,
    };
    let mut i = 0;
    while i < argv.len() {
        let a = &argv[i];
        match a.as_str() {
            "--dry-run" => opts.dry_run = true,
            "--force" | "-f" => opts.force = true,
            "--only" => {
                i += 1;
                if let Some(v) = argv.get(i) {
                    opts.only = Some(v.clone());
                }
            }
            "-h" | "--help" => opts.help = true,
            s if !s.starts_with('-') => {
                opts.target = std::fs::canonicalize(s).unwrap_or_else(|_| PathBuf::from(s));
            }
            _ => {}
        }
        i += 1;
    }
    opts
}

fn print_help() {
    let list: String = agents()
        .iter()
        .map(|a| format!("  {:<10} {}", a.id, a.file.or(a.description).unwrap_or("")))
        .collect::<Vec<_>>()
        .join("\n");
    println!(
        "caveman init — drop always-on caveman rule into a target repo\n\n\
Usage: caveman init [target-dir] [--dry-run] [--force] [--only <agent>]\n\n\
Defaults to current working directory. Idempotent — safe to re-run.\n\n\
Targets installed:\n{}\n\n\
Flags:\n\
  --dry-run   show what would change, do not write\n\
  --force     overwrite existing rule files (default: skip)\n\
  --only <id> only install for one agent (id from list above)",
        list
    );
}

pub fn run(argv: &[String]) -> i32 {
    let opts = parse_args(argv);
    if opts.help {
        print_help();
        return 0;
    }

    println!(
        "🪨 caveman init — {}{}\n",
        opts.target.display(),
        if opts.dry_run { " (dry run)" } else { "" }
    );

    let rb = rule_body();
    let (mut added, mut appended, mut overwritten, mut skipped) = (0, 0, 0, 0);

    for agent in agents() {
        if let Some(only) = &opts.only {
            if only != agent.id {
                continue;
            }
        }
        let result = process_agent(&agent, &opts.target, &rb, &opts);
        let target = agent
            .file
            .map(|f| f.to_string())
            .or_else(|| result.detail.clone())
            .or_else(|| agent.description.map(|d| d.to_string()))
            .unwrap_or_else(|| agent.id.to_string());
        println!("  {} {} ({})", result.label, target, result.status);
        match result.status.as_str() {
            "added" | "installed" | "would-add" => added += 1,
            "appended" => appended += 1,
            "overwritten" => overwritten += 1,
            _ => skipped += 1,
        }
    }

    println!(
        "\n{} added, {} appended, {} overwritten, {} skipped",
        added, appended, overwritten, skipped
    );
    if opts.dry_run {
        println!("(dry run — no files were written)");
    }
    0
}
