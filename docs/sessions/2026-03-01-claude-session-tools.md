---
date: 2026-03-01
branch: main
project: murmur-voice
tags: [coding-session, claude-session-tools, cli]
---

# claude-session-tools — 2026-03-01

## What happened

用戶分享了 claude-devtools (matt1398/claude-devtools) 這個 GUI 工具，它能可視化 Claude Code 的 session logs。安裝試用後，用戶決定把核心功能包裝成 CLI + skills，這樣不需要開 GUI 就能在 Claude Code session 內直接取得分析結果。

先逆向分析了 claude-devtools 的原始碼，搞清楚它讀什麼資料（~/.claude/projects/*/*.jsonl）、怎麼解析（streaming dedup by requestId、compaction detection via isCompactSummary flag）、算了什麼指標。然後照 slack-cli 的 pattern（Python + Typer + Rich + hatchling/uv）建了 claude-session-tools CLI，用 subagent-driven development 平行實作 8 個 tasks。

最後建了 3 個 skills（session-analyze、session-list、context-budget）掛在 session-tools pack 下，並更新了 registry.yaml。

## Current state

完成，CLI 已全域安裝（`claude-sessions`），3 個 skills 已建立並 symlink。claude-devtools GUI 也裝了但用戶似乎打開後遇到問題（未深入排查）。

## Follow-ups

- [ ] claude-devtools GUI 無法正常使用的問題未排查（可能是 macOS 安全性限制，需 xattr -cr）
- [ ] 考慮加 `claude-sessions search` 指令做跨 session 全文搜尋
- [ ] context-budget 目前不計算 skills 的 token 佔用（skills 是動態載入的，不好估算）
