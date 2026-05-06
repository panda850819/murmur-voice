#!/usr/bin/env bash
# scripts/playback.sh — fixture playback regression runner (BRIEF.md quality gate).
#
# Runs whisper-cli over test/fixtures/*.wav, diffs against .expected.txt,
# applies the sanity filter spec'd in test/fixtures/README.md.
#
# Usage:
#   scripts/playback.sh
#   WHISPER_MODEL=~/models/ggml-medium.bin scripts/playback.sh
#   WER_TOL=0.15 WHISPER_LANG=zh scripts/playback.sh
#
# Exit codes: 0 clean (or no fixtures), 1 regression, 2 doctor failure.
#
# whisper-cli params here intentionally don't match the app's anti-hallucination
# settings (set_temperature_inc(0.0), entropy_thold(2.4), no_speech_thold). Drift
# between CLI and app runtime is a known follow-up; sanity filter still catches
# the class of stupid output BRIEF.md cares about.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FIXTURES_DIR="${FIXTURES_DIR:-${REPO_ROOT}/test/fixtures}"
WER_TOL="${WER_TOL:-0.10}"
WHISPER_CLI="${WHISPER_CLI:-whisper-cli}"
WHISPER_MODEL="${WHISPER_MODEL:-${HOME}/.cache/whisper.cpp/ggml-base.bin}"
WHISPER_LANG="${WHISPER_LANG:-auto}"

shopt -s nullglob
fixtures=("${FIXTURES_DIR}"/*.wav)
if (( ${#fixtures[@]} == 0 )); then
  echo "no fixtures in ${FIXTURES_DIR}, skipping (record per test/fixtures/README.md)"
  exit 0
fi

for cmd in "${WHISPER_CLI}" python3; do
  command -v "${cmd}" >/dev/null 2>&1 || { echo "ERROR: ${cmd} not on PATH" >&2; exit 2; }
done
[[ -f "${WHISPER_MODEL}" ]] || { echo "ERROR: model not at ${WHISPER_MODEL} (set WHISPER_MODEL=<path>)" >&2; exit 2; }

fail=0
for wav in "${fixtures[@]}"; do
  base="${wav%.wav}"
  name="$(basename "${base}")"
  exp="${base}.expected.txt"
  if [[ ! -f "${exp}" ]]; then
    printf "FAIL %-30s  missing .expected.txt\n" "${name}"
    fail=$((fail + 1))
    continue
  fi

  raw="$("${WHISPER_CLI}" -m "${WHISPER_MODEL}" -f "${wav}" -l "${WHISPER_LANG}" -nt 2>/dev/null || true)"
  if [[ -z "${raw// }" ]]; then
    printf "FAIL %-30s  whisper-cli empty output\n" "${name}"
    fail=$((fail + 1))
    continue
  fi

  result="$(RAW="${raw}" REF_FILE="${exp}" WER_TOL="${WER_TOL}" python3 <<'PY'
import os, re

raw = os.environ["RAW"]
ref = open(os.environ["REF_FILE"], encoding="utf-8").read().strip()
tol = float(os.environ["WER_TOL"])

# safety net: strip timestamps if -nt was ignored by older whisper-cli
hyp = " ".join(
    s for s in (
        re.sub(r"^\[\d\d:\d\d:\d\d\.\d+\s*-->\s*\d\d:\d\d:\d\d\.\d+\]\s*", "", line).strip()
        for line in raw.splitlines()
    ) if s
).strip()

disallowed = [
    ("emoji",     re.compile(r"[\U0001F300-\U0001FAFF✀-➿]")),
    ("box",       re.compile(r"[─-╿]")),
    ("ctrl",      re.compile(r"[\x00-\x08\x0B\x0C\x0E-\x1F\x7F]")),
    ("rep_punct", re.compile(r"([!?.,])\1{3,}")),
]
hit = next((n for n, rx in disallowed if rx.search(hyp)), None)
if not hit:
    allowed = re.compile(
        r"[\s -~ -ÿĀ-ſ"
        r"　-〿぀-ゟ゠-ヿ"
        r"一-鿿＀-￯]"
    )
    bad = next((c for c in hyp if not allowed.match(c)), None)
    if bad:
        hit = f"block:U+{ord(bad):04X}"

def tok(s):
    out = []
    for w in s.split():
        if re.search(r"[぀-ヿ一-鿿]", w):
            out.extend(list(w))
        else:
            out.append(w.lower())
    return out

def wer(r, h):
    if not r:
        return 0.0 if not h else 1.0
    n, m = len(r), len(h)
    dp = list(range(m + 1))
    for i in range(1, n + 1):
        prev, dp[0] = dp[0], i
        for j in range(1, m + 1):
            cur = dp[j]
            dp[j] = prev if r[i - 1] == h[j - 1] else 1 + min(dp[j], dp[j - 1], prev)
            prev = cur
    return dp[m] / n

w = wer(tok(ref), tok(hyp))
reasons = []
if hit:
    reasons.append(f"sanity:{hit}")
if w > tol:
    reasons.append(f"wer:{w:.3f}>{tol:.2f}")
if reasons:
    print("FAIL\t" + ",".join(reasons))
else:
    print(f"PASS\twer={w:.3f}")
PY
)"
  status="${result%%	*}"
  detail="${result#*	}"
  printf "%s %-30s  %s\n" "${status}" "${name}" "${detail}"
  [[ "${status}" == "FAIL" ]] && fail=$((fail + 1))
done

echo "---"
total=${#fixtures[@]}
echo "${total} fixtures, $((total - fail)) pass, ${fail} fail"
exit $(( fail > 0 ? 1 : 0 ))
