# CLAUDE.md — caveman

## README is a product artifact

README = product front door. Non-technical people read it to decide if caveman worth install. Treat like UI copy.

**Rules for any README change:**

- Readable by non-AI-agent users. If you write "SessionStart hook injects system context," invisible to most — translate it.
- Keep Before/After examples first. That the pitch.
- Install table always complete + accurate. One broken install command costs real user.
- What You Get table must sync with actual code. Feature ships or removed → update table.
- Preserve voice. Caveman speak in README on purpose. "Brain still big." "Cost go down forever." "One rock. That it." — intentional brand. Don't normalize.
- Benchmark numbers from real runs in `benchmarks/` and `evals/`. Never invent or round. Re-run if doubt.
- Adding new agent to install table → add detail block in `<details>` section below.
- Readability check before any README commit: would non-programmer understand + install within 60 seconds?

---

## Project overview

Caveman makes AI coding agents respond in compressed caveman-style prose — cuts ~65-75% output tokens, full technical accuracy. Ships as Claude Code plugin, Codex plugin, Gemini CLI extension, agent rule files for Cursor, Windsurf, Cline, Copilot, 40+ others via `npx skills`.

---

## What lives where

Post-cleanup layout. Sources of truth at the top, distribution mirrors below, build outputs in `dist/`, human docs alongside each skill.

```
caveman/
├── README.md                    # Front door (product pitch)
├── INSTALL.md                   # Per-agent install commands
├── CONTRIBUTING.md              # Dev guide
├── CLAUDE.md                    # This file (maintainer instructions)
├── AGENTS.md / GEMINI.md        # Autodiscovery files (must stay at root)
│
├── install.sh / install.ps1     # cargo build --release → `caveman install` (no Node)
│
├── rust/                        # Native Rust binary — the WHOLE Claude Code runtime
│   ├── Cargo.toml
│   └── src/                     # one binary, subcommands replace the old Node hooks:
│       ├── main.rs              #   dispatch
│       ├── config.rs           #   mode resolution + symlink-safe flag I/O (was caveman-config.js)
│       ├── activate.rs         #   SessionStart hook (was caveman-activate.js)
│       ├── mode_tracker.rs     #   UserPromptSubmit hook (was caveman-mode-tracker.js)
│       ├── stats.rs            #   /caveman-stats (was caveman-stats.js)
│       ├── statusline.rs       #   badge (was caveman-statusline.sh/.ps1)
│       ├── init.rs             #   per-repo rules (was src/tools/caveman-init.js)
│       ├── install.rs          #   install/uninstall + settings.json edit (was install.sh's node -e)
│       └── settings.rs         #   JSONC-tolerant settings.json reader/writer
│
├── skills/                      # ALL skills, single source of truth
│   ├── caveman/{SKILL.md, README.md}
│   ├── caveman-commit/{SKILL.md, README.md}
│   ├── caveman-review/{SKILL.md, README.md}
│   ├── caveman-help/{SKILL.md, README.md}
│   ├── caveman-stats/{SKILL.md, README.md}
│   ├── caveman-compress/{SKILL.md, README.md, scripts/}
│   └── cavecrew/{SKILL.md, README.md}
│
├── agents/                      # cavecrew subagents (single source — kept at root for plugin auto-discovery)
├── commands/                    # Codex/Gemini TOML command stubs (root for plugin auto-discovery)
│
├── src/                         # Internal source — not auto-discovered by plugin
│   └── rules/                   # Auto-activation rule bodies (single source; embedded into the Rust binary)
│
├── .claude-plugin/              # Claude Code plugin manifest (REQUIRED at root)
├── plugins/caveman/             # Claude Code plugin distribution (CI-mirrored)
│   ├── skills/                  # ← from skills/
│   └── agents/                  # ← from agents/
│
├── dist/                        # Build artifacts (gitignored)
│   └── caveman.skill            # ZIP of skills/caveman/, rebuilt by CI
│
├── tests/                       # Python compress-skill tests (Rust tests live in rust/ via `cargo test`)
├── benchmarks/                  # Real token measurements through Claude API
├── evals/                       # Three-arm eval harness
├── docs/                        # User-facing docs site
└── .github/workflows/           # CI sync
```

---

## File structure and what owns what

