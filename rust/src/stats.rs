// caveman-stats — read the active Claude Code session log, print real token
// usage plus an estimated savings figure. Port of caveman-stats.js.

use crate::config;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

// Mean per-task savings from benchmarks/results/*.json. Only 'full' has
// measured data; other modes show no estimate until benchmarked.
fn compression(mode: &str) -> Option<f64> {
    match mode {
        "full" => Some(0.65),
        _ => None,
    }
}

// Approximate Anthropic public output-token pricing, USD per million.
// Most-specific prefixes first — first match wins.
const MODEL_OUTPUT_PRICE_PER_M: &[(&str, f64)] = &[
    ("claude-opus-4-0", 75.00),
    ("claude-opus-4-1", 75.00),
    ("claude-opus-4-2025", 75.00),
    ("claude-opus-4", 25.00),
    ("claude-sonnet-4", 15.00),
    ("claude-haiku-4", 5.00),
    ("claude-3-5-sonnet", 15.00),
    ("claude-3-5-haiku", 4.00),
    ("claude-3-opus", 75.00),
];

fn price_for_model(model: Option<&str>) -> Option<f64> {
    let m = model?;
    for (prefix, price) in MODEL_OUTPUT_PRICE_PER_M {
        if m.starts_with(prefix) {
            return Some(*price);
        }
    }
    None
}

fn format_usd(amount: f64) -> String {
    if amount >= 1.0 {
        format!("${:.2}", amount)
    } else if amount >= 0.01 {
        format!("${:.3}", amount)
    } else {
        format!("${:.4}", amount)
    }
}

fn sep() -> String {
    "─".repeat(34)
}

/// Group an integer with thousands separators (mimics Number.toLocaleString()).
fn group_int(n: i64) -> String {
    let neg = n < 0;
    let s = n.abs().to_string();
    let bytes = s.as_bytes();
    let mut out = String::new();
    let len = bytes.len();
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(*b as char);
    }
    if neg {
        format!("-{}", out)
    } else {
        out
    }
}

fn round_half(x: f64) -> i64 {
    x.round() as i64
}

#[derive(Clone)]
struct Parsed {
    output_tokens: i64,
    cache_read_tokens: i64,
    turns: i64,
    model: Option<String>,
}

fn find_recent_session(claude_dir: &Path) -> Option<PathBuf> {
    let projects = claude_dir.join("projects");
    let mut stack: Vec<PathBuf> = std::fs::read_dir(&projects)
        .ok()?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    let mut best: Option<(PathBuf, std::time::SystemTime)> = None;
    while let Some(p) = stack.pop() {
        let md = match std::fs::metadata(&p) {
            Ok(m) => m,
            Err(_) => continue,
        };
        if md.is_dir() {
            if let Ok(rd) = std::fs::read_dir(&p) {
                for child in rd.flatten() {
                    stack.push(child.path());
                }
            }
        } else if p.extension().map(|e| e == "jsonl").unwrap_or(false) {
            if let Ok(mt) = md.modified() {
                if best.as_ref().map(|(_, bm)| mt > *bm).unwrap_or(true) {
                    best = Some((p.clone(), mt));
                }
            }
        }
    }
    best.map(|(p, _)| p)
}

fn parse_session(path: &Path) -> Parsed {
    let raw = match std::fs::read_to_string(path) {
        Ok(r) => r,
        Err(_) => {
            return Parsed {
                output_tokens: 0,
                cache_read_tokens: 0,
                turns: 0,
                model: None,
            }
        }
    };
    let mut output_tokens = 0i64;
    let mut cache_read_tokens = 0i64;
    let mut turns = 0i64;
    let mut model: Option<String> = None;
    for line in raw.split('\n') {
        if line.trim().is_empty() {
            continue;
        }
        let entry: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if entry.get("type").and_then(|t| t.as_str()) != Some("assistant") {
            continue;
        }
        let msg = match entry.get("message") {
            Some(m) if m.is_object() => m,
            _ => continue,
        };
        let usage = match msg.get("usage") {
            Some(u) if u.is_object() => u,
            _ => continue,
        };
        output_tokens += usage.get("output_tokens").and_then(|v| v.as_i64()).unwrap_or(0);
        cache_read_tokens += usage
            .get("cache_read_input_tokens")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        turns += 1;
        if model.is_none() {
            if let Some(m) = msg.get("model").and_then(|v| v.as_str()) {
                model = Some(m.to_string());
            }
        }
    }
    Parsed {
        output_tokens,
        cache_read_tokens,
        turns,
        model,
    }
}

