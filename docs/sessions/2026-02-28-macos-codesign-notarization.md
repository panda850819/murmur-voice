---
date: 2026-02-28
branch: main
project: murmur-voice
tags: [coding-session, murmur-voice, codesign, notarization]
---

# macOS Code Signing & Notarization Setup — 2026-02-28

## What we were doing
Set up proper Apple Developer code signing and notarization for the macOS build, replacing the previous adhoc signing (`APPLE_SIGNING_IDENTITY="-"`).

## Key Decisions
- **Hardened Runtime Entitlements**: Chose 3 entitlements — `allow-unsigned-executable-memory` (Metal/whisper JIT), `disable-library-validation` (dylib loading), `device.audio-input` (microphone). Non-sandboxed app, minimal entitlements.
- **G2 Sub-CA**: Selected G2 (not Previous Sub-CA) for the Developer ID certificate intermediary since Xcode version is well above 11.4.1.
- **tauri-action built-in signing**: Relied on `tauri-apps/tauri-action@v0` built-in certificate import + notarization (notarytool) instead of custom scripts.

## Problems & Solutions
- **Certificate identity mismatch**: First CI run failed with `certificate from APPLE_CERTIFICATE does not match provided identity`. Root cause: extra whitespace in `APPLE_SIGNING_IDENTITY` GitHub secret value. Fixed by re-entering the secret without leading/trailing spaces.
- **Export .p12 greyed out**: User couldn't select .p12 format when exporting certificate. Cause: was in "Certificates" category instead of "My Certificates" (which pairs cert + private key).

## Follow-ups
- [ ] Update MEMORY.md to reflect signed builds (no longer adhoc)
- [ ] Consider adding Apple Distribution cert setup for future iOS/TestFlight work

## Notes
- 6 GitHub Secrets needed: `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`
- `APPLE_PASSWORD` is an App-Specific Password (from appleid.apple.com), not account password
- Developer ID Application cert exported as .p12 from "My Certificates" in Keychain Access, then `base64 -i file.p12 | pbcopy`
- v0.3.5 is the first properly signed + notarized release
- User also has Apple Development and Apple Distribution certs for future iOS work
