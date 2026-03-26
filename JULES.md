# JULES.md — Bot Instructions for Murmur Voice

## Project Context

This is a **Tauri 2 desktop app** (Rust backend + vanilla JS frontend). It runs locally on the user's machine — NOT a web app, NOT a server.

## Severity Evaluation Rules

When assessing security or performance issues, calibrate severity for a **local desktop application**:

| Category | Web App Risk | Desktop App Risk | Notes |
|----------|-------------|-------------------|-------|
| XSS / innerHTML | HIGH | LOW | No remote content loaded; all HTML is local static files |
| TOCTOU file access | HIGH | LOW | Single-user local app; no concurrent external actors |
| API key in settings UI | MEDIUM | LOW | User's own key on their own machine; not transmitted |
| stdout/stderr logging | MEDIUM | LOW | Local process output; not exposed to network |
| Performance micro-opts | Varies | LOW | Sub-ms optimizations irrelevant for voice transcription UX |

**Do NOT mark local-only issues as CRITICAL or HIGH.** Use MEDIUM at most when a key or secret is unexpectedly transmitted over the network. Use LOW for defense-in-depth improvements.

## PR Submission Rules

1. **Before creating a PR, check if an open PR already exists targeting the same files.** If one exists, do NOT create another.
2. **At most 1 open PR per issue.** Do not submit variations of the same fix.
3. **Branch naming**: Use descriptive names. The numeric suffix should be your task ID.
4. **Limit PRs to the minimum files required** to fix the reported issue.

## Code Standards

- Rust: `cargo clippy --all-targets -- -D warnings` must pass (zero warnings policy)
- Frontend: No build step. Plain HTML/JS/CSS.
- Do not add dependencies without justification.
- Do not refactor unrelated code in the same PR.