struct CompressedPair {
    original_size: u64,
    compressed_size: u64,
}

fn find_compressed_pairs(dirs: &[PathBuf]) -> Vec<CompressedPair> {
    let mut pairs = Vec::new();
    for dir in dirs {
        let rd = match std::fs::read_dir(dir) {
            Ok(r) => r,
            Err(_) => continue,
        };
        for entry in rd.flatten() {
            let ft = match entry.file_type() {
                Ok(f) => f,
                Err(_) => continue,
            };
            if !ft.is_file() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.ends_with(".original.md") {
                continue;
            }
            let base = &name[..name.len() - ".original.md".len()];
            let original_path = dir.join(&name);
            let compressed_path = dir.join(format!("{}.md", base));
            let o_size = match std::fs::metadata(&original_path) {
                Ok(m) => m.len(),
                Err(_) => continue,
            };
            let c_size = match std::fs::metadata(&compressed_path) {
                Ok(m) => m.len(),
                Err(_) => continue,
            };
            if o_size <= c_size {
                continue;
            }
            pairs.push(CompressedPair {
                original_size: o_size,
                compressed_size: c_size,
            });
        }
    }
    pairs
}

struct CompressedSummary {
    count: usize,
    tokens_saved: i64,
}

fn summarize_compressed(pairs: &[CompressedPair]) -> Option<CompressedSummary> {
    if pairs.is_empty() {
        return None;
    }
    let total_original: u64 = pairs.iter().map(|p| p.original_size).sum();
    let total_compressed: u64 = pairs.iter().map(|p| p.compressed_size).sum();
    let bytes_saved = total_original as i64 - total_compressed as i64;
    let tokens_saved = round_half(bytes_saved as f64 / 4.0);
    Some(CompressedSummary {
        count: pairs.len(),
        tokens_saved,
    })
}

fn derive_savings(output_tokens: i64, mode: Option<&str>, model: Option<&str>) -> (i64, f64) {
    let ratio = mode.and_then(compression);
    let price = price_for_model(model);
    let ratio = match ratio {
        Some(r) => r,
        None => return (0, 0.0),
    };
    let est_normal = round_half(output_tokens as f64 / (1.0 - ratio));
    let est_saved_tokens = est_normal - output_tokens;
    let est_saved_usd = match price {
        Some(p) => (est_saved_tokens as f64 / 1_000_000.0) * p,
        None => 0.0,
    };
    (est_saved_tokens, est_saved_usd)
}

fn parse_duration(spec: Option<&str>) -> Option<u64> {
    let s = spec?.trim();
    let re = regex::Regex::new(r"^(\d+)([dh])$").unwrap();
    let caps = re.captures(s)?;
    let n: u64 = caps[1].parse().ok()?;
    Some(if &caps[2] == "d" {
        n * 86_400_000
    } else {
        n * 3_600_000
    })
}

struct HistoryAgg {
    sessions: usize,
    output_tokens: i64,
    est_saved_tokens: i64,
    est_saved_usd: f64,
}

