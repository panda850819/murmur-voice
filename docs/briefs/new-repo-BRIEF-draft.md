# {新 repo name TBD}: Brief

> Draft authored 2026-05-11 in `murmur-voice/docs/briefs/`. Sprint 2 will copy this file into the new repo root as `BRIEF.md` at scaffold time.
>
> 3 個月實際 scope cap。跟未來 `ROADMAP.md` 衝突時以本檔為準。

- **Status**: pre-build (新 repo 未開, scaffold pending Sprint 2)
- **Window**: 2026-05-11 → 2026-08-11
- **Hard kill date**: 2026-08-11 (criteria below)
- **Next review**: 2026-08-11 (kill / pivot / continue)

## Frame

個人練手導向, 練 **product-building flow** (寫 BRIEF / ROADMAP / dogfood loop / release process / retro), 順便練 **Apple modern stack** (Swift + SwiftUI + WhisperKit, macOS+iOS native).

不是 ship 競品。不是做 PMF。不主動推廣。

唯一存在理由：自己每天 mobile + desktop 用、不退化, 同時對「現代 Apple stack + cross-platform iOS/macOS native」累積實作經驗。

舊 `murmur-voice` (Tauri/Rust, macOS+Windows desktop) 進 bug-fix mode 不退役, 繼續 dogfood 直到新 repo 有 dogfoodable iOS TestFlight build。

## User problem (1 sentence)

Panda 在 mobile (iOS, 通勤 / 走路 / 排隊) 想語音轉文字, 但現有 desktop-only 工具 (typeless / superwhisper / VoiceInk / 自己的 murmur-voice) 都不能用。

## 1 metric (success signal)

**Panda 日均 mobile dictation 觸發 ≥ 5 次, 連續 ≥ 14 天 (2026-08-11 前達成)**。

低於這個數 = 沒解掉問題 = 自己都不用 = 失敗。不是「app 好不好」, 是「真有沒有把現實場景吃掉」。

備用觀察 (NOT decision metric, 只是 debug 用):
- desktop dictation 觸發次數 (mirror 舊 murmur-voice baseline, 不能比舊版低)
- WhisperKit on-device transcribe latency p50 / p95 (沒目標, 只觀察)
- Groq fallback 觸發率 (預期 < 20%, 高代表 on-device 不夠用)

## Users

- Panda 自己 (主力, 每日 mobile + desktop)
- Yei / Sommet 同事自願使用者 (繼承自舊 murmur-voice dogfood 圈, 等 TestFlight)
- 朋友圈自願使用者 (同上)

不主動推廣。不開 issue 給陌生人。不上 Mac App Store / iOS App Store paid 通路 (TestFlight 內部分發, 跟 BRIEF 哲學一致)。

## Budget (usage-based, NOT hour-based)

不寫月時數 — 時間估計向來不準, 寫了也是自欺欺人。改用 **行為 anchor**:

- **Monthly check-in (每月 11 號, 對齊 hard kill 月節點)**: 寫一段 200 字以內紀錄 (檔: `docs/sessions/YYYY-MM-DD-monthly-check.md`), 答三題:
  1. 這個月有沒有真的打開 repo 寫 code? (Y/N)
  2. Panda 自己 dogfood 觸發次數 (粗估, 不求精確)
  3. 學 Swift / SwiftUI / WhisperKit 哪一塊卡住 / 沒卡住?
- **Drift signal**: 連續 2 個月 check-in 答 #1 = N → 強制 review (見 review triggers)。
- **學習是 feature**: 看 Apple docs / WhisperKit README / SwiftUI tutorial 都算 repo work, 不是 cost; 不要假裝「等熟了再開始」。

## Quality gate (唯一)

每次 release (TestFlight build + macOS DMG) 必須通過：

1. **Self-recorded fixture set 不退化**: 開新 repo 後第一個任務之一是錄 10-20 個 Apple-device-native fixture (iPhone mic + MacBook mic), 跑 WhisperKit on-device + Groq fallback, baseline WER 紀錄。後續 release WER 不能比上一版差。
   - 注意: 不沿用舊 murmur-voice `test/fixtures/` (Tauri cpal-recorded, 規格不同)
   - fixture 規格 + baseline 在 Sprint 3 dojo 階段定
