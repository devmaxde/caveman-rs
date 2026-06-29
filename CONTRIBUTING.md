# Contributing to caveman

Thanks for considering a contribution. Caveman is a multi-agent skill that
makes 30+ AI coding agents talk in compressed caveman-style prose. Most
contributions fall into one of three buckets:

1. **Editing skill prose** — change how caveman speaks, what intensity levels do, what slash commands trigger.
2. **Adding a new agent** — add a per-repo rule target in `rust/src/init.rs` or a row to the install tables.
3. **Fixing the hooks or installer** — the native Rust binary in `rust/` (hooks, installer, stats, statusline). Needs the Rust toolchain ([rustup.rs](https://rustup.rs)); no Node.

Caveman like simple. Small focused PR > big rewrite.

---

## Quick orientation

The repo distributes one skill (caveman) plus a handful of sub-skills
(caveman-commit, caveman-review, caveman-compress, cavecrew-*) to many
agents through different distribution mechanisms (Claude Code plugin, Codex
plugin, Gemini extension, Cursor/Windsurf/Cline rule files, `npx skills` for
the long tail). The Claude Code path is a **native Rust binary** (`rust/`) —
no Node. Other agents are reached through the external `npx skills` CLI and
the per-repo rule writer `caveman init`.

Sources of truth live at the **top level** of the repo. Agent-specific
copies live under `plugins/caveman/` and similar mirror dirs — those are
**rebuilt by CI** and edits there are reverted.

---

## What to edit (sources of truth)

| I want to change... | Edit this file |
|---|---|
| Caveman behavior (intensity levels, voice, rules) | `skills/caveman/SKILL.md` |
| Caveman commit-message format | `skills/caveman-commit/SKILL.md` |
| Caveman code-review format | `skills/caveman-review/SKILL.md` |
| Caveman compress logic | `skills/caveman-compress/SKILL.md` and `skills/caveman-compress/scripts/` |
| Caveman quick-reference card | `skills/caveman-help/SKILL.md` |
| Cavecrew decision guide (when to delegate to subagents) | `skills/cavecrew/SKILL.md` |
| cavecrew subagent definitions | `agents/cavecrew-investigator.md`, `agents/cavecrew-builder.md`, `agents/cavecrew-reviewer.md` |
| Auto-activation rule body (Cursor/Windsurf/Cline/Copilot) | `src/rules/caveman-activate.md` (embedded into the Rust binary at build) |
| Add a per-agent rule target for `caveman init` | `rust/src/init.rs` (`agents()` list) |
| Per-repo init logic (drops rule files into a user's repo) | `rust/src/init.rs` |
| Claude Code hooks (activate, mode-tracker, stats, statusline) | `rust/src/{activate,mode_tracker,stats,statusline}.rs` |
| Mode resolution + symlink-safe flag I/O | `rust/src/config.rs` |
| Settings.json read/write helpers + installer | `rust/src/settings.rs`, `rust/src/install.rs` |

That's it. Every other markdown file with `SKILL.md` in the path is a copy.

---

## What NOT to edit (CI-generated mirrors)

Edits to these files are wiped by the next CI run. The
`.github/workflows/sync-skill.yml` job rebuilds them from the sources above
on every push to `main`.

| Path | Rebuilt from |
|------|--------------|
| `plugins/caveman/skills/caveman/SKILL.md` | `skills/caveman/SKILL.md` |
| `plugins/caveman/skills/caveman-compress/{SKILL.md, scripts/}` | `skills/caveman-compress/{SKILL.md, scripts/}` |
| `plugins/caveman/skills/cavecrew/SKILL.md` | `skills/cavecrew/SKILL.md` |
| `plugins/caveman/agents/cavecrew-*.md` | `agents/cavecrew-*.md` |
| `dist/caveman.skill` | ZIP of `skills/caveman/` (gitignored; rebuilt by CI on each push to `main`) |

`caveman-commit`, `caveman-review`, `caveman-help`, and `caveman-stats` are **not** mirrored under `plugins/caveman/skills/` by CI. Claude Code reaches them through the standalone hook + skill install path and `npx skills` carries them to other agents. If you see `plugins/caveman/skills/caveman-stats/` checked in, treat it as a legacy hand-committed copy — the workflow in `.github/workflows/sync-skill.yml` does not touch it.

When in doubt: if the file lives under `plugins/`, `dist/`, or any agent
dotdir mirror, it's a build artifact. Edit the top-level source instead.

---

## Adding a new agent

There's no unified installer anymore — the Claude Code path is the Rust binary,
and other agents install through the external `npx skills` CLI. To add an agent:

1. Confirm the agent has a distribution path. Either:
   - it has a profile slug in upstream [vercel-labs/skills](https://github.com/vercel-labs/skills) (most common), or
   - it has a native plugin / extension / rule-file mechanism we can target.
2. If it reads a per-repo rule file, add an `Agent` entry to the `agents()` list
   in `rust/src/init.rs` (`id`, `file`, `frontmatter`, `Mode::Replace`/`Append`),
   then `cargo test` / `cargo build`.
3. Add a row to the install tables in `README.md` and `INSTALL.md` with the
   correct `npx skills add ... -a <profile>` slug (or `caveman init --only <id>`).

Bad slug? `npx skills add` fails at install **runtime**, not before. Always
verify the slug against the vercel-labs/skills README before merging.

---

## Adding a new skill

1. Create `skills/<name>/SKILL.md` with frontmatter:
   ```yaml
   ---
   name: <name>
   description: <one sentence, present tense>
   ---
   ```
2. Create `skills/<name>/README.md` — human-facing summary, install hint, example.
3. Add `skills/<name>/scripts/` if the skill ships helpers (Python or Node).
4. If the skill should be in the Claude Code plugin, add a sync step to `.github/workflows/sync-skill.yml` so CI mirrors it into `plugins/caveman/skills/<name>/`.
5. If it's user-invocable as a slash command, add a row to the slash-command table in `README.md` and `INSTALL.md`.
6. Add an eval prompt to `evals/prompts/en.txt` if you want the eval harness to score it.

---

## Running tests

```bash
# Rust unit tests (settings JSONC, hook merge, stats math, flag I/O)
cargo test --manifest-path rust/Cargo.toml

# Compress-skill safety tests (Python)
python3 -m unittest tests.test_compress_safety
```

If any test depends on a network or
external SDK, it must skip cleanly when the dependency is missing — never
gate the whole suite on optional creds.

---

## Running benchmarks and evals

Benchmarks hit the real Claude API and record raw token counts:

```bash
uv run python benchmarks/run.py     # needs ANTHROPIC_API_KEY in .env.local
```

Evals are a three-arm offline harness (`__baseline__`, `__terse__`, each skill):

```bash
python evals/llm_run.py             # regenerates evals/snapshots/results.json
python evals/measure.py             # reads snapshot, prints token deltas
```

Snapshots are committed to git. Only regenerate when a `SKILL.md` or
`evals/prompts/en.txt` changes. Numbers in `README.md` and any docs come from
real runs — never invent or round.

---

## Pull-request guidelines

- **Conventional Commits** for the commit subject. See `skills/caveman-commit/SKILL.md` for the format we use here.
- **One concern per PR.** A README copy-edit and an installer fix go in separate PRs.
- **Rebuild after Rust changes.** `cargo build --release --manifest-path rust/Cargo.toml` and `cargo test` must pass.
- **Show before/after** for prose changes to any `SKILL.md`. One sentence on why the new wording is better.
- **Mention the CI sync.** If you edited a source-of-truth file, note it: "CI will resync `plugins/caveman/skills/...` on merge."

PR descriptions don't need to be long. Caveman style fine. Just say what change, why.

---

## Code style

A handful of invariants that have bitten us before. Keep them.

- **Hook subcommands must silent-fail on filesystem errors.** A hook that panics blocks Claude Code session start — that's user-facing breakage. See the `let _ = (|| -> Option<()> {...})()` patterns in `rust/src/config.rs` and the silent stdin handling in `rust/src/mode_tracker.rs`.
- **Settings.json reads/writes go through `rust/src/settings.rs`.** `read_settings` tolerates JSONC comments + trailing commas. It backs up to `settings.json.bak` before any write.
- **Symlink-safe flag writes via `safe_write_flag()`** in `rust/src/config.rs`. The flag lives at a predictable path under `$CLAUDE_CONFIG_DIR/`; without `O_NOFOLLOW` and a parent-symlink + uid check, a local attacker can clobber any file the user can write.
- **Honor `CLAUDE_CONFIG_DIR`.** Use `config::claude_dir()` — never hardcode `~/.claude`.
- **`install.sh` / `install.ps1` only build with cargo and call `caveman install`.** Don't shell out to another runtime to edit `settings.json`; that logic lives in `rust/src/settings.rs`.

---

## Ideas

See [issues labeled `good first issue`](../../issues?q=label%3A%22good+first+issue%22)
for starter tasks. Or grep `TODO` / `FIXME` in `rust/src/` —
each one is a real lead.

Caveman like contribution. You bring rock, caveman put rock in pile. Pile
get bigger. Brain still big.