fn aggregate_history(history_path: &Path, since_ms: Option<u64>) -> HistoryAgg {
    let lines = config::read_history(history_path);
    let cutoff = since_ms.map(|ms| config::now_millis_u64().saturating_sub(ms));
    let mut latest: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
    for line in &lines {
        let entry: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if !entry.is_object() {
            continue;
        }
        let ts = entry.get("ts").and_then(|v| v.as_u64()).unwrap_or(0);
        if let Some(c) = cutoff {
            if ts < c {
                continue;
            }
        }
        let id = entry
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("_")
            .to_string();
        let keep = match latest.get(&id) {
            Some(prev) => {
                let prev_ts = prev.get("ts").and_then(|v| v.as_u64()).unwrap_or(0);
                ts >= prev_ts
            }
            None => true,
        };
        if keep {
            latest.insert(id, entry);
        }
    }
    let mut output_tokens = 0i64;
    let mut est_saved_tokens = 0i64;
    let mut est_saved_usd = 0.0f64;
    for e in latest.values() {
        output_tokens += e.get("output_tokens").and_then(|v| v.as_i64()).unwrap_or(0);
        est_saved_tokens += e
            .get("est_saved_tokens")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        est_saved_usd += e.get("est_saved_usd").and_then(|v| v.as_f64()).unwrap_or(0.0);
    }
    HistoryAgg {
        sessions: latest.len(),
        output_tokens,
        est_saved_tokens,
        est_saved_usd,
    }
}

fn humanize_tokens(n: i64) -> String {
    if n <= 0 {
        return "0".to_string();
    }
    let f = n as f64;
    if f >= 1e6 {
        format!("{:.1}M", f / 1e6)
    } else if f >= 1e3 {
        format!("{:.1}k", f / 1e3)
    } else {
        round_half(f).to_string()
    }
}

fn format_history(agg: &HistoryAgg, since: Option<&str>) -> String {
    let s = sep();
    let window = since.map(|w| format!(" (last {})", w)).unwrap_or_default();
    if agg.sessions == 0 {
        return format!(
            "\nCaveman Stats — Lifetime{window}\n{s}\nNo sessions logged yet — run /caveman-stats inside any session to start tracking.\n{s}\n",
            window = window,
            s = s
        );
    }
    let usd_line = if agg.est_saved_usd > 0.0 {
        format!("Est. saved (USD):      ~{}\n", format_usd(agg.est_saved_usd))
    } else {
        String::new()
    };
    format!(
        "\nCaveman Stats — Lifetime{window}\n{s}\nSessions:   {sessions}\n{s}\nOutput tokens:         {ot}\nEst. tokens saved:     {est}\n{usd}{s}\n",
        window = window,
        s = s,
        sessions = group_int(agg.sessions as i64),
        ot = group_int(agg.output_tokens),
        est = group_int(agg.est_saved_tokens),
        usd = usd_line
    )
}

fn format_share(p: &Parsed, mode: Option<&str>) -> String {
    if p.turns == 0 {
        return "🪨 caveman armed but no turns yet — caveman.sh".to_string();
    }
    let ratio = mode.and_then(compression);
    let price = price_for_model(p.model.as_deref());
    if let Some(r) = ratio {
        let est_saved = round_half(p.output_tokens as f64 / (1.0 - r)) - p.output_tokens;
        let usd = match price {
            Some(pr) => {
                let amt = (est_saved as f64 / 1_000_000.0) * pr;
                format!(" (~{})", format_usd(amt))
            }
            None => String::new(),
        };
        format!(
            "🪨 Saved {} output tokens{} across {} turns this session — caveman.sh",
            group_int(est_saved),
            usd,
            p.turns
        )
    } else {
        format!(
            "🪨 {} turns, {} output tokens this session — caveman.sh",
            p.turns,
            group_int(p.output_tokens)
        )
    }
}

