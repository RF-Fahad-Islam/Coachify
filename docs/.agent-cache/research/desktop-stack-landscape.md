# Desktop stack landscape check (2026-09-20)

Captured by /architect for spec 0001. Reuse for 30 days.

1. Tauri 2.11.5 (July 2026). No built in silent printing. Call WebView2 `ICoreWebView2_16.PrintAsync` with `ICoreWebView2PrintSettings` through the `webview2-com` crate. Community `tauri-plugin-printer-v2` unmaintained since May 2024. https://v2.tauri.app/release/
2. rusqlite `bundled-sqlcipher` (and `bundled-sqlcipher-vendored-openssl`) builds SQLCipher with FTS5; trigram tokenizer needs SQLite 3.34+, which current SQLCipher includes. Verify in the spike. https://github.com/sqlcipher/sqlcipher
3. Tauri NSIS `webviewInstallMode: offlineInstaller` still current. `tauri-plugin-updater` signature verification mandatory (minisign style keys). https://v2.tauri.app/plugin/updater/
4. `tauri-specta` v2 still a release candidate (rc.25, May 2026). `specta` and `ts-rs` are stable. https://github.com/specta-rs/tauri-specta/releases
5. Electron 44.4.3 (September 2026) supports `webContents.print({ silent, deviceName, pageSize })`. napi-rs is stable for calling a Rust core from Node. https://releases.electronjs.org/release
