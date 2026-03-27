---
date: 2026-03-27
branch: main
project: murmur-voice
tags: [coding-session, murmur-voice]
chain_path: [brainstorming, writing-plans, executing-plans, simplify, dev-review, done]
chain_grade: B
---

# dynamic-height-expansion — 2026-03-27

## What happened

Implemented main bar dynamic height expansion for Murmur Voice. The 420x48px overlay bar now grows upward during recording to show multi-line live transcription (max 160px), then collapses back to 48px when the result is ready and the preview window takes over. The initial design went through brainstorming with design-lead and product-lead reviews, then a formal spec and implementation plan. Implementation required multiple iterations to fix: (1) preview window showing simultaneously during recording, (2) CSS `calc(100vh)` creating infinite resize loops with ResizeObserver, (3) flex layout alignment issues in expanded state, (4) layout jank from instant 120px jump vs gradual expansion. Also fixed two Whisper live preview issues that surfaced during testing: capped live preview audio to last 10 seconds to prevent repetition hallucinations, and locked language detection after first CJK content to prevent language flickering.

## Retrospective

- The original ResizeObserver + `calc(100vh)` approach was fundamentally flawed due to circular dependency between CSS viewport units and native window size. Replaced with a simpler "measure scrollHeight on text change" approach.
- Design-lead's early recommendation to keep two-window system (bar for recording, preview for results) was correct -- merging would have been much more complex.
- Product-lead caught two important issues in spec review: expand-only rule and decoupled preview timing. Both prevented bugs.
- The iteration count (11 commits for one feature) reflects the difficulty of coordinating CSS layout, native window resize, and IPC timing in a Tauri overlay app. Each fix revealed the next layer of issues.

## Current state

Feature is complete and passing all three agent reviews (product, eng, design). 11 commits on main, clippy clean, 67 tests pass. Ready for version bump.

## Follow-ups

- [ ] VoiceCommand/ClipboardRewrite E2E tests (original session plan item 2)
- [ ] Version bump + release (original session plan item 3)
- [ ] Groq mode: bar briefly expands to 80px with no content before collapsing (minor cosmetic)
- [ ] Language lock is CJK-only -- Japanese/Korean content locks to "zh" which may not be optimal
- [ ] Consider atomic window resize via NSPanel setFrame: to fully eliminate jank
