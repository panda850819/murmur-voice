---
date: 2026-05-11
type: brief
source: office-hours
topic: murmur-voice → WhisperKit-based new product (VoiceInk UX patterns, iOS-capable)
tags: [brief, office-hours, murmur-voice, voice-to-text, pivot, whisperkit]
note: filename retains "voiceink-based" for path stability; content updated to WhisperKit-based per B1 post-research correction (2026-05-11 same day)
---

# Murmur Voice → WhisperKit-based New Product (VoiceInk UX patterns)

## Problem

帶進來的問題 frame 是「murmur-voice 太複雜, 要簡化、預設 Groq、TA 用、重設計 UI、換 Tauri、加未來功能 (倒裝/Glossary)」。

7 次 reframe grill 後 surface 的真正 frame：**murmur-voice 對你來說「練手」階段結束了, 你想開新一輪練手, 練 product-building 的 flow**。VoiceInk 是 inspiration 的「UX patterns」(Power Mode, Personal Dictionary, hotkey UX), 但**不是 stack base** — VoiceInk 是 Swift+AppKit+whisper.cpp+macOS only, 跟你的「iOS 不會動目標 + 學 SwiftUI 現代 stack」衝突。

「複雜」是 symptom，「練手到頭、要新挑戰」是 root cause。實際 stack base 是 **WhisperKit + SwiftUI** (cross-platform iOS+macOS), VoiceInk 提供 UX 設計參考。

## Original premise (我帶進來的)

- murmur-voice 多模式 (local whisper + Groq + Ollama + custom LLM + multi hotkey) 太複雜
- 預設 Groq, 簡化 settings 面板 + main bar
- 評估要不要繼續 Tauri
- 未來功能方向：自動修正、倒裝句調正、Glossary 矯正

## Revised premise (after 7-frame grill + VoiceInk reality check)

- 不是「簡化 murmur-voice」, 是「用 murmur-voice 練手結束、開新一輪練手」
- L1 goal = **練 product-building flow** (寫 BRIEF / ROADMAP / dogfood loop / release process / retro), 不是 ship 競品
- TA = 自己 + 小圈子 (跟 BRIEF.md `Users` 段一致：Panda + Yei/Sommet 同事 + 朋友圈)
- 不主動推廣、不做 PMF — BRIEF.md 哲學保留並繼承新 repo
- Stack 選擇從「Rust 為了速度」改成「**WhisperKit + Swift + SwiftUI** 為了學現代 Apple stack + cross-platform iOS/macOS」
- VoiceInk 是 **UX patterns 參考** (Power Mode, Personal Dictionary, hotkey UX, app integration), **stack 不直接學** — VoiceInk = AppKit + whisper.cpp + macOS only, 跟新方向衝突

## Alternatives considered

- **A: Dogfood 內最小簡化** → REJECT (你拒絕, "iOS 是不會動目標")
- **B: Mobile pivot 全押, desktop freeze, stack open** → DEFER (太重, 200hr 押在預期未來 dogfood)
- **C: Share Rust core, 兩端 evolve** → DEFER (你不想 share boundary)
- **D: Fork-and-rewrite, 新 repo, Rust + Flutter** → DEFER (你說「沒那麼複雜」)
- **E (a): 上游合進開源 host (e.g. Vibe)** → REJECT (跟你 maker motivation 「自己 craft + 自選 stack」直接衝突)
- **E (b): Fork host repo as Murmur spin** → DEFER
- **E (c): 純 adopt 別人, archive murmur, 你變 user** → DEFER (失去 craft 樂趣)
- **F (initial): VoiceInk-based 新產品** → SUPERSEDED by F (B1) — reality check 顯示 VoiceInk 不能 transfer 到 iOS, stack 要換
- **F (B1): WhisperKit-based 新產品 (chosen)** — Swift + SwiftUI + WhisperKit, macOS+iOS native, VoiceInk 只提供 UX patterns 參考

## Chosen approach

**F (B1): WhisperKit-based 新產品, Swift+SwiftUI, macOS + iOS native, VoiceInk UX patterns, 練手導向**

Architecture (高層, 細節在 /sprint dojo 階段 resolve)：