2. **Sanity filter 無 false negative**: LLM enhance 輸出無「奇怪符號」(emoji / box drawing / 控制字元 / 連續標點 / 非預期 unicode block)。fixture set 內任一檔觸發 sanity filter fallback 就視為 regression。
3. **iOS TestFlight build 可裝可跑**: build 過 / 上得了 TestFlight / 裝得進 Panda 自己的 iPhone / 啟動不 crash。

playback runner + manual install test 都跑通才能 tag release。沒過就回滾。

## Out of scope (明確不做)

- 跟 typeless / superwhisper / VoiceInk 競爭 (frame = 練手 not market)
- PMF 驗證 / marketing / Product Hunt / acquisition
- AppKit (VoiceInk 用的, 我們學 SwiftUI 不學 AppKit)
- Mac Catalyst (separate iOS native target, share core via Swift Package)
- whisper.cpp via FFI (WhisperKit Core ML pipeline only)
- Android (iOS dogfood 穩定再考慮, 不在本 window 內)
- Rust (新 product 完全不用; 舊 murmur-voice 維持 Rust 在 bug fix mode)
- Tauri (新 product 換 SwiftUI)
- Ollama / Custom LLM provider (Groq only at MVP)
- Multi hotkey mode (一個 primary mode, dictation, 即可)
- 直接 fork VoiceInk (GPL 傳染 + maker motivation 衝突)
- copy VoiceInk 任何 source code (只看 design patterns; 違反這條 = 未來不能 commercial / paid release)
- "Verifiable privacy" 賣點作為**主打** (WhisperKit on-device default 保留能力, 但不打這個戰場)
- v0.1 之前任何 power feature (Power Mode / Personal Dictionary smart replace / app-aware preset / screen OCR / Accessibility API integration / Apple Foundation Models)

舊 `murmur-voice` ROADMAP.md v0.4 / v0.5 / v1.0 內容**完全不繼承**。新 repo 開 ROADMAP.md 從零寫。

## MVP definition (v0.1, 不能再簡化)

**macOS only, 一個 dictation flow, 一個按鈕。**

- 按 hotkey → 錄音 (AVAudioEngine) → WhisperKit on-device transcribe → 貼上前景 app
- Groq fallback: 無 model 下載 / 連網 / on-device 失敗時 → 雲端 Whisper API
- LLM enhance: Groq chat clean-up (optional toggle, default on)

沒有的 (v0.1):
- iOS target (v0.2)
- Personal Dictionary / Glossary (v0.3)
- Settings 第二頁 (v0.1 settings 一頁能放完, 不能 = scope 漂)
- 多 hotkey / Toggle mode / 多模式 (一個 hold-to-record 即可)
- 多語言 UI (英文一種; 中文之後再說)

**Second-system effect 防禦**: v0.1 ship 之前不開 v0.2 任何 issue。v0.1 ship 定義 = Panda 自己連用 7 天不卡。

## Differentiation (誠實版)

**沒有差異化**。frame = 練手, 不是市場。

- WhisperKit on-device: argmaxinc 自己 demo 就有
- Groq fallback: openquack 已有, VoiceInk 已有
- Glossary: VoiceInk Personal Dictionary 已有
- macOS dictation hotkey: 整票
- iOS dictation: Apple 內建 + Whisper apps 上 App Store 也有

任何 brief / commit / PR / README 文案出現「differentiated」「moat」「unique」字眼 = 漂走 = stop 自查。

## Reference / inspiration (UX patterns only, NO code copy)

- **VoiceInk** (Beingpax, GPL v3, macOS AppKit) — Power Mode / Personal Dictionary / hotkey UX / app integration patterns
  - ⚠️ GPL 傳染風險: **只看 source 學設計, 不 copy code, 不 fork**
  - 若未來想 commercial / paid → 不能含 GPL-derived code, brief 提前 lock
- **openquack** (larryxiao, MIT, macOS menu-bar) — menu-bar UX / WhisperKit 整合 patterns
- **WhisperKit demo apps** (argmaxinc, MIT) — iOS audio capture + Core ML pipeline reference

只看, 不直接學 stack: VoiceInk 是 AppKit + whisper.cpp + macOS only, 跟「SwiftUI 現代 stack + iOS+macOS native」直接衝突。

