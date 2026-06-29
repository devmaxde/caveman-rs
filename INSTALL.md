# Install caveman

One install. Works for every AI coding agent on your machine.

If just want it to work, run the one-liner. If want to know what gets touched, scroll down.

## Claude Code (native Rust — no Node)

The Claude Code path is pure Rust. One binary does the SessionStart hook, the
prompt hook, the statusline, stats, and the installer itself. **No Node ever runs.**

Needs the Rust toolchain once — grab it from [rustup.rs](https://rustup.rs).

**macOS / Linux / WSL / Git Bash**

```bash
git clone https://github.com/JuliusBrussee/caveman
cd caveman
bash install.sh           # cargo build --release, then wire hooks + statusline
```

**Windows (PowerShell 5.1+)**

```powershell
git clone https://github.com/JuliusBrussee/caveman
cd caveman
pwsh install.ps1
```

What it does:

- Builds the native `caveman` binary with `cargo build --release`.
- Copies it to `$CLAUDE_CONFIG_DIR/hooks/caveman` (default `~/.claude/hooks/`).
- Merges the SessionStart + UserPromptSubmit hooks and the statusline badge into `settings.json` (JSONC-tolerant, idempotent, backs up first).
- Registers the slash commands by copying the skills into `$CLAUDE_CONFIG_DIR/skills/` — `/caveman`, `/caveman-commit`, `/caveman-review`, `/caveman-stats`, `/caveman-compress`, `/caveman-help`, `/cavecrew` (so Claude Code recognizes them — no "Unknown command"). cavecrew subagents go into `$CLAUDE_CONFIG_DIR/agents/`.
- Safe to re-run. `--force` rebuilds and re-wires. `bash install.sh --uninstall` removes everything.

Prefer the Claude Code plugin? `claude plugin marketplace add JuliusBrussee/caveman && claude plugin install caveman@caveman`. The plugin hooks build the Rust binary on first session (Rust must be installed) — still no Node.

### No Rust? Install from a prebuilt binary

Every release ships a statically-linked Linux x86_64 `caveman` binary. It needs **no Rust, no build, no repo checkout** — the binary bakes in every skill and agent. Install it on any number of machines:

```bash
curl -fsSL https://raw.githubusercontent.com/JuliusBrussee/caveman/main/install-release.sh | bash
```

Or pin a version / re-wire an existing install:

```bash
bash install-release.sh v0.2.0          # specific release tag
bash install-release.sh latest --force  # re-wire over an existing install
```

The script downloads the binary, verifies its SHA-256 checksum, and runs `caveman install` (copies it to `$CLAUDE_CONFIG_DIR/hooks/caveman`, wires the hooks + statusline, registers the slash commands). Uninstall the same as source installs: `caveman uninstall`.

Prefer to grab it by hand? Download `caveman-x86_64-unknown-linux-musl` from the [latest release](https://github.com/JuliusBrussee/caveman/releases/latest), `chmod +x` it, and run `./caveman-x86_64-unknown-linux-musl install`.

## Other agents

The agents below install through the upstream [`skills`](https://github.com/vercel-labs/skills) CLI (`npx skills add`) or their own native mechanisms. These are external tools — only the Claude Code path above is the native Rust binary.

| Agent | Install command | Auto-activates? |
|---|---|:-:|
| **Gemini CLI** | `gemini extensions install https://github.com/JuliusBrussee/caveman` | Yes |
| **Codex CLI** | `npx skills add JuliusBrussee/caveman -a codex` | Per-session: `/caveman` |
| **Cursor** | `npx skills add JuliusBrussee/caveman -a cursor` | Per-session by default; `caveman init --only cursor` for an always-on rule file |
| **Windsurf** | `npx skills add JuliusBrussee/caveman -a windsurf` | Per-session by default; `caveman init --only windsurf` for an always-on rule file |
| **Cline** | `npx skills add JuliusBrussee/caveman -a cline` | Per-session by default; `caveman init --only cline` for an always-on rule file |
| **GitHub Copilot** *(soft probe)* | `npx skills add JuliusBrussee/caveman -a github-copilot` | Repo-wide instructions via `caveman init --only copilot` |
| **Continue** | `npx skills add JuliusBrussee/caveman -a continue` | No — say `/caveman` |
| **Kilo Code** | `npx skills add JuliusBrussee/caveman -a kilo` | No |
| **Roo Code** | `npx skills add JuliusBrussee/caveman -a roo` | No |
| **Augment Code** | `npx skills add JuliusBrussee/caveman -a augment` | No |
| **Aider Desk** | `npx skills add JuliusBrussee/caveman -a aider-desk` | No |
| **Sourcegraph Amp** | `npx skills add JuliusBrussee/caveman -a amp` | No |
| **IBM Bob** | `npx skills add JuliusBrussee/caveman -a bob` | No |
| **Crush** | `npx skills add JuliusBrussee/caveman -a crush` | No |
| **Devin (terminal)** | `npx skills add JuliusBrussee/caveman -a devin` | No |
| **Droid (Factory)** | `npx skills add JuliusBrussee/caveman -a droid` | No |
| **ForgeCode** | `npx skills add JuliusBrussee/caveman -a forgecode` | No |
| **Block Goose** | `npx skills add JuliusBrussee/caveman -a goose` | No |
| **iFlow CLI** | `npx skills add JuliusBrussee/caveman -a iflow-cli` | No |
| **Kiro CLI** | `npx skills add JuliusBrussee/caveman -a kiro-cli` | No |
| **Mistral Vibe** | `npx skills add JuliusBrussee/caveman -a mistral-vibe` | No |
| **OpenHands** | `npx skills add JuliusBrussee/caveman -a openhands` | No |
| **Qwen Code** | `npx skills add JuliusBrussee/caveman -a qwen-code` | No |
| **Atlassian Rovo Dev** | `npx skills add JuliusBrussee/caveman -a rovodev` | No |
| **Tabnine CLI** | `npx skills add JuliusBrussee/caveman -a tabnine-cli` | No |
| **Trae** | `npx skills add JuliusBrussee/caveman -a trae` | No |
| **Warp** | `npx skills add JuliusBrussee/caveman -a warp` | No |
| **Replit Agent** | `npx skills add JuliusBrussee/caveman -a replit` | No |
| **JetBrains Junie** *(soft probe)* | `npx skills add JuliusBrussee/caveman -a junie` | No |
| **Qoder** *(soft probe)* | `npx skills add JuliusBrussee/caveman -a qoder` | No |
| **Google Antigravity** *(soft probe)* | `npx skills add JuliusBrussee/caveman -a antigravity` | No |

"Soft probe" = installer won't auto-detect these without `--only <id>` because there's no reliable always-on signal (Copilot subscription state is auth-gated; the others have no CLI / config-dir-only). Pass the flag when you want them.

For "auto-activates? No" agents, type `/caveman` once per session (or use natural-language triggers like "talk like caveman", "caveman mode").

**Finding a profile slug for `npx skills add ... -a <profile>`?** Read the table above, or browse the live list at [vercel-labs/skills](https://github.com/vercel-labs/skills). The slug must exist upstream.

## The `caveman` binary

The Rust binary installed for Claude Code is also a normal CLI. Run it from a clone at `rust/target/release/caveman`, or after install at `$CLAUDE_CONFIG_DIR/hooks/caveman` (default `~/.claude/hooks/caveman`):

| Command | What |
|---|---|
| `caveman install [--force]` | Copy the binary into the Claude config dir, wire the hooks + statusline into `settings.json`, and **extract the embedded slash-command skills** into `$CLAUDE_CONFIG_DIR/skills/` (+ cavecrew agents into `agents/`). Fully self-contained — the skills are baked into the binary, so this works even when the binary is copied somewhere with no repo nearby. |
| `caveman uninstall` | Remove caveman hooks + statusline from `settings.json`, the registered skills/agents, the binary, and the flag file. |
| `caveman init [dir] [--dry-run] [--force] [--only <agent>]` | Drop the always-on rule file into a repo for Cursor / Windsurf / Cline / Copilot / opencode / `AGENTS.md`. |
| `caveman stats [--share] [--all] [--since Nd\|Nh]` | Print token usage + estimated savings. |
| `caveman statusline` | Print the `[CAVEMAN]` badge (the statusline calls this). |
| `caveman activate` / `caveman mode-tracker` | The SessionStart / UserPromptSubmit hooks. You don't call these by hand — `settings.json` does. |

All paths honor `CLAUDE_CONFIG_DIR`.

## Always-on rules (other agents)

For agents without a hook system (Cursor, Windsurf, Cline, Copilot, opencode), the always-on path is a static rule file. The Rust binary writes them:

```bash
caveman init                  # all supported agents, into $PWD
caveman init --only cursor    # one agent
caveman init --dry-run        # preview, write nothing
```

`caveman init` writes the rule into every supported per-agent location (`.cursor/rules/`, `.windsurf/rules/`, `.clinerules/`, `.github/copilot-instructions.md`, `.opencode/AGENTS.md`, `AGENTS.md`). The rule body is embedded at build time from the single source [`src/rules/caveman-activate.md`](src/rules/caveman-activate.md).

## Verify

After install, three quick checks:

**1. Confirm the binary is wired in.**

```bash
"${CLAUDE_CONFIG_DIR:-$HOME/.claude}/hooks/caveman" --version
```

Prints `caveman <version>`. If the file is missing, the build or `caveman install` step didn't complete — re-run `bash install.sh --force`.

**2. Talk to Claude Code.**

Open Claude Code, type `/caveman`. Response should be terse fragments — "Got it. Caveman mode on." or similar. Try a real question: "What is closures in JS?" — answer should drop articles and read like grunts.

**3. Check the flag file.**

```bash
cat "${CLAUDE_CONFIG_DIR:-$HOME/.claude}/.caveman-active"
# expected output: full
```

If it's missing or empty, the SessionStart hook didn't fire. See troubleshooting below.

Statusline should show `[CAVEMAN]` (orange) at the bottom of Claude Code. After your first `/caveman-stats` run it appends a savings counter like `[CAVEMAN] ⛏ 12.4k`.

## Uninstall

```bash
bash install.sh --uninstall                          # from a clone
# or directly:
"${CLAUDE_CONFIG_DIR:-$HOME/.claude}/hooks/caveman" uninstall
```

What it removes:

- Caveman hook entries from `$CLAUDE_CONFIG_DIR/settings.json` (default `~/.claude/`; matched by the substring `caveman`).
- The caveman statusLine (when it points at the caveman binary).
- The installed binary `$CLAUDE_CONFIG_DIR/hooks/caveman`.
- The `.caveman-active` flag file.

What it does **not** remove:

- The Claude Code plugin (if you installed that way) — `claude plugin disable caveman`.
- The Gemini CLI extension — `gemini extensions uninstall caveman`.
- Skills installed via `npx skills add` — run `npx skills remove caveman` (or use your IDE's skill manager).
- Per-repo rule files written by `caveman init`. Delete by hand if you want.

## Troubleshooting

**"Install script broke. What now?"**

Open your agent in this repo and say:

> "Read CLAUDE.md and INSTALL.md. Install caveman for me."

Agent read repo. Agent run install. Caveman make agent talk less — agent first job is install caveman to talk less. Snake eat tail.

Still broken? [Open an issue](https://github.com/JuliusBrussee/caveman/issues).

**"I ran the installer but Claude Code isn't talking caveman."**

1. Confirm the binary exists: `"$CLAUDE_CONFIG_DIR/hooks/caveman" --version` (default `~/.claude/hooks/caveman`). Missing → re-run `bash install.sh --force`.
2. Open `$CLAUDE_CONFIG_DIR/settings.json` (default `~/.claude/settings.json`) and look for `"hooks"` containing `caveman activate` and `caveman mode-tracker`. If missing, re-run with `--force`.
3. Check `$CLAUDE_CONFIG_DIR/.caveman-active` exists with content `full`. If not, the SessionStart hook silent-failed — run `"$CLAUDE_CONFIG_DIR/hooks/caveman" activate </dev/null` to see if it errors.
4. Restart Claude Code. The SessionStart hook only fires on session start, not mid-session.

**"Cargo not found / build failed."**

- caveman is native Rust now. Install the toolchain once from [rustup.rs](https://rustup.rs), then re-run `bash install.sh`.
- If you just installed Rust, open a new shell or `source "$HOME/.cargo/env"` so `cargo` is on `PATH` (the installer also sources it automatically when present).

**"Hooks failing on Windows."**

- Use `pwsh install.ps1`. It builds `caveman.exe` with cargo and wires the same hooks.
- PowerShell 5.1 minimum. Check with `$PSVersionTable.PSVersion`.

**"My `settings.json` got mangled."**

The binary uses a JSONC-tolerant parser so comments and trailing commas don't crash the merge, and writes a backup at `$CLAUDE_CONFIG_DIR/settings.json.bak` before any change. If something still went wrong:

1. Check for a backup at `$CLAUDE_CONFIG_DIR/settings.json.bak` (installer writes one before any merge).
2. If no backup, restore from your shell history or version control.
3. File an issue with the broken `settings.json` content (redacted) — that file passing validation but breaking Claude Code is a bug we want to fix.

**"I'm in a managed env where I can't install hooks."**

Use the rule-file-only path. Hooks are Claude Code-specific; everything else works via static rule files:

```bash
# Write rule files into the current repo only (no global state, no hooks)
caveman init --only cursor
caveman init --only windsurf
```

This drops `.cursor/rules/caveman.mdc` (and friends) into your repo. No hooks, no global config, nothing outside the repo.

**"`npx skills add` errored on a profile slug."**

The profile slug must exist in [vercel-labs/skills](https://github.com/vercel-labs/skills). If a row in the table above 404s, the upstream profile was renamed or removed — open an issue, we'll update.

## Privacy

The `caveman` binary doesn't phone home. It writes to:

- `$CLAUDE_CONFIG_DIR` (default `~/.claude/`) — the binary, flag file, `settings.json` merge.
- Your current working directory (only with `caveman init`) — repo-local rule files.

No telemetry. No analytics. The binary itself makes no network calls. The one network step is at build time: `cargo build` fetches crate dependencies from [crates.io](https://crates.io) (see `rust/Cargo.toml` / `rust/Cargo.lock`). Other agents' installs go through their own CLIs (`gemini extensions install`, `npx skills add`) which fetch from their registries. Source: [`rust/`](rust/).

---

Stuck? Open an issue: <https://github.com/JuliusBrussee/caveman/issues>
