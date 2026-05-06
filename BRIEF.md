# Murmur Voice: Brief

> 3 個月實際 scope cap。跟 `ROADMAP.md` 衝突時以本檔為準。
> ROADMAP.md = long-term wishlist；BRIEF.md = 當前實際做什麼。

- **Status**: dogfood-only
- **Window**: 2026-05-05 → 2026-08-05
- **Next review**: 2026-08-05 (kill / pivot / continue)

## Frame

個人 + 小團體 dogfood 工具。不追 typeless feature parity。不做 PMF 驗證。不做行銷。

唯一存在理由：自己每天用、不退化。

## Users

- Panda 自己（主力使用者，每日錄會議 / 日記 / 思考）
- 同事 (Yei / Sommet 內) 自願使用者
- 朋友圈 自願使用者

不主動推廣。不開 issue 給陌生人。

## Time budget

每月 2-4 小時。包含：bug fix + playback regression run + competitor gap log。

超過 budget 表示在做 scope 外的事，停下來檢查是不是漂走了。

## Quality gate (唯一)

每次 release 必須通過：

1. **Playback regression set 不退化**: `apps/murmur-voice/test/fixtures/` 內 10-20 個錄音檔，WER 不能比上一版差。
2. **Sanity filter 無 false negative**: LLM 輸出無「奇怪符號」(emoji / box drawing / 控制字元 / 連續標點 / 非預期 unicode block)。fixture set 內任一檔觸發 sanity filter fallback 就視為 regression。

playback runner 跑通才能 tag release。沒過就回滾。

## Out of scope (明確不做)

- 第二張前端皮（macOS native 或 menu bar 二選一，鎖定到 v0.5 之前不開第二張）
- typeless 出新 feature 跟做（gap log 觀察用，不 implement）
- 公開 marketing / Product Hunt / 寫部落格推
- v0.5 Context-Aware Intelligence (ROADMAP.md 段落) — 暫緩到下個 review window
- Power Mode / 螢幕 OCR / Accessibility API integration — 暫緩
- vault 整合 — 暫緩
- Windows-specific 功能 cycle（macOS 主力，Windows 跟 release 不主動修）
- 接 automated scanning bots / coding agents 自動 PR（threat model 是 dogfood 個人工具，不是 prod multi-user app；TOCTOU / XSS / API-key-in-settings 這類在 web app 是真 bug，在這裡不是。per 5/6 Jules cleanup）

ROADMAP.md v0.4 內容（Speed + Data Sovereignty）酌情做，**前提是 quality gate 沒 regression**。

## Competitors

baseline: **typeless**（體驗最好的競品）。已接受體驗 gap，不追功能。

每月底跑 `COMPETITORS.md` gap log（15 分鐘）：

- 跑同 fixture set 在 typeless / superwhisper / wisprflow
- 記 3 條最大體驗差距（不是 feature 差距）
- 累積 6 個月後（2026-11）看趨勢決定收掉還是繼續

不在每月 gap log 上的競品變化，不看不追。

## Bug / roadmap location

- Bug → GitHub Issues（不進 vault daily note）
- Roadmap wishlist → ROADMAP.md（已存在，當 long-term reference）
- Scope cap → 本檔
- 競品觀察 → COMPETITORS.md（待建）

## Review triggers (任一觸發 = 強制 review)

- 月 budget 連續 2 個月超過 4 小時
- playback regression 連 2 次 release 沒過
- typeless gap log 出現「我願意切過去」的條目
- 2026-08-05 例行 review

review 結果三選一：
1. **Continue**：續寫一份新 BRIEF，window 再 +3 個月
2. **Pivot**：scope 重定義（e.g. 從 dogfood 變差異化競品，但要先寫 5 條 typeless 不滿足的 case）
3. **Kill**：archive repo，切去用 typeless

## What this brief is NOT

- 不是 product spec
- 不是 marketing plan
- 不是 ROADMAP.md 的取代

是 scope cap，避免做超過。
