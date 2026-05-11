---
date: 2026-05-11
type: sprint
state: SHIPPED
topic: murmur-voiceink-pivot-brief
mode: default
iteration: 2
persona: pandastack:product-lead
tags: [sprint, shipped, murmur-voice, brief, pivot, whisperkit]
---

# Sprint — murmur-voiceink-pivot-brief — 2026-05-11

## Capability probe

```
[1] AGENTS substrate    : ok    (~/.claude/CLAUDE.md present)
[2] vault root          : ok    (brain at /Users/panda/site/knowledge/brain/)
[3] lib/ files          : ok    (capability-probe / push-once / escape-hatch / gate-contract / skill-decision-tree)
[4] persona skills      : ok    (product-lead loaded)
[5] cli tools           : ok    (none required)
[6] write paths         : ok    (docs/briefs/ writable)

→ degraded: []   → blocked: []
```

## Stage progression

| Stage | Status | Output |
|---|---|---|
| 0 capability probe | ok | all 6 green |
| 1 dojo | done (inline, code-repo not vault) | 3 past cases surfaced, 5 gotchas carried |
| 2 grill (lite) | done | 3 Qs: output path / kill trigger / inheritance — all recommended chosen |
| 3 execute | done iter 1 | product-lead lens, draft written |
| 4 review | done iter 2 | P1-b + CG-1 patched; P1-a deferred to OPEN_QUESTIONS |
| 5 ship gate | SHIPPED | local commit, no push (solo planning artifact) |
| 6 terminal | SHIPPED | commit + backflow (feedback memory + this artifact) |

## Findings (review)

```
Iteration 1: P0=0 / P1=2 / COVERAGE GAP=1 / SCOPE DRIFT=0
  P1-a: Users 段沒寫「朋友圈為什麼會用」  → deferred to OPEN_QUESTIONS
  P1-b: 月 budget 15-25 hr 範圍太寬       → REPLACED with usage-based check-in
  CG-1: Glossary 沒寫 "when to add" gate  → PATCHED (3-condition gate added)

Iteration 2: P0=0 / P1=0 / COVERAGE GAP=0 / SCOPE DRIFT=0  → clean
```

## Gate log

- Stage 2 grill: 3 Qs, all Recommended options chosen (output path / kill trigger / inheritance)
- Stage 4 review: user replied "你不需要管這個要花多久時間，反正你評估時間都是錯的" — interpreted as drop hour-based budget, applied as Edit + saved as feedback memory
- Stage 5 ship: user chose SHIPPED commit + no push

## Backflow

1. **New feedback memory**: `~/.claude/projects/-Users-panda-site-apps-murmur-voice/memory/feedback_no-time-budgets-in-briefs.md` — "no hour-based budgets in BRIEFs for Panda's personal projects; anchor on usage / check-in / gates"
2. **Pattern lesson (not new, but reinforced)**: input-brief → product-lead-persona BRIEF draft is single-track, single-persona, ~2 iterations work — matches `lib/skill-decision-tree.md` Q1 = Yes path
3. **Drift defense added to draft**: BRIEF now contains its own "5-day drift detector" — if user wants to overturn within 2 weeks (2026-05-25), forced re-office-hours, not silent flip

## Terminal state: SHIPPED

- Draft at `docs/briefs/new-repo-BRIEF-draft.md` — copy into new repo root as `BRIEF.md` during Sprint 2 scaffold
- Input brief at `docs/briefs/2026-05-11-voiceink-based-pivot.md` (untracked → committed with this sprint)
- Sprint artifact: this file
- Memory: feedback_no-time-budgets-in-briefs.md (outside repo, in `~/.claude/projects/.../memory/`)

## OPEN_QUESTIONS (carried forward)

1. **P1-a from review**: BRIEF "Users" 段沒寫朋友圈 dogfood 動機。低優, Sprint 1 retro 時補, 或在 Sprint 2 開新 repo 時順手加。
2. **新 repo name**: TBD — Sprint 2 開 repo 時手動定 (沿用 "Murmur" 或新名)
3. **WhisperKit model 大小 default** (tiny/base/small/medium) — Sprint 1 dojo 階段研究 (本 Sprint 沒處理)
4. **Glossary 形狀** (user-edit / LLM rules / 主動學習) — v0.1+v0.2 ship 後再處理
5. **App Store policy on BYOK** — Sprint 2 開 repo 前 quick check
6. **舊 murmur-voice README "successor" 文案** — 新 repo 有 dogfoodable build 後寫
7. **Hard kill 達成後的公開 retro 寫哪** — 2026-08-11 看當下決定

## Next sprint (per input brief sequence)

```
/sprint murmur-voiceink-repo-bootstrap
  Persona: pandastack:eng-lead
  Goal: 開新 repo, scaffold Swift + SwiftUI macOS app + Swift Package
        shared core, WhisperKit dep added, hello-world build, CI placeholder.
  Pre-flight: 把 docs/briefs/new-repo-BRIEF-draft.md copy 進新 repo root 為 BRIEF.md
              (Sprint 1 不在 murmur-voice repo 內處理 cross-repo, 等 Sprint 2)
```
