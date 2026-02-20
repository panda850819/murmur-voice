## 2025-02-14 - Content Security Policy (CSP) Implementation
**Vulnerability:** The application was missing a Content Security Policy (`csp: null`), which would allow loading scripts, styles, and other resources from any origin, increasing the risk of XSS attacks.
**Learning:** Even local-first applications built with Tauri need strict CSP to prevent malicious content from executing if an attacker manages to inject it (e.g., via compromised dependencies or unexpected input vectors).
**Prevention:** Implemented a strict CSP in `tauri.conf.json` that whitelists only necessary sources (`self`, Google Fonts, Buymeacoffee CDN).

## 2025-02-14 - Secure File Permissions for Settings
**Vulnerability:** The settings file (`settings.json`) containing sensitive API keys was being created with default file permissions (e.g., 644), making it readable by other users on the system.
**Learning:** `std::fs::write` and `File::create` use the process's umask by default, which is often permissive. Explicit permission setting is required for sensitive files.
**Prevention:** Modified `save_settings` to explicitly set file permissions to `0600` (read/write by owner only) on Unix systems using `std::os::unix::fs::PermissionsExt`.