### Single source of truth files — edit only these

| File | What it controls |
|------|-----------------|
| `skills/caveman/SKILL.md` | Caveman behavior: intensity levels, rules, wenyan mode, auto-clarity, persistence. Only file to edit for behavior changes. `rust/src/activate.rs` reads this at runtime to emit the ruleset (filtered to the active level). |
| `src/rules/caveman-activate.md` | Always-on auto-activation rule body. **Embedded into the Rust binary at build time** via `include_str!` in `rust/src/init.rs` (powers `caveman init`). Edit here; rebuild to propagate. |
| `src/rules/caveman-openclaw-bootstrap.md` | Marker-fenced OpenClaw SOUL.md bootstrap snippet. Currently **not wired into the Rust binary** — OpenClaw native install was a Node helper (`bin/lib/openclaw.js`) that was removed in the Rust port. Kept as the source of truth if OpenClaw support is re-added. Must keep the SENTINEL `Respond terse like smart caveman`. |
| `skills/caveman-commit/SKILL.md` | Caveman commit message behavior. Fully independent skill. |
| `skills/caveman-review/SKILL.md` | Caveman code review behavior. Fully independent skill. |
| `skills/caveman-help/SKILL.md` | Quick-reference card. One-shot display, not a persistent mode. |
| `skills/caveman-compress/SKILL.md` | Compress sub-skill behavior. |
| `skills/cavecrew/SKILL.md` | Cavecrew decision guide — when to delegate to caveman subagents vs vanilla. Edit only here. |
| `agents/cavecrew-investigator.md` | Read-only locator subagent (haiku). Output contract: `path:line — symbol — note`. |
| `agents/cavecrew-builder.md` | Surgical 1-2 file editor subagent. Refuses 3+ file scope. |
| `agents/cavecrew-reviewer.md` | Diff/file reviewer subagent (haiku). One-line findings with severity emoji. |
| `rust/src/config.rs` | Mode resolution + symlink-safe flag read/write/append. Port of the old `caveman-config.js`. Every flag-file write goes through `safe_write_flag` here. |
| `rust/src/settings.rs` | JSONC-tolerant `settings.json` reader/writer + caveman hook add/remove. Used by `caveman install`/`uninstall`. |

### Auto-generated / auto-synced — do not edit directly

We removed the agent-specific dotdir mirrors at the repo root (`.cursor/`, `.windsurf/`, `.clinerules/`, `.github/copilot-instructions.md`, root `caveman/SKILL.md`). They were never read by the installer — only used to self-apply caveman to this repo when a maintainer opened it in Cursor/Windsurf/Cline. Devs who want caveman in their editor while editing this repo should run `caveman init` once (writes per-repo rule files; the rule body is embedded from `src/rules/caveman-activate.md` at build time). For per-user installs through the upstream skills CLI, run `npx skills add JuliusBrussee/caveman -a <profile>`.

A handful of dotdir leftovers (`.junie/`, `.kiro/`, `.roo/`, `.agents/`) still hold a stale `cavecrew/SKILL.md` mirror from before the cleanup. They aren't read by anything in the current install path; remove on sight, no migration needed.

What's left is the Claude Code plugin distribution (required by the plugin loader) and the release ZIP.

| File | Synced from |
|------|-------------|
| `plugins/caveman/skills/caveman/SKILL.md` | `skills/caveman/SKILL.md` |
| `plugins/caveman/skills/caveman-compress/SKILL.md` (+ `scripts/`) | `skills/caveman-compress/SKILL.md` (+ `scripts/`) |
| `plugins/caveman/skills/cavecrew/SKILL.md` | `skills/cavecrew/SKILL.md` |
| `plugins/caveman/agents/cavecrew-*.md` | `agents/cavecrew-*.md` |
| `dist/caveman.skill` | ZIP of `skills/caveman/` directory (gitignored; rebuilt by CI on release) |

Skills not in this table (`caveman-commit`, `caveman-review`, `caveman-help`, `caveman-stats`) are not mirrored into the Claude Code plugin distribution by CI. They reach Claude Code through the standalone hook + skill install path, and reach other agents via `npx skills add`. A `plugins/caveman/skills/caveman-stats/` directory is currently checked in as a hand-committed copy; the sync workflow does not touch it, so don't rely on edits there to propagate.

