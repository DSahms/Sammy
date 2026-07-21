# Env notes (machine-specific)

- Rust toolchain via rustup (required).
- Strawberry Perl installed so SQLCipher’s vendored OpenSSL can build on Windows.
- `cargo build` (debug) succeeds, BUT a raw `cargo build` exe stays in Tauri dev mode — it expects the Vite dev server at `localhost:1420` and shows "web page can't be found" when launched standalone. Do not ship a raw `cargo build` exe.
- `tauri build` (release, full bundler) may hit Windows Application Control blocking some crate build scripts.

## Standalone desktop exe (works, no dev server needed)

Build a self-contained exe with the frontend embedded, skipping the installer bundler:

```
npm run tauri build -- --debug --no-bundle
```

Output: `src-tauri/target/debug/sammy.exe` (UI embedded; launches standalone).
Desktop shortcut: `%USERPROFILE%\OneDrive\Desktop\Sammy.lnk` -> that exe.
Close any running `sammy.exe` first (file lock), rebuild, relaunch.

## Local brain

KoboldCpp: `http://127.0.0.1:5001` (preferred when reachable). Venice cloud is fallback when a key is saved in the OS keychain.
