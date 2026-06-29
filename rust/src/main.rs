// caveman — native Claude Code hooks + installer, single binary.
//
// Subcommands replace the former Node hooks one-for-one:
//   activate      SessionStart hook  (was caveman-activate.js)
//   mode-tracker  UserPromptSubmit   (was caveman-mode-tracker.js)
//   stats         /caveman-stats     (was caveman-stats.js)
//   statusline    statusline badge   (was caveman-statusline.sh)
//   init          per-repo rules     (was src/tools/caveman-init.js)
//   install       wire hooks + statusline into settings.json (was install.sh, no Node)
//   uninstall     remove caveman entries from settings.json   (was uninstall.sh, no Node)

mod activate;
mod config;
mod init;
mod install;
mod mode_tracker;
mod settings;
mod stats;
mod statusline;

use std::process::exit;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("");
    let rest: Vec<String> = args.iter().skip(2).cloned().collect();

    match cmd {
        "activate" => activate::run(),
        "mode-tracker" => mode_tracker::run(),
        "stats" => exit(stats::run(&rest)),
        "statusline" => statusline::run(),
        "init" => exit(init::run(&rest)),
        "install" => exit(install::run_install(&rest)),
        "uninstall" => exit(install::run_uninstall(&rest)),
        "--version" | "-V" | "version" => {
            println!("caveman {}", env!("CARGO_PKG_VERSION"));
        }
        "" | "-h" | "--help" | "help" => print_help(),
        other => {
            eprintln!("caveman: unknown subcommand '{}'\n", other);
            print_help();
            exit(2);
        }
    }
}

fn print_help() {
    println!(
        "caveman {} — terse caveman mode for Claude Code (native, no Node)\n\
\n\
Usage: caveman <command> [args]\n\
\n\
Hook commands (wired into settings.json by `install`):\n\
  activate        SessionStart hook — emit ruleset, write mode flag\n\
  mode-tracker    UserPromptSubmit hook — switch modes, reinforce\n\
  statusline      print the [CAVEMAN] statusline badge\n\
\n\
User commands:\n\
  stats [--share] [--all] [--since Nd|Nh] [--session-file F]\n\
                  token usage + estimated savings\n\
  init [dir] [--dry-run] [--force] [--only <agent>]\n\
                  drop the always-on caveman rule into a repo\n\
  install [--force]   copy this binary into the Claude config dir and wire hooks\n\
  uninstall           remove caveman hooks + statusline from settings.json\n\
  --version           print version",
        env!("CARGO_PKG_VERSION")
    );
}
