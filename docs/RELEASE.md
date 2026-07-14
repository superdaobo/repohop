# Release Process

Stage 5 (not fully implemented yet). Target workflow:

## Artifacts

- `rhop-x86_64-pc-windows-msvc.exe` (or `rhop.exe`)
- SHA-256 checksum file
- `install.ps1` / `uninstall.ps1`

## Build

```powershell
cargo build --release
Get-FileHash target\release\rhop.exe -Algorithm SHA256
```

## GitHub Release

1. Tag `vX.Y.Z` on main after changelog update  
2. Attach binary + checksums  
3. Install script downloads latest release to a user-local bin directory and updates user PATH  

## Verification checklist

- [ ] `rhop version` matches tag  
- [ ] `rhop doctor` on clean machine  
- [ ] No dependency on system Rust/Node for running the binary  
- [ ] Uninstall removes binary and optional PATH entry without deleting user DB unless requested  

## Code signing

Optional later; document if/when certificates are available.
