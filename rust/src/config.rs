// caveman — shared configuration resolver (port of src/hooks/caveman-config.js)
//
// Resolution order for default mode:
//   1. CAVEMAN_DEFAULT_MODE environment variable
//   2. Repo-local config (<cwd>/.caveman/config.json or <cwd>/.caveman.json,
//      walking up to the filesystem root)
//   3. User config file defaultMode field
//   4. 'full'

use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub const VALID_MODES: &[&str] = &[
    "off",
    "lite",
    "full",
    "ultra",
    "wenyan-lite",
    "wenyan",
    "wenyan-full",
    "wenyan-ultra",
    "commit",
    "review",
    "compress",
];

pub fn is_valid_mode(s: &str) -> bool {
    VALID_MODES.contains(&s)
}

/// The longest legitimate flag value is "wenyan-ultra" (12 bytes); 64 leaves
/// slack without enabling exfiltration through the predictable flag path.
const MAX_FLAG_BYTES: usize = 64;

fn home_dir() -> PathBuf {
    if let Ok(h) = env::var("HOME") {
        if !h.is_empty() {
            return PathBuf::from(h);
        }
    }
    #[cfg(windows)]
    {
        if let Ok(up) = env::var("USERPROFILE") {
            if !up.is_empty() {
                return PathBuf::from(up);
            }
        }
    }
    PathBuf::from(".")
}

pub fn claude_dir() -> PathBuf {
    if let Ok(d) = env::var("CLAUDE_CONFIG_DIR") {
        if !d.is_empty() {
            return PathBuf::from(d);
        }
    }
    home_dir().join(".claude")
}

pub fn config_dir() -> PathBuf {
    if let Ok(x) = env::var("XDG_CONFIG_HOME") {
        if !x.is_empty() {
            return PathBuf::from(x).join("caveman");
        }
    }
    #[cfg(windows)]
    {
        let appdata = env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home_dir().join("AppData").join("Roaming"));
        return appdata.join("caveman");
    }
    #[allow(unreachable_code)]
    home_dir().join(".config").join("caveman")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

/// Walk up from `start` looking for a repo-local caveman config. Returns the
/// absolute path of the first match, or None. Bounded to 64 levels. Refuses
/// symlinked config files (symmetric with the flag read/write policy).
pub fn find_repo_config_path(start: &Path) -> Option<PathBuf> {
    let mut dir = fs::canonicalize(start).unwrap_or_else(|_| start.to_path_buf());
    let candidates = [".caveman/config.json", ".caveman.json"];
    for _ in 0..64 {
        for rel in &candidates {
            let p = dir.join(rel);
            if let Ok(st) = fs::symlink_metadata(&p) {
                if st.file_type().is_symlink() || !st.is_file() {
                    continue;
                }
                return Some(p);
            }
        }
        match dir.parent() {
            Some(parent) if parent != dir => dir = parent.to_path_buf(),
            _ => return None,
        }
    }
    None
}