---

## CI sync workflow

`.github/workflows/sync-skill.yml` triggers on main push when `skills/**/SKILL.md` or `agents/cavecrew-*.md` changes.

What it does:
1. Copies `skills/caveman/SKILL.md` and `skills/cavecrew/SKILL.md` into their `plugins/caveman/skills/<name>/` mirrors so the Claude Code plugin loader sees the latest behavior.
2. Copies `skills/caveman-compress/SKILL.md` and its `scripts/` into `plugins/caveman/skills/caveman-compress/`.
3. Copies `agents/cavecrew-*.md` into `plugins/caveman/agents/`.
4. Rebuilds `dist/caveman.skill` (ZIP of `skills/caveman/`) for the release artifact.
5. Commits and pushes with `[skip ci]` to avoid loops.

CI bot commits as `github-actions[bot]`. After PR merge, wait for workflow before declaring release complete.

The old steps that mirrored SKILL.md and rules into root dotdirs (`.cursor/`, `.windsurf/`, `.clinerules/`, `.github/copilot-instructions.md`) are gone — those mirrors no longer exist. The old `caveman-compress/` → `skills/compress/` rename-on-sync is also gone now that compress lives at `skills/caveman-compress/`.

---

## Hook system (Claude Code) — native Rust

**No Node.** One Rust binary, `caveman`, is the entire Claude Code runtime. Each
hook is a subcommand of the same binary (built from `rust/`). Hooks communicate
via the flag file at `$CLAUDE_CONFIG_DIR/.caveman-active` (falls back to
`~/.claude/.caveman-active`). All subcommands honor `CLAUDE_CONFIG_DIR`.

```
caveman activate ──writes "full"──▶ $CLAUDE_CONFIG_DIR/.caveman-active ◀──writes mode── caveman mode-tracker
   (SessionStart)                                    │                                    (UserPromptSubmit)
                                                   reads
                                                     ▼
                                            caveman statusline
                                          [CAVEMAN] / [CAVEMAN:ULTRA] / ...
```

### `rust/src/config.rs` — shared module

- `get_default_mode()` — resolves the default mode in order: `CAVEMAN_DEFAULT_MODE` env var → repo-local config (`<cwd>/.caveman/config.json` or `<cwd>/.caveman.json`, walking up to the filesystem root) → user config (`$XDG_CONFIG_HOME/caveman/config.json` / `~/.config/caveman/config.json` / `%APPDATA%\caveman\config.json`) → `"full"`. The env var short-circuits before any cwd walk.
- `find_repo_config_path(start)` — walks up looking for the first `.caveman/config.json` or `.caveman.json`. Bounded to 64 ancestors. Refuses symlinked files.
- `safe_write_flag(path, content)` — symlink-safe write: refuses if the flag target is a symlink; a symlinked **parent** dir is allowed only when it resolves to a dir owned by the current uid. `O_NOFOLLOW`, atomic temp + rename, `0600`. Silent-fails on any fs error.
- `read_flag` / `append_flag` / `read_history` — symmetric symlink-safe read, 64-byte + whitelist cap on the flag, append-only history log.

### `caveman activate` — SessionStart hook (`rust/src/activate.rs`)

1. Writes the active mode to the flag file via `safe_write_flag` (or deletes it for `off`).
2. Emits the caveman ruleset on stdout — Claude Code injects SessionStart stdout as hidden system context. The ruleset is read from `skills/caveman/SKILL.md` relative to the binary (plugin layout: `<plugin_root>/skills/caveman/SKILL.md`) and filtered to the active level; if SKILL.md isn't found (standalone install), an embedded fallback ruleset is used.
3. If `settings.json` has no statusline, appends a setup nudge.

Silent-fails on all filesystem errors — never blocks session start.

### `caveman mode-tracker` — UserPromptSubmit hook (`rust/src/mode_tracker.rs`)

Reads JSON from stdin. Four responsibilities:

**1. Slash-command activation.** `/caveman`, `/caveman lite|ultra|wenyan|wenyan-lite|wenyan-full|wenyan-ultra`, `/caveman off|stop|disable`, `/caveman-commit`, `/caveman-review`, `/caveman-compress` (and the `caveman:`-prefixed plugin forms).