```
新 repo: TBD (暫定 murmur-voice-app, 開 repo 時定)
  
  Stack:
    - Swift + SwiftUI (現代 Apple stack, NOT AppKit despite VoiceInk 用 AppKit)
    - WhisperKit (argmaxinc, MIT) — cross-platform iOS+macOS Whisper on Core ML
    - macOS first (existing dogfood 群繼承), iOS native SwiftUI target 加做
      (NOT Catalyst — separate iOS target, share core via Swift Package)
    - Transcription engine: WhisperKit on-device default (Apple Neural Engine)
                          + Groq Whisper API cloud fallback (對應 BRIEF 「verifiable
                            privacy」哲學保留 — 默認本地, cloud 可選)
    - LLM enhance: Groq chat default + 未來可選 Apple Foundation Models / 本地
    - Glossary engine: 為 mobile UX 設計 (具體形狀待 dojo, VoiceInk Personal
      Dictionary 是參考形態, 但不是差異化護城河)
  
  Reference (UX patterns 學, stack 不學):
    - VoiceInk (Beingpax/VoiceInk, GPL v3, macOS-only AppKit)
      → Power Mode app-aware preset, Personal Dictionary, hotkey UX,
        app integration patterns
      → ⚠️ GPL 傳染風險: 不直接 fork / 不大量 copy code, 只學 design patterns
    - openquack (larryxiao, MIT, macOS menu-bar) — menu-bar UX, WhisperKit
      整合 patterns
    - WhisperKit demo apps (argmaxinc) — iOS-side audio capture + Core ML
      整合 reference
  
  舊 murmur-voice repo:
    - Bug fix mode (不開新 feature, BRIEF.md 月 budget 2-4hr 維持到 2026-08 review)
    - README 加 "successor: murmur-voice-app" link (新 repo 開後)
    - 不 archive (留歷史 + 朋友圈現有 dogfood 仍能跑)
```

理由：你最後一個 reply 直接 commit 並明示 frame = **「練手 + 練 product flow」**。VoiceInk 的 reality check 顯示「VoiceInk-stack」跟你「iOS 不會動目標」直接衝突, 所以「VoiceInk-inspired」這個 label 收窄成「VoiceInk UX patterns inspired」, stack base 改成 WhisperKit。

## Scope

**In:**
- 新 repo with 新 BRIEF.md (繼承舊 BRIEF.md 的 dogfood 哲學, 寫新 product 的 hard kill criteria)
- 學 Swift + SwiftUI + WhisperKit (這是 feature, 不是 cost — 練手 goal 之一)
- macOS minimum viable product (record → WhisperKit on-device transcribe → Groq enhance → paste/copy)
- iOS native SwiftUI target (share core via Swift Package, NOT Catalyst, NOT AppKit)
- Groq cloud fallback (沒下 model / 連網場景, 對應舊 BRIEF "verifiable privacy" 哲學保留)
- Glossary engine 為 mobile UX 設計 (form factor TBD: user 編字典 / LLM correction rules / 主動學習) — 注意 VoiceInk 已有 Personal Dictionary, 這條 **不是差異化**, 是 minimum viable feature
- 練 product flow: 寫 BRIEF / 寫 ROADMAP / release process / dogfood loop / retro / changelog

**Out:**
- Compete with typeless / superwhisper / VoiceInk (frame = 練手 not market)
- PMF / marketing / acquisition (BRIEF.md 哲學保留)
- AppKit (VoiceInk 用的 stack, 我們學 SwiftUI 現代 paradigm 不學 AppKit)
- whisper.cpp via Rust FFI (我們用 WhisperKit 的 Core ML pipeline, 純 Swift)
- Mac Catalyst (separate iOS native target 比較乾淨)
- "Verifiable privacy" 賣點作為**主打** (用 WhisperKit local default 保留這個能力, 但不打這個戰場)
- Android (iOS 走完 + dogfood 穩定再考慮, 跟你說的「一步一步來」一致)
- Rust (新 product 不用, 舊 murmur 保留 Rust 在 bug fix mode)
- Tauri (新 product 換 SwiftUI)
- Ollama / Custom LLM provider (Groq only at MVP, 不背複雜配置)
- Multi hotkey mode (一個 primary mode, dictation, 即可)
- 直接 fork VoiceInk (GPL 傳染 + 跟你 maker motivation 衝突)

## Next skill (recommended)

按 `lib/skill-decision-tree.md` 2-question test:

