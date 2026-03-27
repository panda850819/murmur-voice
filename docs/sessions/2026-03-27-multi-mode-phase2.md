---
date: 2026-03-27
branch: main
project: murmur-voice
tags: [coding-session, murmur-voice]
chain_path: [brainstorming, executing-plans, simplify, done]
chain_grade: B
---

# multi-mode-phase2 — 2026-03-27

## What happened
Designed and implemented Phase 2 of Murmur Voice: Multi-Mode + Template Variables. Brainstorming established 4 built-in modes (Dictation, Translate, VoiceCommand, ClipboardRewrite) with independent hotkeys and template variable support ({selected}, {clipboard}). Product-lead agent coordinated eng-lead agent to implement the full feature autonomously across 13 files (+1054/-369 lines), followed by /simplify which caught dead legacy shims, orphaned frontend constants, and a redundant translate recorder. After implementation, E2E testing revealed 6 UX bugs that were fixed iteratively: permission flow (clickable items + next gate), preview auto-hide on no-speech, state event missing on silent audio, live preview hallucination/flickering, and traditional Chinese bias being truncated by oversized dictionary packs.

## Retrospective
- The brainstorming → autonomous agent team → simplify flow delivered the feature, but E2E testing revealed bugs the agent team missed — preview.js PASTED/CLIPBOARD constants were orphaned (would have been runtime bugs), and the no-speech/silent-audio paths had missing events
- Live preview hallucination was a pre-existing issue amplified by toggle mode — the 0.2s minimum audio threshold was far too low for stable Whisper output
- Dictionary packs exceeding Whisper's 1024 token limit was silently truncating the traditional Chinese bias prompt — reordering (dictionary first, language bias last) was a simple fix with high impact
- Permission UX (clickable items → open System Settings, disabled Next until both granted) significantly improves first-run experience
- Main bar dynamic expansion for long text is a good next feature — noted as follow-up

## Current state
v0.5.0 on main, 12 commits (spec + feat + simplify + 6 bug fixes). 67 tests pass, zero clippy warnings. E2E tested for basic dictation flow. VoiceCommand/ClipboardRewrite modes not yet tested (need LLM enabled). Not pushed or released.

## Follow-ups
- [ ] Main bar dynamic height expansion for long transcription text
- [ ] E2E test VoiceCommand and ClipboardRewrite modes with LLM enabled
- [ ] Version bump to v0.5.0 in Cargo.toml
- [ ] Release workflow (gh-ship)
- [ ] Dictionary packs: consider hard-capping total token count or warning user when exceeding limit