**2. Natural-language activation/deactivation.** "activate/turn on/talk like caveman", "less/fewer tokens", "be brief/terse" → default mode; "stop/disable/deactivate caveman", "normal mode" → delete flag.

**3. `/caveman-stats`.** Runs the stats logic **in-process** (calls `stats::run_capture`, no subprocess) and returns the output as a `decision: "block"` reason.

**4. Per-turn reinforcement.** When the flag is a non-independent mode, emits a `hookSpecificOutput` JSON reminder so the model keeps caveman style after other plugins inject competing instructions mid-conversation.

### `caveman statusline` — badge (`rust/src/statusline.rs`)

Reads the flag file. `full`/empty → `[CAVEMAN]` (orange); else `[CAVEMAN:<MODE>]`. Appends the lifetime-savings suffix (`⛏ 12.4k`) from `$CLAUDE_CONFIG_DIR/.caveman-statusline-suffix`, written by `caveman stats` on every run. **Default on**; opt out with `CAVEMAN_STATUSLINE_SAVINGS=0`. Suffix file absent until stats runs once. Symlink-refuses + strips control bytes — never echoes arbitrary bytes.

### Install / uninstall (`rust/src/install.rs`, `rust/src/settings.rs`)

**Self-contained binary.** `skills/` and `agents/` are baked into the binary at build time via `include_dir!` (see `SKILLS_DIR` / `AGENTS_DIR` in `rust/src/install.rs`). So the compiled `caveman` is fully portable — copy it anywhere and `caveman install` works with no repo present.

**Standalone** — `bash install.sh` (or `pwsh install.ps1`) runs `cargo build --release` then `caveman install`, which: (1) copies the binary to `$CLAUDE_CONFIG_DIR/hooks/caveman`; (2) merges the SessionStart + UserPromptSubmit hooks and the statusline into `settings.json`; (3) **registers the slash commands** by extracting the embedded `skills/*` into `$CLAUDE_CONFIG_DIR/skills/` and `agents/*` into `$CLAUDE_CONFIG_DIR/agents/`. Step 3 is what makes Claude Code recognize `/caveman`, `/caveman-commit`, etc. (the prompt hook fires on the raw `/caveman …` text regardless, but without a registered skill Claude Code prints "Unknown command"). It also lands `SKILL.md` exactly where `caveman activate` looks, so installs emit the real filtered ruleset. `settings.rs` is JSONC-tolerant (strips comments + trailing commas on read), backs up to `settings.json.bak`, and is idempotent. Hook commands embed the binary path and contain the substring `caveman` for detection.

> **Rebuild after editing any `skills/**` or `agents/*` file** — they're embedded at compile time, so `cargo build --release` must re-run for changes to ship in the binary. `activate` also uses the embedded `caveman/SKILL.md` (`install::embedded_caveman_skill()`) as its fallback ruleset.

**Plugin** — `.claude-plugin/plugin.json` wires the SessionStart/UserPromptSubmit hooks to a small `sh -c` wrapper that builds the Rust binary on first use (`cargo build --release`, Rust required) then `exec`s it. Still no Node.

**Uninstall** — `bash install.sh --uninstall` (or `caveman uninstall`). Strips caveman hook entries + statusline from `settings.json` (substring `caveman`), removes the registered skills (`skills/caveman*`, `skills/cavecrew`) and cavecrew agents, the installed binary, and the flag file. `npx skills add` installs for other agents are removed via their own tooling.

**Prebuilt-binary install (no Rust)** — `install-release.sh` downloads the statically-linked release binary, verifies its SHA-256, and runs `caveman install`. Because skills/agents are baked in via `include_dir!`, the downloaded binary is fully self-contained — no repo, no cargo, no rebuild. Keep this script separate from `install.sh`/`install.ps1` (those build from source); it must never shell out to cargo or Node. The binaries it pulls come from the release pipeline below.

---

## Release pipeline

`.github/workflows/release.yml` triggers on `v*` tag pushes (and `workflow_dispatch`).

