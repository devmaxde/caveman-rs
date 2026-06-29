# caveman — node-free installer for Claude Code (Windows / PowerShell).
#
# Builds the native Rust `caveman` binary (no Node, ever) and wires the
# SessionStart + UserPromptSubmit hooks and the statusline badge into
# settings.json.
#
# Usage:
#   pwsh install.ps1            # build + install
#   pwsh install.ps1 --force    # rebuild + re-wire over an existing install
#   pwsh install.ps1 --uninstall
#
# Requires the Rust toolchain (cargo). Install once from https://rustup.rs.

[CmdletBinding()]
param(
  [Parameter(ValueFromRemainingArguments = $true)]
  [string[]]$InstallerArgs
)

$ErrorActionPreference = "Stop"

$here = Split-Path -Parent $MyInvocation.MyCommand.Path
$manifest = Join-Path $here "rust/Cargo.toml"

$force = ""
$uninstall = $false
foreach ($arg in $InstallerArgs) {
  switch ($arg) {
    "--force"     { $force = "--force" }
    "-f"          { $force = "--force" }
    "--uninstall" { $uninstall = $true }
  }
}

# Make cargo reachable even if the shell was not reopened since installing rustup.
$cargo = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $cargo) {
  $cargoBin = Join-Path $HOME ".cargo/bin"
  if (Test-Path $cargoBin) { $env:PATH = "$cargoBin;$env:PATH" }
  $cargo = Get-Command cargo -ErrorAction SilentlyContinue
}
if (-not $cargo) {
  Write-Error @"
caveman: 'cargo' (Rust toolchain) not found. caveman is now native Rust — no Node required.
  Install Rust once: https://rustup.rs
  Then re-run: pwsh install.ps1
"@
  exit 1
}

if (-not (Test-Path $manifest)) {
  Write-Error "caveman: cannot find $manifest — run this script from a caveman checkout."
  exit 1
}

Write-Host "Building caveman (release)..."
& cargo build --release --manifest-path $manifest
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$bin = Join-Path $here "rust/target/release/caveman.exe"
if (-not (Test-Path $bin)) { $bin = Join-Path $here "rust/target/release/caveman" }
if (-not (Test-Path $bin)) {
  Write-Error "caveman: build did not produce a binary at $bin"
  exit 1
}

if ($uninstall) {
  & $bin uninstall
  exit $LASTEXITCODE
}

Write-Host ""
if ($force) { & $bin install $force } else { & $bin install }
exit $LASTEXITCODE
