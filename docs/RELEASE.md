# Release Process

## One-line install (end users)

```powershell
irm https://raw.githubusercontent.com/superdaobo/repohop/main/install.ps1 | iex
```

- Installs to `%LOCALAPPDATA%\RepoHop\bin\rhop.exe`
- Appends that directory to the **user** `PATH` (no admin)
- Verifies SHA-256 when `SHA256SUMS.txt` is present on the release
- Pin version: `$env:REPOPHOP_VERSION = 'v0.1.0'` before running the installer

Uninstall:

```powershell
irm https://raw.githubusercontent.com/superdaobo/repohop/main/uninstall.ps1 | iex
```

## Automated release (GitHub Actions)

Workflow: [`.github/workflows/release.yml`](../.github/workflows/release.yml)

**Triggers**

1. Push a version tag: `git tag v0.1.0 && git push origin v0.1.0`
2. Manual: Actions → **Release** → Run workflow (optional tag input)

**What it does**

1. `cargo fmt` / `clippy` / `test` / `build --release --locked` on `windows-latest`
2. Packages:
   - `rhop-x86_64-pc-windows-msvc.exe`
   - `rhop.exe`
   - `install.ps1`, `uninstall.ps1`
   - `SHA256SUMS.txt`
3. Creates a GitHub Release and uploads artifacts
4. Also uploads a workflow artifact `rhop-windows-x64` for download from the Actions run

## Local build

```powershell
cargo build --release --locked
Get-FileHash target\release\rhop.exe -Algorithm SHA256
```

## Versioning

- Crate version: `Cargo.toml` → `version`
- Release tags: `vMAJOR.MINOR.PATCH` (semver)
- Keep `CHANGELOG.md` updated before tagging

## Verification checklist

- [ ] `rhop version` matches release tag
- [ ] `rhop doctor` on a machine without Rust
- [ ] Installer one-liner works in PowerShell 5.1 and 7
- [ ] New terminal finds `rhop` on PATH
- [ ] Uninstall removes binary and PATH entry; data kept unless purged

## Code signing

Optional later; document if/when certificates are available.
