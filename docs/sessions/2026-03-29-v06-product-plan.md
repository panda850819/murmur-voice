---
date: 2026-03-29
branch: main
project: murmur-voice
tags: [coding-session, murmur-voice, product-planning]
---

# v0.6 Product Plan — 2026-03-29

## What happened

Lex Tang (@lexrus) posted a comprehensive competitive review of the macOS voice input market, covering WisprFlow, Spokenly, Typeless, LazyTyper, Handy, and others. Product-lead agent analyzed the thread for positioning insights; design-lead agent audited Murmur's onboarding and settings UI against Lex's critiques. User confirmed positioning as "Privacy-first multi-mode" (option A), vibe coding as style enhancement (option B), and willingness to invest in mixed Chinese-English quality.

Follow-up competitive UI research compared settings interfaces across all 5 recommended products. Key finding: Murmur is the only product using manual Save/Cancel and single-column scrolling settings -- all competitors use instant save and tabbed/sidebar navigation. Design-lead produced a detailed settings redesign spec with sidebar tabs, instant save, accent color fix, and accessibility corrections.

## Current state

Product plan finalized at `docs/plans/2026-03-29-v06-product-plan.md` with 4 execution phases:
1. Mixed-language quality benchmark + LLM prompt tuning + vibe coding style
2. Settings UI restructure (sidebar tabs, instant save, accent, a11y)
3. Main bar state display + error guidance + tray toggle
4. Onboarding polish + E2E verification

## Follow-ups
- [ ] Phase 1: Build 20-sentence Chinese-English benchmark, tune LLM prompt
- [ ] Phase 2: Settings sidebar tab navigation + instant save refactor
- [ ] Phase 2: Accent color #6c5ce7 -> #007AFF across all CSS files
- [ ] Phase 3: Main bar mode/engine badge + processing state differentiation
