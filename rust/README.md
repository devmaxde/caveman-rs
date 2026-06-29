# caveman (native Rust binary)

This crate is the whole runtime for caveman on Claude Code. **No Node.** One
binary, `caveman`, replaces the former Node hooks, the shell statusline, and the
`node -e` settings.json editing the installer used to do.

## Build & install

Requires the Rust toolchain (`cargo`). Install once from <https://rustup.rs>.

```bash
bash install.sh            # from the repo root: builds release + wires hooks
bash install.sh --force    # rebuild + re-wire over an existing install
bash install.sh --uninstall
```

`install.sh` runs `cargo build --release` then `caveman install`, which copies
the binary to `$CLAUDE_CONFIG_DIR/hooks/caveman` (default `~/.claude/hooks/`),
merges the hooks + statusline into `settings.json`, and extracts the embedded
skills into `$CLAUDE_CONFIG_DIR/skills/` so `/caveman` & friends are registered.

The `skills/` and `agents/` trees are **baked into the binary** at build time
(`include_dir!`), so the compiled `caveman` is fully self-contained: copy it to
any machine and `caveman install` works with no repo present. Rebuild
(`cargo build --release`) after editing any `skills/**` or `agents/*` file.

## Subcommands

| Command | Replaces | Role |
|---------|----------|------|
| `caveman activate` | `caveman-activate.js` | SessionStart hook — write mode flag, emit ruleset (filtered to active level from `skills/caveman/SKILL.md`, else an embedded fallback), nudge statusline setup |
| `caveman mode-tracker` | `caveman-mode-tracker.js` | UserPromptSubmit hook — reads stdin JSON, switches mode on `/caveman …` + natural language, runs `/caveman-stats` in-process, emits per-turn reinforcement |
| `caveman stats [--share] [--all] [--since Nd\|Nh] [--session-file F]` | `caveman-stats.js` | token usage + estimated savings; appends lifetime history + statusline suffix |
| `caveman statusline` | `caveman-statusline.sh/.ps1` | print the `[CAVEMAN]` / `[CAVEMAN:ULTRA]` badge + savings suffix |
| `caveman init [dir] [--dry-run] [--force] [--only <agent>]` | `caveman-init.js` | drop the always-on rule into a repo (cursor/windsurf/cline/copilot/opencode/AGENTS.md) |
| `caveman install [--force]` | `install.sh` + its `node -e` block | copy binary into the config dir, wire `settings.json`, and extract the **embedded** skills (`/caveman`, `/caveman-commit`, …) + cavecrew agents — self-contained, no repo needed |
| `caveman uninstall` | `uninstall.sh` + its `node -e` block | strip caveman hooks/statusline, remove binary + flag |

## Security parity with the old hooks

The flag/history/suffix files keep the original hardening, ported 1:1:

- symlink-refusing reads and writes (`O_NOFOLLOW`), atomic temp+rename, `0600`
- a 64-byte cap + mode whitelist on the flag, control-byte stripping on the
  statusline suffix (blocks ANSI-escape injection)
- parent-dir symlinks allowed only when they resolve to a dir owned by the
  current uid

All hook paths silent-fail on any filesystem error — a hook never blocks a
session.

## Flag files

Same locations and formats as before, honoring `CLAUDE_CONFIG_DIR`:

```
$CLAUDE_CONFIG_DIR/.caveman-active             active mode (statusline reads this)
$CLAUDE_CONFIG_DIR/.caveman-statusline-suffix  pre-rendered savings suffix
$CLAUDE_CONFIG_DIR/.caveman-history.jsonl      lifetime stats log
```

## Tests

```bash
cargo test            # pure-logic unit tests (settings JSONC, stats math, …)
```
