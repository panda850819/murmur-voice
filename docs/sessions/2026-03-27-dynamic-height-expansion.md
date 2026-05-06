---
date: 2026-03-27
branch: main
project: murmur-voice
tags: [coding-session, murmur-voice]
chain_path: [brainstorming, writing-plans, executing-plans, simplify, dev-review, dev-qa, gh-ship, done]
chain_grade: B
---

# dynamic-height-expansion — 2026-03-27

## What happened

Implemented main bar dynamic height expansion for Murmur Voice. The 420x48px overlay bar now grows upward during recording to show multi-line live transcription (max 160px), then collapses back to 48px when the result is ready and the preview window takes over. The initial design went through brainstorming with design-lead and product-lead reviews, then a formal spec and implementation plan. Implementation required multiple iterations to fix: (1) preview window showing simultaneously during recording, (2) CSS `calc(100vh)` creating infinite resize loops with ResizeObserver, (3) flex layout alignment issues in expanded state, (4) layout jank from instant 120px jump vs gradual expansion. Also fixed two Whisper live preview issues that surfaced during testing: capped live preview audio to last 10 seconds to prevent repetition hallucinations, and locked language detection after first CJK content to prevent language flickering.

After feature completion, ran manual E2E QA for VoiceCommand and ClipboardRewrite modes (5/5 flows passed). Bumped version to v0.5.0 and shipped release to GitHub. User then reported Groq transcription engine producing empty results — discovered `env_logger` was never initialized so all log output was silent. Added `env_logger::init()` but root cause of Groq failure still unknown (needs debug logs in next session).

## Retrospective

- The original ResizeObserver + `calc(100vh)` approach was fundamentally flawed due to circular dependency between CSS viewport units and native window size. Replaced with a simpler "measure scrollHeight on text change" approach.
- Design-lead's early recommendation to keep two-window system (bar for recording, preview for results) was correct -- merging would have been much more complex.
- Product-lead caught two important issues in spec review: expand-only rule and decoupled preview timing. Both prevented bugs.
- The iteration count (11 commits for one feature) reflects the difficulty of coordinating CSS layout, native window resize, and IPC timing in a Tauri overlay app. Each fix revealed the next layer of issues.
- Groq transcription engine issue surfaced late — error handling hides windows immediately after emitting RECORDING_ERROR, so user never sees the error message. Two bugs: (1) the actual Groq API failure, (2) error UX hiding too fast.

## Current state

v0.5.0 released to GitHub (CI builds macOS + Windows). Dynamic height, multi-mode, E2E all done. Groq transcription engine broken (empty results) — `env_logger::init()` added but not yet shipped. Next session: run with `RUST_LOG=debug` to see actual error, fix Groq issue, fix error display UX.

## Follow-ups

- [x] VoiceCommand/ClipboardRewrite E2E tests — PASS (5/5 flows)
- [x] Version bump + release — v0.5.0 shipped
- [ ] Debug Groq transcription engine empty results (run with RUST_LOG=debug)
- [ ] Fix error display UX: window hidden immediately after RECORDING_ERROR, user can't see error
- [ ] Groq mode: bar briefly expands to 80px with no content before collapsing (minor cosmetic)
- [ ] Language lock is CJK-only -- Japanese/Korean content locks to "zh" which may not be optimal
- [ ] Consider atomic window resize via NSPanel setFrame: to fully eliminate jank