## Stack (lock, 不在 Sprint 1-3 重新討論)

| Layer | Choice | Why |
|---|---|---|
| Language | Swift | Apple modern stack 練手目標 |
| UI | SwiftUI (macOS + iOS) | 跨平台 share-able, NOT AppKit, NOT Catalyst |
| Code sharing | Swift Package (`Core`) | iOS / macOS share core, separate UI targets |
| Transcription (on-device) | WhisperKit (argmaxinc, MIT) | 真 cross-platform iOS+macOS Swift framework, Core ML pipeline |
| Transcription (cloud fallback) | Groq Whisper API (`whisper-large-v3-turbo`) | 沿用舊 murmur-voice 經驗, 已有 API key UX |
| LLM enhance | Groq chat completions | Same |
| Audio capture | AVAudioEngine | iOS + macOS 通吃; cpal Rust 不適用 Swift |
| Distribution | TestFlight (iOS) + signed DMG (macOS) | 不上 Mac App Store / 不上 iOS App Store paid; dogfood 圈直接裝 |

Stack 重新討論 trigger: WhisperKit 1.x API 大幅 breaking, 或 argmaxinc 停維護。其他都不準在 Sprint 1-3 改。

## Hard kill criteria (numeric, time-bound, binary)

### Primary kill trigger (single, dominant)

**2026-08-11 之前沒有 dogfoodable iOS TestFlight build 在 Panda 自己 iPhone 上連跑 7 天不 crash → kill**。

定義 ("dogfoodable iOS TestFlight build"):
- 上得了 TestFlight (Apple review 通過 internal testing)
- Panda 自己 iPhone 裝得進 + 啟動不 crash
- 至少能完成「按 hotkey → 錄音 → WhisperKit transcribe → 貼上」一個 flow (macOS 版同 flow 也要過)
- 連續 7 天 (≥ 5 天 mobile 觸發 ≥ 5 次/天)

達不到 = kill = archive repo, 公開 retro 寫「為什麼 3 個月內 ship iOS native 失敗」, 切回舊 murmur-voice desktop 繼續用。

### Secondary review triggers (任一觸發 = 強制 review, 不一定 kill)

- 連續 2 個月 monthly check-in 答「沒有真的打開 repo 寫 code」
- WhisperKit 1.x 出現 breaking change 且 1 週內沒 fix path
- argmaxinc 公開停止維護 WhisperKit
- Panda 自己連 14 天 mobile 觸發 < 1 次/天 (= 假設的 user problem 不存在)
- 5 月寫的這份 BRIEF 在 2 週內 (2026-05-25 之前) 又想推翻 → 觸發舊 BRIEF 5-day-drift pattern, 強制重 office-hours

Review 結果三選一:
1. **Continue**: 續寫一份新 BRIEF, window 再 +3 個月
2. **Pivot**: scope 重定義 (frame 從練手變正式 product, 但要先答 「市場差異化」這條 — 對應目前 differentiation = 沒有 = 須誠實補)
3. **Kill**: archive repo, 寫公開 retro

## Failure mode prediction (product-lead iron law: 預測失敗)

最可能失敗的場景, 按可能性排序:

1. **學 Swift / SwiftUI / WhisperKit 卡 6+ 週, MVP v0.1 macOS 都還沒 ship** — 學習曲線 underestimate
   - 防禦: v0.1 限 macOS only, 一個按鈕一個 flow; v0.1 卡 6 週就觸發 review
2. **WhisperKit iOS on-device accuracy 不夠用** (iPhone neural engine 跑 tiny/base 太爛, 跑 small/medium 太慢/占空間)
   - 防禦: Sprint 1 dojo 階段研究 WhisperKit model size + iOS benchmark; Groq cloud fallback 永遠在線
3. **iOS dictation 場景 reality check 失敗** (Panda 真的不會在 mobile 用語音轉文字 — 通勤吵 / 路上不方便講話 / 已習慣鍵盤輸入)
   - 防禦: 觸發 < 1 次/天 14 天 = secondary trigger, 強制 review
