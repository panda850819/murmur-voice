---
date: 2026-03-18
branch: main
project: murmur-voice
tags: [coding-session, murmur-voice]
---

# whisper-anti-hallucination — 2026-03-18

## What happened

User reported that Whisper produces hallucinated text (e.g. "局長形線內海邊。") when no speech is present — just pressing the hotkey without saying anything. The existing anti-hallucination measures (energy threshold at 1e-6, suppress_blank, no_speech_thold) were insufficient because background noise easily passes the very low energy gate.

Implemented a multi-layer defense: raised the silence energy threshold from 1e-6 to 5e-5, added per-segment no_speech_probability filtering (>0.5 threshold), added token-average-probability confidence gate (reject if avg < 0.4), and added a known hallucination phrase blocklist covering English, Traditional/Simplified Chinese, and Japanese patterns. Also added a guard for very short output (<=2 chars).

During debugging, accidentally `cat`'d the settings.json which exposed the user's Groq API key in the conversation. Added this to the error learning log in CLAUDE.md.

User also reported translate hotkey (Option+T) not working due to conflicts with other apps. Changed translate_hotkey from `AltLeft+KeyT` to `ControlLeft+ShiftLeft+KeyT` in settings.json.

## Current state

Anti-hallucination code is written and tests pass (47/47), but changes are uncommitted and untested in a live session (user was running the old build). The translate hotkey setting has been updated in settings.json but needs app restart to take effect.

## Follow-ups

- [ ] User should test with `pnpm tauri dev` to validate anti-hallucination changes work in practice
- [ ] User should rotate Groq API key (exposed in conversation)
- [ ] If hallucinations still slip through, tune the avg_token_prob threshold (currently 0.4) based on log output
- [ ] Commit changes after validation
