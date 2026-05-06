# Release Note Template (Telegram Channel)

## Template

```
🎙 Murmur Voice v{VERSION}

✦ {feature_1_benefit}
✦ {feature_2_benefit}
✦ {feature_3_benefit}

🐛 修復：{bug_fix_summary}

📥 下載：github.com/panda850819/murmur-voice/releases/tag/v{VERSION}
```

## Writing Rules

1. **One feature = one line**, describe the benefit not the implementation
2. **User language** — "轉錄更準" not "adjusted NO_SPEECH_THRESHOLD to 0.6"
3. **Bug fixes compressed** — all in one line, comma-separated
4. **Max 10 lines** total — Telegram messages get skipped if too long
5. **Download link last** — clear CTA at the bottom

## Feature Line Formula

```
✦ {what it is} — {why you should care}
```

Good: `✦ 內建術語辭典包 — Crypto、AI/ML 常用詞，開就能用，轉錄更準`
Bad:  `✦ Added dictionary packs with include_str! macro for crypto/ai-ml/dev-tools terminology`

## Optional Sections

For major releases (x.0.0), add before download link:

```
⚠️ 注意：{breaking_change_or_migration_note}
```

For releases with visual changes:

```
🖼 預覽：{screenshot_url}
```

## Example: v0.4.3

```
🎙 Murmur Voice v0.4.3

✦ 內建術語辭典包 — Crypto、AI/ML、開發者常用詞，開就能用，轉錄更準
✦ 翻譯快捷鍵支援多組合鍵 — Cmd+Shift+T 這類組合也行了
✦ 更強的防幻覺過濾 — 減少 Whisper 自己「腦補」出不存在的文字

🐛 修復：ESC 取消後預覽窗卡住、更新後重複跑 onboarding

📥 下載：github.com/panda850819/murmur-voice/releases/tag/v0.4.3
```