What it does:
1. Builds `caveman` for `x86_64-unknown-linux-musl` — statically linked (`RUSTFLAGS=-C target-feature=+crt-static`, musl is static by default), so it runs on any Linux distro with no shared-lib deps.
2. Runs `cargo test` and asserts the output binary is static (`ldd` → "not a dynamic executable") before publishing.
3. Packages the raw binary `caveman-x86_64-unknown-linux-musl`, a `caveman-x86_64-unknown-linux-musl.tar.gz`, and a `.sha256` checksum file.
4. Publishes all three as assets on the GitHub Release for the tag (auto-generated notes).

To cut a release: bump `version` in `rust/Cargo.toml`, commit, then `git tag vX.Y.Z && git push --tags`. Users then install with no toolchain via `install-release.sh` (see above). Adding another arch (i686, aarch64) = add a `target` to the matrix and the `rustup target add` line; `install-release.sh`'s `case "$ARCH"` maps `uname -m` → target name.

---

## Skill system

Skills = Markdown files with YAML frontmatter consumed by Claude Code's skill/plugin system and by `npx skills` for other agents.

Each skill has a human-facing `README.md` alongside the LLM-facing `SKILL.md`. The README explains what the skill does for users browsing GitHub; the SKILL.md is the prompt body the agent loads. Don't merge them — different audiences, different formats.

### Intensity levels

Defined in `skills/caveman/SKILL.md`. Six levels: `lite`, `full` (default), `ultra`, `wenyan-lite`, `wenyan-full`, `wenyan-ultra`. Persists until changed or session ends.

### Auto-clarity rule

Caveman drops to normal prose for: security warnings, irreversible action confirmations, multi-step sequences where fragment ambiguity risks misread, user confused or repeating question. Resumes after. Defined in skill — preserve in any SKILL.md edit.

### caveman-compress

Sub-skill in `skills/caveman-compress/SKILL.md`. Takes file path, compresses prose to caveman style, writes to original path, saves backup at `<filename>.original.md`. Validates headings, code blocks, URLs, file paths, commands preserved. Retries up to 2 times on failure with targeted patches only. Requires Python 3.10+.

The slash command is `/caveman-compress` everywhere — same name in plugin and standalone install. CI no longer renames the directory on sync (the old `caveman-compress/` → `skills/compress/` sed rename is gone now that the source lives at `skills/caveman-compress/`).

### caveman-commit / caveman-review

Independent skills in `skills/caveman-commit/SKILL.md` and `skills/caveman-review/SKILL.md`. Both have own `description` and `name` frontmatter so they load independently. caveman-commit: Conventional Commits, ≤50 char subject. caveman-review: one-line comments in `L<line>: <severity> <problem>. <fix>.` format.

---

## Agent distribution

How caveman reaches each agent type:

| Agent | Mechanism | Auto-activates? |
|-------|-----------|----------------|
| Claude Code | Native Rust binary (`caveman`) wired as SessionStart + UserPromptSubmit hooks + statusline, via `bash install.sh` or the plugin | Yes — SessionStart hook injects rules |
| Codex | Plugin in `plugins/caveman/` plus repo `.codex/hooks.json` (inline `echo` ruleset, no Node) and `.codex/config.toml` | Yes on macOS/Linux — SessionStart hook |
| Gemini CLI | Extension with `GEMINI.md` context file | Yes — context file loads every session |
| Cursor | `npx skills add ... -a cursor` writes the upstream skill profile; per-repo `.cursor/rules/caveman.mdc` via `caveman init --only cursor` | Yes — always-on rule |
| Windsurf | `npx skills add ... -a windsurf`; per-repo `.windsurf/rules/caveman.md` via `caveman init --only windsurf` | Yes — always-on rule |
| Cline | `npx skills add ... -a cline`; per-repo `.clinerules/caveman.md` via `caveman init --only cline` | Yes — Cline auto-discovers `.clinerules/` |
| Copilot | `npx skills add ... -a github-copilot`; per-repo `.github/copilot-instructions.md` via `caveman init --only copilot` | Yes — repo-wide instructions |
| Others (Junie, Trae, Warp, Tabnine, Mistral, Qwen, Devin, Droid, ForgeCode, Bob, Crush, iFlow, OpenHands, Qoder, Rovo Dev, Replit, Antigravity, …) | `npx skills add JuliusBrussee/caveman -a <profile>` | No — user must say `/caveman` each session |

