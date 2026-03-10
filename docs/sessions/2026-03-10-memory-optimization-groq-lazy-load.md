---
date: 2026-03-10
branch: main
project: murmur-voice
tags: [coding-session, murmur-voice, performance]
---

# memory-optimization-groq-lazy-load — 2026-03-10

## What happened

User reported ~2GB memory usage. Root cause: Whisper model (`ggml-large-v3-turbo.bin`, 1.62GB) loads at startup regardless of engine setting, so Groq-only users waste ~2GB holding a model they never use. Fixed by gating startup engine loading on `engine != "groq"`. Also added engine lifecycle management in `save_settings` — switching to Groq unloads the engine (frees memory), switching to local triggers background loading.

Deferred model download from onboarding to first recording attempt. Onboarding simplified from 5 steps to 4 (removed model download step). When local engine user presses record without model, download triggers automatically with progress shown in main bar.

Extracted duplicated engine loading code (3 copies) into `spawn_engine_load()` helper. Fixed double lock acquisition and unnecessary clones in `save_settings`.

Also: cleaned up 30 bot PRs + 23 stale remote branches, updated README with Groq API key guide, fixed zh-TW i18n for recording mode buttons.

## Current state

Committed and tagged as v0.3.7. Released.

## Follow-ups

- [ ] Investigate disabling the Bolt/Sentinel bots that generate daily duplicate PRs
- [ ] Consider idle-unload for local engine users (further memory savings when app is idle)