4. **App Store policy 2026 對 BYOK + cloud fallback 審核擋住** (Groq API key UX 要 user 自己填, Apple 可能不過)
   - 防禦: Sprint 2 開 repo 前 quick check Apple developer guidelines; 若擋, fallback 改 server-side proxy 或 disable cloud fallback
5. **舊 murmur-voice 朋友圈 dogfood 流失** (新 repo 太久沒 ship, 大家切回 typeless)
   - 防禦: 舊 murmur-voice 維持 bug fix mode 不停 (月 2-4hr budget); 不 archive 不凍結 quality gate

## Inheritance from 舊 murmur-voice

**完全不繼承 code / fixture / settings format**:
- `test/fixtures/` (Tauri cpal-recorded) → 不沿用, Sprint 3 重錄
- BRIEF.md / ROADMAP.md / COMPETITORS.md → 不繼承內容, 新 repo 從零
- Tauri / Rust 程式碼 → 完全棄, 新 repo 純 Swift

**只繼承哲學**:
- Dogfood-only, 不主動推廣
- Scope cap 3 個月 window, hard kill criteria
- 月 budget 上限 + 強制 review trigger
- Out-of-scope 明確 (不做 PMF / 不做 marketing / 不做差異化)
- Playback regression 概念 (但 fixture 重錄)

**舊 repo 後續**:
- bug fix mode 維持 (月 2-4hr, 沿用舊 BRIEF.md 2026-08-05 window)
- README 加 "successor: {新 repo name}" link (新 repo 有 dogfoodable build 後再加)
- 不 archive (留歷史 + 朋友圈現有 dogfood 仍能跑)
- 2026-08-05 review 時對齊新 repo kill date (2026-08-11), 一起決定走向

## Review schedule

- 2026-06-11 (1 個月後): self-checkpoint — Sprint 進度 + budget + 學習曲線
- 2026-07-11 (2 個月後): self-checkpoint — v0.1 macOS 是否 ship + iOS target 啟動狀態
- **2026-08-11 (3 個月後, hard kill date)**: continue / pivot / kill 三選一; **default = kill** if primary trigger not met
- 任一 secondary trigger 觸發: 立即 review, 不等月度節點

## Open questions (resolve in 後續 sprints, not blocking this BRIEF)

1. **新 repo name**: 沿用 "Murmur" 或新名? (低優, 開 repo 時定; 影響 brand + 對外溝通, 但 dogfood 圈不重要)
2. **WhisperKit model 大小 default**: tiny / base / small / medium — iOS app size 跟 accuracy tradeoff。Sprint 1 dojo 階段研究。
3. **Glossary 形狀**: user 編字典 / LLM correction rules / 主動學習 / 混合? VoiceInk Personal Dictionary 已是 user-edit + smart replace, 我們 form factor TBD。Sprint 3+ 處理, **NOT** v0.1 scope。
   - **加進來 gate (避免新 feature drift)**: v0.1 macOS + v0.2 iOS 都 ship 完 + Panda 自己連用 30 天 + 累積 ≥ 3 條明確 transcribe 錯誤 case 寫進 issue 才開始做。三個條件任一缺 = 還早, 別動。
4. **App Store policy on BYOK**: Sprint 2 開 repo 前 quick check; 若擋, fallback design adjust。
5. **舊 murmur-voice README "successor" 文案**: 等新 repo 有 dogfoodable build 再寫; 不在這個 BRIEF 處理。
6. **Hard kill 達成後的公開 retro 寫哪**: 個人 brain `learnings/` 或 blog 或不公開? 留到 2026-08-11 看當下決定。

## What this brief is NOT

- 不是 product spec (那是 ROADMAP / sprint output)
- 不是 marketing plan (frame = 練手, no marketing)
- 不是 ROADMAP.md 的取代 (ROADMAP 開 repo 後另寫)
- 不是 differentiation claim (沒有差異化, 誠實寫了)

是 scope cap + hard kill date, 避免做超過 + 避免 5-day 漂移再次發生。

## Authorship

- Authored: 2026-05-11
- Origin: `/sprint murmur-voiceink-pivot-brief` Stage 3 (product-lead persona)
- Input brief: `docs/briefs/2026-05-11-voiceink-based-pivot.md`
- Next sprint: `/sprint murmur-voiceink-repo-bootstrap` (eng-lead, scaffold new repo)
