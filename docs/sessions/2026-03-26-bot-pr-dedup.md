---
date: 2026-03-26
branch: main
project: murmur-voice
tags: [coding-session, murmur-voice, ci]
---

# bot-pr-dedup — 2026-03-26

## What happened

Completed issue #94: Bolt/Sentinel bot PR dedup mechanism. Created `bot-pr-guard.yml` GitHub Actions workflow that auto-detects and closes duplicate bot PRs by comparing file overlap (>=50% threshold). Created `JULES.md` with severity calibration for desktop apps (prevents bots from marking local-only issues as CRITICAL) and PR submission rules (1 open PR per issue max). Ran /simplify review which caught script injection risk, unused outputs, loop-invariant recomputation, and temp file collision — all fixed before shipping.

## Current state

Shipped to main (735ada8). Issue #94 closed. Next time a Bolt/Sentinel bot opens a PR, the guard workflow will trigger automatically.

## Follow-ups

- [ ] Monitor first few bot PRs to verify the guard workflow fires correctly
- [ ] Consider adding a label (e.g. `bot-duplicate`) when closing, for easier tracking