fn read_mode_from_config_file(config_path: &Path) -> Option<String> {
    let raw = fs::read_to_string(config_path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let mode = v.get("defaultMode")?.as_str()?.to_lowercase();
    if is_valid_mode(&mode) {
        Some(mode)
    } else {
        None
    }
}

pub fn get_default_mode() -> String {
    // 1. Environment variable (highest priority)
    if let Ok(env_mode) = env::var("CAVEMAN_DEFAULT_MODE") {
        let m = env_mode.to_lowercase();
        if is_valid_mode(&m) {
            return m;
        }
    }

    // 2. Repo-local config
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if let Some(repo_cfg) = find_repo_config_path(&cwd) {
        if let Some(m) = read_mode_from_config_file(&repo_cfg) {
            return m;
        }
    }

    // 3. User config file
    if let Some(m) = read_mode_from_config_file(&config_path()) {
        return m;
    }

    // 4. Default
    "full".to_string()
}

// ---- symlink-safe filesystem helpers -------------------------------------

#[cfg(unix)]
fn current_uid() -> u32 {
    unsafe { libc::getuid() }
}

/// Resolve a directory that may itself be a symlink, verifying ownership.
/// Returns the real directory path, or None if the symlink target is
/// untrusted (not a dir, or owned by another user / outside home).
fn resolve_trusted_dir(dir: &Path) -> Option<PathBuf> {
    let lst = match fs::symlink_metadata(dir) {
        Ok(m) => m,
        Err(_) => return None,
    };
    if !lst.file_type().is_symlink() {
        return Some(dir.to_path_buf());
    }
    let real = fs::canonicalize(dir).ok()?;
    let real_stat = fs::metadata(&real).ok()?;
    if !real_stat.is_dir() {
        return None;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if real_stat.uid() != current_uid() {
            return None;
        }
    }
    #[cfg(not(unix))]
    {
        let home = fs::canonicalize(home_dir()).unwrap_or_else(|_| home_dir());
        let rl = real.to_string_lossy().to_lowercase();
        let hl = home.to_string_lossy().to_lowercase();
        if rl != hl && !rl.starts_with(&(hl + std::path::MAIN_SEPARATOR_STR)) {
            return None;
        }
    }
    Some(real)
}

#[cfg(unix)]
fn open_excl_nofollow(path: &Path) -> std::io::Result<fs::File> {
    use std::os::unix::fs::OpenOptionsExt;
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
}

#[cfg(not(unix))]
fn open_excl_nofollow(path: &Path) -> std::io::Result<fs::File> {
    fs::OpenOptions::new().write(true).create_new(true).open(path)
}

#[cfg(unix)]
fn open_append_nofollow(path: &Path) -> std::io::Result<fs::File> {
    use std::os::unix::fs::OpenOptionsExt;
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
}

#[cfg(not(unix))]
fn open_append_nofollow(path: &Path) -> std::io::Result<fs::File> {
    fs::OpenOptions::new().create(true).append(true).open(path)
}

fn now_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

/// Symlink-safe, atomic flag write (temp + rename, 0600, O_NOFOLLOW). The flag
/// file itself must never be a symlink. Silent-fails on any filesystem error.
pub fn safe_write_flag(flag_path: &Path, content: &str) {
    let _ = (|| -> Option<()> {
        let flag_dir = flag_path.parent()?;
        let _ = fs::create_dir_all(flag_dir);
        let real_dir = resolve_trusted_dir(flag_dir)?;
        let real_flag = real_dir.join(flag_path.file_name()?);

        // The flag file itself must never be a symlink.
        match fs::symlink_metadata(&real_flag) {
            Ok(m) if m.file_type().is_symlink() => return None,
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return None,
        }

        let temp = real_dir.join(format!(
            ".caveman-active.{}.{}",
            std::process::id(),
            now_millis()
        ));
        {
            let mut f = open_excl_nofollow(&temp).ok()?;
            f.write_all(content.as_bytes()).ok()?;
        }
        if fs::rename(&temp, &real_flag).is_err() {
            let _ = fs::remove_file(&temp);
            return None;
        }
        Some(())
    })();
}

/// Symlink-safe, size-capped, whitelist-validated flag read. Returns None on
/// any anomaly (symlink, oversize, unknown mode).
pub fn read_flag(flag_path: &Path) -> Option<String> {
    let st = fs::symlink_metadata(flag_path).ok()?;
    if st.file_type().is_symlink() || !st.is_file() {
        return None;
    }
    if st.len() as usize > MAX_FLAG_BYTES {
        return None;
    }
    let mut f = open_rdonly_nofollow(flag_path).ok()?;
    let mut buf = vec![0u8; MAX_FLAG_BYTES];
    let n = f.read(&mut buf).ok()?;
    let raw = String::from_utf8_lossy(&buf[..n]).trim().to_lowercase();
    if is_valid_mode(&raw) {
        Some(raw)
    } else {
        None
    }
}

#[cfg(unix)]
fn open_rdonly_nofollow(path: &Path) -> std::io::Result<fs::File> {
    use std::os::unix::fs::OpenOptionsExt;
    fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
}

#[cfg(not(unix))]
fn open_rdonly_nofollow(path: &Path) -> std::io::Result<fs::File> {
    fs::File::open(path)
}

/// Symlink-safe append (O_APPEND, 0600, O_NOFOLLOW). Used for the lifetime
/// stats log. Silent-fails on any filesystem error.
pub fn append_flag(file_path: &Path, line: &str) {
    let _ = (|| -> Option<()> {
        let dir = file_path.parent()?;
        let _ = fs::create_dir_all(dir);
        let real_dir = resolve_trusted_dir(dir)?;
        let real_path = real_dir.join(file_path.file_name()?);

        match fs::symlink_metadata(&real_path) {
            Ok(m) if m.file_type().is_symlink() => return None,
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return None,
        }

        let mut f = open_append_nofollow(&real_path).ok()?;
        let trimmed = line.strip_suffix('\n').unwrap_or(line);
        f.write_all(trimmed.as_bytes()).ok()?;
        f.write_all(b"\n").ok()?;
        Some(())
    })();
}

/// Symlink-safe history read. Returns non-empty lines, or empty vec on anomaly.
/// No size cap — history grows with use.
pub fn read_history(file_path: &Path) -> Vec<String> {
    (|| -> Option<Vec<String>> {
        let st = fs::symlink_metadata(file_path).ok()?;
        if st.file_type().is_symlink() || !st.is_file() {
            return None;
        }
        let mut f = open_rdonly_nofollow(file_path).ok()?;
        let mut raw = String::new();
        f.read_to_string(&mut raw).ok()?;
        Some(
            raw.split('\n')
                .filter(|l| !l.trim().is_empty())
                .map(|l| l.to_string())
                .collect(),
        )
    })()
    .unwrap_or_default()
}

pub fn now_millis_u64() -> u64 {
    now_millis() as u64
}
