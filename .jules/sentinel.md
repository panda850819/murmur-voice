## 2025-02-14 - Content Security Policy (CSP) Implementation
**Vulnerability:** The application was missing a Content Security Policy (`csp: null`), which would allow loading scripts, styles, and other resources from any origin, increasing the risk of XSS attacks.
**Learning:** Even local-first applications built with Tauri need strict CSP to prevent malicious content from executing if an attacker manages to inject it (e.g., via compromised dependencies or unexpected input vectors).
**Prevention:** Implemented a strict CSP in `tauri.conf.json` that whitelists only necessary sources (`self`, Google Fonts, Buymeacoffee CDN).
