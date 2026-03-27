---
date: 2026-03-19
branch: main
project: murmur-voice
tags: [coding-session, murmur-voice]
---

# v0.4.3 features and fixes — 2026-03-19

## What happened

Big session covering multiple improvements shipped as v0.4.3. Started with the translate hotkey being limited to two keys (one modifier + one regular key) — refactored the entire hotkey recording flow in settings.js to accumulate multiple modifiers via a Set, and updated the Rust backend to handle multi-modifier parsing (macOS: OR'd bitmask, Windows: packed VK codes in u64 slots). Fixed the `(flags & mask) != 0` check to `== mask` so all modifiers must be held.

Found and fixed the preview window stuck on "LISTENING..." after ESC cancel — preview.js wasn't listening for the `recording_cancelled` event. Also fixed the persistent onboarding re-trigger bug: the Settings struct had 7 fields without `#[serde(default)]`, so any deserialization hiccup would reset everything to defaults including `onboarding_complete: false`. Fixed by adding `#[serde(default)]` at struct level and adding error logging to `load_settings`.

Added Whisper anti-hallucination measures from a prior session's work (audio.rs threshold change, confidence gating, no_speech segment filtering, hallucination pattern detection). Ran /simplify which caught dead code (7 unused default_* functions), inconsistent thresholds, and CJK false positives in the 1-2 char hallucination filter.

Finally built the dictionary packs feature. First attempt loaded all terms as individual tags in the UI (178 tags cluttering the screen) — user rejected this. Redesigned as toggle checkboxes (Crypto/Web3, AI/ML, Dev Tools) that load pack terms into Whisper's initial_prompt via `include_str!` without touching the custom dictionary UI. Clean separation between pack terms and user's custom terms.

## Current state

Complete. v0.4.3 shipped — commit, tag, push, GitHub release all done. CI should be building the release artifacts (macOS .dmg, Windows CPU .msi, Windows CUDA .msi).

## Follow-ups

- [ ] Test dictionary packs in real usage — verify Whisper accuracy improvement with packs enabled
- [ ] Consider adding more dictionary packs based on user feedback
- [ ] The `displayNameFor` function in settings.js and `pttDisplayName` in onboarding.js are duplicated (found by simplify review, deferred as out of scope)
