---
date: 2026-03-01
branch: main
project: murmur-voice
tags: [coding-session, murmur-voice, refactor]
---

# simplify-followup — 2026-03-01

## What happened

跑了 /simplify 對最近的 code changes 做三面向 review（reuse, quality, efficiency），即時修了 3 個問題（audio usability check 重複、AVFoundation dlopen 洩漏、mic request 沒 early return）。Review 還找到 6 個 skipped items，用 OpenSpec 開了 `simplify-followup` change 來管理。

用 /opsx-team-apply 分 3 個 phase 平行實作 17 個 tasks：Phase 1 兩個 agent 做 IPC constants + energy subsample，Phase 2 兩個 agent 做 idle reset helper + device-open helper，Phase 3 一個 agent 做 tri-state mic permission。每個 phase 過 cargo check + clippy gate 才進下一 phase。

/opsx:verify + Codex review 雙重驗證抓到 2 個 agent 引入的 bug：settings.html 漏載 events.js 導致 COMMANDS undefined，以及 AudioRecorder::start() 被簡化後丟失了 smart config selection（default_input_config 可能回 U16 格式）。兩個都修了。

## Current state

完成，無進行中的事。bcd357d 已 push to main。OpenSpec change 已 archive。

主要改動：events.rs (new), lib.rs (reset_to_idle + event constants), audio.rs (is_audio_usable subsample + open_default_input), frontapp_macos.rs (OnceLock cache + tri-state), onboarding.js (tri-state mic UX), 所有 JS files 改用 COMMANDS constants。
