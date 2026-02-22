---
date: 2026-02-22
branch: main
project: murmur-voice
tags: [coding-session, murmur-voice]
---

# mic-permission-groq-hallucination-fix — 2026-02-22

## What we were doing
用戶測試打包的 v0.3.3 時遇到三個問題：麥克風權限偵測失敗、Groq 轉錄輸出多語言幻覺亂碼、onboarding 重複出現。

## Key Decisions
- **用 dlsym 取得 AVMediaTypeAudio 常數**：原本用 `cf_str("soun")` 自建 NSString，但 `authorizationStatusForMediaType:` 不認自建的字串。用 `dlsym` 從 AVFoundation framework 取得真正的 `AVMediaTypeAudio` 常數才能正確查詢權限狀態。
- **統一音頻品質檢查**：原本 local whisper 有 MIN_SAMPLES + energy check，但 Groq 路徑完全沒有。改為在轉錄前統一檢查，對所有引擎都生效。
- **用 cpal 觸發 macOS 權限對話框**：而非直接用 AVCaptureDevice requestAccessForMediaType（需要建 Objective-C block，從 Rust FFI 太複雜）。

## Problems & Solutions
- **AVCaptureDevice authorizationStatus 永遠不是 3**：`cf_str("soun")` 自建 NSString 不被 AVFoundation 認可 → 用 `dlsym` 從 framework 取得 `AVMediaTypeAudio` 常數
- **Groq 轉錄幻覺**：Groq 路徑沒有音頻品質檢查，靜音/噪音直接送 API → 統一 MIN_TRANSCRIBE_SAMPLES (1s) + energy check
- **Onboarding 重複**：上述兩個問題的下游效果 → 修好權限偵測 + 加 request_microphone 命令

## Follow-ups
- [ ] 調查 Groq + zh 在正常音頻下是否還有幻覺問題（用戶說修好了但只測了一次）
- [ ] 考慮 bump 版本到 0.3.4 並打包發布

## Notes
- PR Review: 同時 review 了 Jules bot 開的 4 個 PR (#28-#31)，close #28/#29（stale base + bugs），merge #30/#31
- Jules bot 的 PR 品質不一：會基於過時 code、夾帶 .jules/ metadata 和 binary files
- `#[link(name = "AVFoundation", kind = "framework")]` 掛在空的 `extern "C" {}` 不會真的載入 framework — 需要 dlopen 或 dlsym
