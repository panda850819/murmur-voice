
## 2025-02-14 - Removed Sensitive Data Logging
**Vulnerability:** The application was using `eprintln!` to log raw and processed transcription text to stderr in `src-tauri/src/lib.rs`. This could expose sensitive user conversations in system logs.
**Learning:** Even during development/debugging, sensitive data should never be logged using standard output macros.
**Prevention:** Removed the infringing `eprintln!` calls. Future debugging should use the `log` crate with appropriate levels (debug/trace) and ensure release builds strip or disable these logs, or better yet, never log content payload.
