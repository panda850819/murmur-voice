## 2026-02-18 - [Clipboard Data Leak on Error]
**Vulnerability:** The `insert_text` function would return early if `simulate_paste` failed, leaving the transcribed text (potentially sensitive) in the clipboard and discarding the user's original clipboard content.
**Learning:** Error propagation (`?`) in cleanup/restoration flows can bypass critical security/privacy restoration steps.
**Prevention:** Always use `defer` patterns or explicit error capturing (`let res = ...`) when subsequent cleanup code *must* run regardless of success.