fn format_stats(
    p: &Parsed,
    mode: Option<&str>,
    session_path: &str,
    compressed: Option<&CompressedSummary>,
) -> String {
    let s = sep();
    let short_path = if session_path.len() > 45 {
        format!("...{}", &session_path[session_path.len() - 45..])
    } else {
        session_path.to_string()
    };

    if p.turns == 0 {
        return format!(
            "\nCaveman Stats\n{s}\nNo conversation yet — stats available after first response.\n{s}\n",
            s = s
        );
    }

    let ratio = mode.and_then(compression);
    let price = price_for_model(p.model.as_deref());

    let savings: String;
    let mut footer = String::new();
    if let Some(r) = ratio {
        let est_normal = round_half(p.output_tokens as f64 / (1.0 - r));
        let est_saved = est_normal - p.output_tokens;
        let usd_line = if let Some(pr) = price {
            let usd = (est_saved as f64 / 1_000_000.0) * pr;
            footer = format!(
                "Savings est. from benchmarks/ (mean per-task). Pricing for {}. Actual varies by task.",
                p.model.as_deref().unwrap_or("")
            );
            format!("Est. saved (USD):      ~{}\n", format_usd(usd))
        } else {
            footer = "Savings est. from benchmarks/ (mean per-task). Actual varies by task.".to_string();
            String::new()
        };
        savings = format!(
            "Est. without caveman:  {}\nEst. tokens saved:     {} (~{}%)\n{}",
            group_int(est_normal),
            group_int(est_saved),
            round_half(r * 100.0),
            usd_line.strip_suffix('\n').unwrap_or(&usd_line)
        );
    } else if mode.map(|m| m != "off").unwrap_or(false) {
        savings = format!(
            "No savings estimate for '{}' mode — only 'full' has benchmark data.",
            mode.unwrap()
        );
    } else {
        savings = "Caveman not active this session.".to_string();
    }

    let mut memory_line = String::new();
    if let Some(c) = compressed {
        if c.count > 0 {
            let plural = if c.count == 1 { "" } else { "s" };
            memory_line = format!(
                "{s}\nMemory compressed:     {count} file{plural}, ~{tok} tokens saved per session start (approx)\n",
                s = s,
                count = c.count,
                plural = plural,
                tok = group_int(c.tokens_saved)
            );
        }
    }

    let session_line = if short_path.is_empty() {
        String::new()
    } else {
        format!("Session:  {}\n", short_path)
    };
    let footer_line = if footer.is_empty() {
        String::new()
    } else {
        format!("{}\n", footer)
    };

    format!(
        "\nCaveman Stats\n{s}\n{session}Turns:    {turns}\n{s}\nOutput tokens:         {ot}\nCache-read tokens:     {crt}\n{s}\n{savings}\n{memory}{footer}",
        s = s,
        session = session_line,
        turns = p.turns,
        ot = group_int(p.output_tokens),
        crt = group_int(p.cache_read_tokens),
        savings = savings,
        memory = memory_line,
        footer = footer_line
    )
}

/// Compute the stats output text. Ok(text) on success; Err((message, code)) on
/// error (message goes to stderr in CLI mode). Performs the same side effects
/// as the original script (history append + statusline suffix write).
pub fn run_capture(args: &[String]) -> Result<String, (String, i32)> {
    let session_file_arg = arg_value(args, "--session-file");
    let share = args.iter().any(|a| a == "--share");
    let all = args.iter().any(|a| a == "--all");
    let since_arg = arg_value(args, "--since");

    let claude_dir = config::claude_dir();
    let history_path = claude_dir.join(".caveman-history.jsonl");

    // Lifetime aggregation short-circuits before we need a live session.
    if all || since_arg.is_some() {
        let since_ms = parse_duration(since_arg.as_deref());
        if let Some(s) = since_arg.as_deref() {
            if since_ms.is_none() {
                return Err((
                    format!(
                        "caveman-stats: --since takes Nh or Nd (e.g. 7d, 24h), got: {}",
                        s
                    ),
                    2,
                ));
            }
        }
        let agg = aggregate_history(&history_path, since_ms);
        return Ok(format_history(&agg, since_arg.as_deref()));
    }

    let session_file = match session_file_arg {
        Some(s) => PathBuf::from(s),
        None => match find_recent_session(&claude_dir) {
            Some(s) => s,
            None => return Err(("caveman-stats: no Claude Code session found.".to_string(), 1)),
        },
    };

    let parsed = parse_session(&session_file);
    let flag = claude_dir.join(".caveman-active");
    let mode = config::read_flag(&flag);

    if parsed.turns > 0 {
        let (est_saved_tokens, est_saved_usd) =
            derive_savings(parsed.output_tokens, mode.as_deref(), parsed.model.as_deref());
        let session_id = session_file
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let record = json!({
            "ts": config::now_millis_u64(),
            "session_id": session_id,
            "mode": mode,
            "model": parsed.model,
            "output_tokens": parsed.output_tokens,
            "est_saved_tokens": est_saved_tokens,
            "est_saved_usd": est_saved_usd,
        });
        config::append_flag(&history_path, &record.to_string());

        let agg = aggregate_history(&history_path, None);
        let suffix = if agg.est_saved_tokens > 0 {
            format!("⛏  {}", humanize_tokens(agg.est_saved_tokens))
        } else {
            String::new()
        };
        config::safe_write_flag(&claude_dir.join(".caveman-statusline-suffix"), &suffix);
    }

    if share {
        Ok(format!("{}\n", format_share(&parsed, mode.as_deref())))
    } else {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut scan_dirs = vec![claude_dir.clone()];
        if cwd != claude_dir {
            scan_dirs.push(cwd);
        }
        let compressed = summarize_compressed(&find_compressed_pairs(&scan_dirs));
        Ok(format_stats(
            &parsed,
            mode.as_deref(),
            &session_file.to_string_lossy(),
            compressed.as_ref(),
        ))
    }
}