```
Shape: single-target-iterative (multi-sprint sequence)
Reasoning: Q1 = Yes (此為 multi-step 但每 step 都是 single-track iterative 
  work: 寫 BRIEF → repo scaffold → 學 Swift stub → MVP). Q2 = No (沒 
  N-branch wall-clock parallelism 需求).
```

**Pre-sprint research**: ✅ **DONE** (2026-05-11 via WebFetch + WebSearch)

Findings:
- VoiceInk = macOS AppKit only, 不可 transfer to iOS → stack base 改 WhisperKit
- VoiceInk 已有 Personal Dictionary → 你的 Glossary 不是差異化, 是基線 feature
- WhisperKit (MIT) 是真 iOS+macOS cross-platform Swift framework
- openquack (MIT) 是 WhisperKit-based macOS menu-bar 參考

額外 research 在 Sprint 1 dojo 階段視需要再跑:
- WhisperKit 1.x API stability, model size choice (tiny/base/small)
- App Store policy 2026 對 voice-transcription + BYOK 的審核細節

**Sprint sequence (推薦)**:

```
Sprint 1: /sprint murmur-voiceink-pivot-brief
  Persona: pandastack:product-lead
  Goal: 寫新 repo 的 BRIEF.md (繼承舊 BRIEF 哲學, 寫 hard kill criteria,
        定 3-month window, 寫 scope/out-of-scope 明確界線, 寫 dogfood
        criteria — e.g. "Panda 日均 ≥5 次 mobile 觸發")
  Output: 此檔 + 新 repo BRIEF.md drafted
  
Sprint 2: /sprint murmur-voiceink-repo-bootstrap
  Persona: pandastack:eng-lead
  Goal: 開新 repo, scaffold Swift + SwiftUI macOS app + Swift Package 
        for shared core, WhisperKit dependency added, hello-world build
        running, CI placeholder
  Output: 新 repo on GitHub + 第一個 SwiftUI macOS build + WhisperKit
          dependency green
  
Sprint 3+: /sprint murmur-voiceink-mvp-{slice}
  Persona: pandastack:eng-lead (主) + design-lead (UI 段)
  Goal: MVP feature slices:
        - audio capture (AVAudioEngine)
        - WhisperKit transcribe (on-device default)
        - Groq enhance LLM
        - paste / copy 系統整合
        - Glossary v0 (user 字典 minimal)
        - iOS target add (share core via Swift Package)
  Output: dogfoodable macOS build → 之後 iOS build → TestFlight
```

不要強行 team-orchestrate — 這條練手主線是 single-track sequential, 平行沒意義。

## Gotchas surfaced (from Stage 1 + grilling + post-write research)

1. **Self-commitment 漂移** — BRIEF.md 5/6 才 lock, 今天 (5/11) 5 天就推翻. 新 BRIEF **必須寫 hard kill criteria** (e.g. "3 個月內 dogfoodable iOS build on TestFlight, 否則 kill"), 否則同 pattern 會再次發生.

2. **"自己 craft + 自選 stack" 是深層 motivation** — 任何「合進別人 repo」path 都會 fail. 所以 E(a) reject, F (B1, 新 repo) 正確.

3. **Second-system effect** — 開新 repo 容易 over-engineer ("這次要做對"). 防禦: MVP timeline 寫死, "一個按鈕的最小 dictation app" 為 v0.1.

4. **學 Swift+SwiftUI+WhisperKit 是 explicit feature** — 不是 cost. 把學習 hours 算 budget 內, 不要假裝「等熟了再開始」.

5. **VoiceInk 是 UX inspiration, 不是 stack base** — 已驗證 (5/11 same day):
   - VoiceInk 是 AppKit + macOS only + whisper.cpp + GPL v3
   - 直接學它 stack = 學 AppKit (跟 SwiftUI 是兩套) + macOS-only (跟 iOS 目標衝突)
   - 真正的 stack base = WhisperKit + SwiftUI
   - 學 VoiceInk 的 UX patterns OK, copy code 會被 GPL 傳染

6. **舊 murmur-voice repo dogfood 流失風險** — 朋友圈現在用的是 desktop Tauri 版. 新 repo 跑出來前, 舊 repo 維持 bug fix mode 但**不要凍結 quality gate** (BRIEF.md 月 2-4hr 維持, playback regression 不退化).

