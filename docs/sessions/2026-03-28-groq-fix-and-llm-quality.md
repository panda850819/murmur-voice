---
date: 2026-03-28
branch: main
project: murmur-voice
tags: [coding-session, murmur-voice]
chain_path: [gh-ship, done]
chain_grade: A
---

# groq-fix-and-llm-quality — 2026-03-28

## What happened

User reported Groq transcription failing with 400 error — dictionary packs produced a 4569-char prompt exceeding Groq's 896-char limit. Fixed by truncating prompt before sending. Also addressed LLM post-processing quality: removed the `protect_english` placeholder mechanism that was stripping semantic context from the LLM, lowered `frequency_penalty` from 1.5 to 0.3, and added tone-aware punctuation rules to prevent false question marks. Error display UX was improved from 3s to 8s visibility, and preview window now shows errors. Shipped as v0.5.1. CI fix for Windows dead code (`pause_all_hotkeys`) was also pushed.

## Current state

- v0.5.1 released and tagged
- CI fix pushed, waiting for green
- Release builds (macOS, Windows, Windows CUDA) were in progress at session end

## Follow-ups

- [ ] Verify LLM post-processing quality improvement with real usage (test with `RUST_LOG=debug pnpm tauri dev`)
- [ ] Monitor if English words get modified after removing `protect_english` — may need prompt tuning
