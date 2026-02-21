## 2025-02-14 - Content Security Policy (CSP) Implementation
**Vulnerability:** The application was missing a Content Security Policy (`csp: null`), which would allow loading scripts, styles, and other resources from any origin, increasing the risk of XSS attacks.
**Learning:** Even local-first applications built with Tauri need strict CSP to prevent malicious content from executing if an attacker manages to inject it (e.g., via compromised dependencies or unexpected input vectors).
**Prevention:** Implemented a strict CSP in `tauri.conf.json` that whitelists only necessary sources (`self`, Google Fonts, Buymeacoffee CDN).

## 2025-02-14 - Insecure File Permissions on Sensitive Data
**Vulnerability:** The application stored sensitive settings (like API keys) in a JSON file with default permissions (often `644`), making them readable by other users on the system.
**Learning:** `std::fs::write` uses the default umask, which typically allows group/other read access. Explicitly setting permissions to `0600` is crucial for files containing secrets.
**Prevention:** Modified `save_settings` to call `std::fs::set_permissions` immediately after writing, restricting access to the owner only.
