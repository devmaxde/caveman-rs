// caveman — statusline badge (port of caveman-statusline.sh).
// Reads the mode flag, prints a colored [CAVEMAN] / [CAVEMAN:<MODE>] badge,
// then appends the lifetime-savings suffix. Symlink-refusing, byte-capped.

use crate::config;
use std::io::{Read, Write};

const ORANGE: &str = "\x1b[38;5;172m";
const RESET: &str = "\x1b[0m";

pub fn run() {
    let claude_dir = config::claude_dir();
    let flag = claude_dir.join(".caveman-active");

    // read_flag is symlink-safe, size-capped, and whitelist-validated.
    let mode = match config::read_flag(&flag) {
        Some(m) => m,
        None => return, // missing / invalid / symlink → render nothing
    };

    let mut out = String::new();
    if mode == "full" {
        out.push_str(&format!("{}[CAVEMAN]{}", ORANGE, RESET));
    } else {
        out.push_str(&format!("{}[CAVEMAN:{}]{}", ORANGE, mode.to_uppercase(), RESET));
    }

    // Savings suffix — on by default, opt out with CAVEMAN_STATUSLINE_SAVINGS=0.
    let savings_on = std::env::var("CAVEMAN_STATUSLINE_SAVINGS")
        .map(|v| v != "0")
        .unwrap_or(true);
    if savings_on {
        if let Some(suffix) = read_suffix(&claude_dir.join(".caveman-statusline-suffix")) {
            if !suffix.is_empty() {
                out.push_str(&format!(" {}{}{}", ORANGE, suffix, RESET));
            }
        }
    }

    print!("{}", out);
    let _ = std::io::stdout().flush();
}

/// Read the suffix file: refuse symlinks, cap at 64 bytes, strip control bytes
/// (< 0x20) to block ANSI-escape injection. Control bytes never appear inside
/// UTF-8 multibyte sequences, so byte-level filtering is safe.
fn read_suffix(path: &std::path::Path) -> Option<String> {
    let st = std::fs::symlink_metadata(path).ok()?;
    if st.file_type().is_symlink() || !st.is_file() {
        return None;
    }
    let mut f = std::fs::File::open(path).ok()?;
    let mut buf = [0u8; 64];
    let n = f.read(&mut buf).ok()?;
    let filtered: Vec<u8> = buf[..n].iter().copied().filter(|&b| b >= 0x20).collect();
    Some(String::from_utf8_lossy(&filtered).to_string())
}
