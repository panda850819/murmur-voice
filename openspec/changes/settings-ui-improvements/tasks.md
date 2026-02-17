## 1. Backend: Settings + Commands

- [x] 1.1 Add `ui_locale` field to Settings struct in `src-tauri/src/settings.rs` with `#[serde(default = "default_en")]`, update Default impl, add backward-compat test
- [x] 1.2 Remove backend auto-hide thread in `src-tauri/src/lib.rs` (the `std::thread::spawn` block ~lines 446-457 that sleeps 10s then hides)
- [x] 1.3 Add `hide_overlay_windows` Tauri command in `src-tauri/src/lib.rs` that hides both preview and main windows, register in invoke_handler

## 2. Frontend: i18n System

- [x] 2.1 Create `src/i18n.js` with `I18N` translation maps (en + zh-TW, ~30 keys), `t(key)` helper, `applyLocale(locale)` DOM walker

## 3. Frontend: Settings HTML

- [x] 3.1 Add `<script src="/i18n.js"></script>` before settings.js in `src/settings.html`
- [x] 3.2 Add `data-i18n` attributes to all static labels in settings.html (~30 elements)
- [x] 3.3 Add UI Language `<select>` row in System section (after Launch at Login)
- [x] 3.4 Add Changelog row in System section (after Updates)
- [x] 3.5 Add vision footer (tagline + roadmap link) inside `<footer>` before actions

## 4. Frontend: Settings CSS

- [x] 4.1 Fix update badge: add `min-width: 120px` and `text-align: center` to `.update-btn`
- [x] 4.2 Add `.link-btn` style (accent-colored text button on elevated bg)
- [x] 4.3 Add `.vision` footer style (muted 10px italic, centered, accent link)

## 5. Frontend: Settings JS

- [x] 5.1 Load `ui_locale` from settings and call `applyLocale()` on DOMContentLoaded
- [x] 5.2 Add change listener on `#ui-locale` select to live-switch locale
- [x] 5.3 Include `ui_locale` in `newSettings` object on save
- [x] 5.4 Add click handler for changelog button (open GitHub releases URL)
- [x] 5.5 Add click handler for roadmap button (open GitHub repo URL)

## 6. Frontend: Preview Auto-Hide

- [x] 6.1 In `src/preview.js`, on `transcription_complete` with mode "pasted": set `autoHideTimer = setTimeout(autoHide, 5000)` where `autoHide()` calls `invoke("hide_overlay_windows")`
- [x] 6.2 On blur of preview text: restart 5s auto-hide timer (if no pending edits)
- [x] 6.3 Remove old comment about backend handling auto-hide

## 7. Verification

- [x] 7.1 Run `cargo test` — all tests pass (16 existing + 1 new = 17)
- [x] 7.2 Run `cargo clippy --all-targets -- -D warnings` — zero warnings
