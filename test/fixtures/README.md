# Playback Regression Fixtures

Per `BRIEF.md` quality gate. Each fixture is an audio file + its expected transcript.

## Layout

```
test/fixtures/
  001-zh-meeting.wav         001-zh-meeting.expected.txt
  002-en-tech-talk.wav       002-en-tech-talk.expected.txt
  003-zh-en-mix.wav          003-zh-en-mix.expected.txt
  ...
```

## Naming

`NNN-<lang>-<scenario>.wav`

- `NNN` 3-digit sequence
- `lang` zh / en / zh-en (mix) / multi
- `scenario` short kebab tag: meeting / tech-talk / casual / noisy-cafe / quiet / dictation

## Expected file

Plain UTF-8 text, exact target transcript. Used for WER diff.

## Initial set (target: 5 → 20)

Start with the 5 recordings Panda uses most:

- [ ] 001 — typical zh meeting clip (Yei standup style)
- [ ] 002 — en tech talk (longer-form thought)
- [ ] 003 — zh-en mix (real Panda dictation pattern)
- [ ] 004 — noisy cafe / background music
- [ ] 005 — short dictation (<10s)

Grow to ~20 only when a regression escapes the existing set (add the missing case after fixing).

## Sanity filter assertion

Output must NOT contain:

- emoji or pictographs
- box drawing / line drawing chars (U+2500..U+257F)
- control chars except `\n` `\r` `\t`
- runs of repeated punctuation (`!!!`, `...???`, etc beyond 3)
- unicode blocks not in: Latin / CJK / Hiragana / Katakana / common punctuation

Triggering sanity filter on any fixture = regression. Fallback to raw transcript for that release.

## What this dir is NOT

- Not unit tests (those live in `src-tauri/`)
- Not user-facing samples
- Not for benchmarking competitors (that's `COMPETITORS.md` workflow)

Just the regression set. Keep it small and high-signal.
