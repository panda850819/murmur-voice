---
date: 2026-03-11
branch: main
project: murmur-voice
tags: [coding-session, murmur-voice, translate-hotkey]
---

# translate-hotkey — 2026-03-11

## What happened

Designed and implemented a translate hotkey feature for Murmur Voice. The feature lets users select text in any app, press Option+T, and get it translated via the existing LLM provider (Groq/Ollama/Custom) — the translation replaces the selection in-place and shows in the preview window.

Started with brainstorming (clarified design: shared LLM provider, fixed target language, full UI feedback with preview, Option+T default). Wrote spec, ran two rounds of spec review to fix critical issues (private field access, modifier key conflict, concurrency guard, create_enhancer bypass). Then wrote implementation plan with 11 tasks across 2 chunks.

Executed the plan using subagent-driven development — dispatched 5 parallel agents for the independent backend tasks (settings, events, llm, clipboard, hotkey), then lib.rs integration, then 4 parallel frontend agents (i18n, main.js, preview.js, settings UI). All 45 tests pass, clippy clean.

Hit two bugs during smoke testing: (1) copy_selection didn't work because Option key was still held during Cmd+C simulation — fixed by releasing all modifier keys before simulating copy; (2) main window stayed visible after translation — fixed by hiding it after translation completes. Updated both READMEs with the new feature.

Also created a `docs-update` skill and registered it to SKM. This skill analyzes git changes and updates README/CHANGELOG before commits. Chained it as upstream of gh-commit and done.

## Current state

Feature complete and working. 15 commits on main, 45/45 tests pass, clippy clean. READMEs updated (EN + zh-TW). Not yet version-bumped or released.

## Follow-ups

- [ ] Version bump (0.3.8?) and release
- [ ] Test on Windows (hotkey detection uses GetAsyncKeyState for translate key)
- [ ] Consider auto-detect + reverse translation (zh<->en) as future enhancement