> **Removed in the Rust port:** the opencode native plugin (`src/plugins/opencode/`) and the OpenClaw install helper (`bin/lib/openclaw.js`) were Node/Bun and have been deleted along with the rest of the Node footprint. `src/rules/caveman-openclaw-bootstrap.md` is retained as the source of truth if OpenClaw support is re-added (as Rust). The repo is now Claude-Code-focused; other agents reach caveman through the external `npx skills` CLI (not our code) and `caveman init`.

For agents without hook systems, the always-on rule body lives in `src/rules/caveman-activate.md` — `caveman init` writes it into each per-repo location.

**Adding a new per-repo rule target.** Add an `Agent` entry to the `agents()` list in `rust/src/init.rs` (`id`, `file`, `frontmatter`, `Mode::Replace`/`Append`). The shared rule body comes from `RULE_BODY_RAW` (embedded from `src/rules/caveman-activate.md`). For agents that install through the upstream skills CLI, just add a row to `INSTALL.md` with the correct vercel-labs/skills profile slug.

---

## Evals

`evals/` has three-arm harness:
- `__baseline__` — no system prompt
- `__terse__` — `Answer concisely.`
- `<skill>` — `Answer concisely.\n\n{SKILL.md}`

Honest delta = **skill vs terse**, not skill vs baseline. Baseline comparison conflates skill with generic terseness — that cheating. Harness designed to prevent this.

`llm_run.py` calls `claude -p --system-prompt ...` per (prompt, arm), saves to `evals/snapshots/results.json`. `measure.py` reads snapshot offline with tiktoken (OpenAI BPE — approximates Claude tokenizer, ratios meaningful, absolute numbers approximate).

Add skill: drop `skills/<name>/SKILL.md`. Harness auto-discovers. Add prompt: append line to `evals/prompts/en.txt`.

Snapshots committed to git. CI reads without API calls. Only regenerate when SKILL.md or prompts change.

---

## Benchmarks

`benchmarks/` runs real prompts through Claude API (not Claude Code CLI), records raw token counts. Results committed as JSON in `benchmarks/results/`. Benchmark table in README generated from results — update when regenerating.

To reproduce: `uv run python benchmarks/run.py` (needs `ANTHROPIC_API_KEY` in `.env.local`).

---

## Key rules for agents working here

- Edit `skills/<name>/SKILL.md` for behavior changes. Never edit synced copies under `plugins/caveman/skills/`.
- Edit `src/rules/caveman-activate.md` for auto-activation rule changes. Never edit any per-agent rule copy a user has on their machine.
- The whole Claude Code runtime is the Rust binary in `rust/`. Edit `rust/src/*.rs` for hook/installer behavior, then rebuild (`cargo build --release`) and run `cargo test`. There is **no Node** in the Claude Code path — do not re-introduce `.js` hooks.
- Edit `src/rules/caveman-activate.md` for the always-on rule body; it's embedded into the binary via `include_str!` in `rust/src/init.rs`, so rebuild to propagate.
- Per-skill human docs live in `skills/<name>/README.md`. The LLM-facing body is in `SKILL.md`. Don't merge them — different audiences.
- Build artifacts go in `dist/` (CI) and `rust/target/` (cargo) — both gitignored. `rust/Cargo.lock` IS committed (binary crate).
- README most important file for user-facing impact. Optimize for non-technical readers. Preserve caveman voice.
- `INSTALL.md` is the per-agent install reference. Keep the install table in `README.md` short and link out to `INSTALL.md` for the full matrix.
- Benchmark and eval numbers must be real. Never fabricate or estimate.
- CI workflow commits back to main after merge. Account for when checking branch state.
- Hook subcommands must silent-fail on all filesystem errors. Never let a hook crash block session start.
- Any new flag-file write must go through `safe_write_flag()` in `rust/src/config.rs`. A plain `fs::write` on a predictable user-owned path reopens the symlink-clobber attack surface.
- All subcommands must respect `CLAUDE_CONFIG_DIR`, not hardcode `~/.claude` (use `config::claude_dir()`).
- `install.sh` / `install.ps1` only build with cargo and call `caveman install`. Keep settings.json edits in `rust/src/settings.rs` (JSONC-tolerant read, backup before write) — never shell out to another runtime to edit JSON.