fn arg_value(args: &[String], key: &str) -> Option<String> {
    let idx = args.iter().position(|a| a == key)?;
    args.get(idx + 1).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_int_matches_locale_string() {
        assert_eq!(group_int(0), "0");
        assert_eq!(group_int(1234), "1,234");
        assert_eq!(group_int(1234567), "1,234,567");
        assert_eq!(group_int(512), "512");
    }

    #[test]
    fn humanize_tokens_thresholds() {
        assert_eq!(humanize_tokens(0), "0");
        assert_eq!(humanize_tokens(-5), "0");
        assert_eq!(humanize_tokens(950), "950");
        assert_eq!(humanize_tokens(2292), "2.3k");
        assert_eq!(humanize_tokens(1_500_000), "1.5M");
    }

    #[test]
    fn format_usd_tiers() {
        assert_eq!(format_usd(1.5), "$1.50");
        assert_eq!(format_usd(0.034), "$0.034");
        assert_eq!(format_usd(0.0004), "$0.0004");
    }

    #[test]
    fn price_prefix_matching_is_order_sensitive() {
        assert_eq!(price_for_model(Some("claude-opus-4-20250514")), Some(75.00));
        assert_eq!(price_for_model(Some("claude-opus-4-5-20251101")), Some(25.00));
        assert_eq!(price_for_model(Some("claude-sonnet-4-20250514")), Some(15.00));
        assert_eq!(price_for_model(Some("unknown-model")), None);
        assert_eq!(price_for_model(None), None);
    }

    #[test]
    fn derive_savings_full_mode() {
        let (tokens, _usd) = derive_savings(1234, Some("full"), Some("claude-sonnet-4-x"));
        assert_eq!(tokens, 2292); // round(1234/0.35) - 1234
        let (z, _) = derive_savings(1234, Some("lite"), Some("claude-sonnet-4-x"));
        assert_eq!(z, 0);
    }

    #[test]
    fn parse_duration_forms() {
        assert_eq!(parse_duration(Some("7d")), Some(7 * 86_400_000));
        assert_eq!(parse_duration(Some("24h")), Some(24 * 3_600_000));
        assert_eq!(parse_duration(Some("bad")), None);
        assert_eq!(parse_duration(None), None);
    }
}

pub fn run(args: &[String]) -> i32 {
    match run_capture(args) {
        Ok(out) => {
            print!("{}", out);
            use std::io::Write;
            let _ = std::io::stdout().flush();
            0
        }
        Err((msg, code)) => {
            eprintln!("{}", msg);
            code
        }
    }
}