7. **Glossary differentiation collapsed** — VoiceInk 已有 Personal Dictionary. 你「Glossary」不是 differentiation, 是 baseline feature. 「練手」frame 下 keep doing it 仍 OK, 但 brief 內絕不能 frame 為差異化護城河. 真差異化要找別處 (例如「主動學習 user 字典」「Apple Foundation Models 本地修正」, 但這些都是後話).

8. **GPL 傳染風險** — VoiceInk 是 GPL v3. 你 reference UX patterns 看 source OK, 但**不要 copy code**. 如果你 brief 後續想 commercial / Mac App Store paid release, 不能含 GPL-derived code. (但你 frame = 練手 + dogfood, 這條不是當前 blocker.)

## Gate Log

- Stage 1 (load context): BRIEF.md (5/6) + ROADMAP.md + brain plan (empty 模板) + project memory dir 空
- Stage 2 (premise challenge): 2 Q asked, push-once menu offered after Q1 (user bypassed, self-revealed mobile reframe). No escape-hatch fired explicit, but 7 次 reframe = implicit signal "still in ideation"
- Stage 3 (alternatives): A REJECT (隱性, iOS 不會動目標); B DEFER (太重); C DEFER (你不要 share boundary); D DEFER (「不那麼複雜」); E (a) REJECT (maker motivation 衝突 surface); E (b)/(c) DEFER; **F (initial) ADD** — VoiceInk-based 新產品
- Stage 4 (premise refresh): 原 simplify frame 廢, root cause = 練手 + product flow practice. BRIEF.md 哲學保留繼承
- Stage 5 (output): brief saved at `docs/briefs/2026-05-11-voiceink-based-pivot.md`. Path 2 no caveat (你 explicit 接受 commit 在當前資訊上, 不加 "verification" caveat).
- **Post-write reality check (same day, 2026-05-11)**: WebFetch + WebSearch VoiceInk findings 跟 brief 5 個衝撞 (Swift+SwiftUI vs AppKit / iOS via Catalyst vs macOS-only / Groq default vs whisper.cpp local / Glossary 差異化 vs Personal Dictionary 已有 / 未來功能 vs 已實裝). User picks Option B1 → brief updated to **WhisperKit-based, VoiceInk UX patterns reference only**. F (initial) → SUPERSEDED by F (B1).

## OPEN_QUESTIONS (not blocking brief, resolve in /sprint dojo)

1. **Glossary 具體形狀**: user 編字典 / LLM correction rules / 主動學習 / 混合? 影響 minimum-viable feature 設計. 注意 VoiceInk Personal Dictionary 是 user 編 + smart text replacements, 你需要在 Sprint 1 決定要不要照同樣 form factor.
2. **WhisperKit model 大小**: tiny/base/small/medium — iOS app size 跟 accuracy 的 tradeoff. Sprint 1 dojo 看 WhisperKit docs + benchmark 決定 default.
3. **新 repo 命名**: 沿用 "Murmur" 還是新名? 影響 brand + 對外溝通. 低優, 開 repo 時手動定.
4. **舊 murmur-voice 對外溝通**: README 加 successor 標籤具體寫什麼? "見 murmur-voice-app" or "this repo is legacy"? 等新 repo 有 dogfoodable build 再寫.
5. **Self-commitment 機制**: 新 BRIEF.md hard kill criteria 具體是 ("3 月內 ship TestFlight" / "月 budget 上限 X 小時" / "dogfood 觸發頻率 ≥ N/天")? **MUST resolve in Sprint 1**.
6. **App Store policy 風險**: BYOK + Groq cloud fallback 在 App Store 2026 政策下審核風險? (WhisperKit local default 應該降低風險, 但 Groq fallback 涉及 user-provided key UX). Resolve in Sprint 2 (開 repo 前 quick check Apple developer guidelines).

## Re-open trigger

If 在 2 週內 (2026-05-25) 出現任一條, 重開 office-hours:

- 你又 reframe 到「其實不是 WhisperKit-based」或「想做別的」
- WhisperKit API / 維護狀況有重大變化 (e.g. argmaxinc 不再維護)
- Sprint 1 寫新 BRIEF 過程中發現 scope 跟此 brief 衝突
- BRIEF.md 5/6 → 5/11 推翻的 5-day pattern 又出現 (新 BRIEF 寫完不到 1 週又想推翻)
