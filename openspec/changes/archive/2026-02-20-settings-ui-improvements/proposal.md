## Why

v0.3.0 shipped with multi-provider LLM support but the settings UI has layout bugs (update badge alignment, preview auto-hide fires during editing) and lacks internationalization for broader audience reach. Adding UI language support and quality-of-life features (changelog, vision footer) improves polish.

## What Changes

- Fix update badge layout alignment in settings (CSS-only)
- Move preview auto-hide from backend thread timer to frontend JS timer, so editing cancels auto-hide
- Add UI language setting (English / zh-TW) with `data-i18n` attribute-based localization
- Add changelog button linking to GitHub releases
- Add vision tagline and roadmap link in settings footer

## Capabilities

### New Capabilities
- `ui-locale`: UI language switching (en / zh-TW) with DOM-based i18n via `data-i18n` attributes
- `settings-extras`: Changelog button and vision/roadmap footer in settings window

### Modified Capabilities
- `transcription-preview`: Auto-hide moves from backend 10s thread timer to frontend 5s JS timer; editing cancels timer, blur restarts it

## Impact

- **Backend** (`src-tauri/src/lib.rs`): Remove backend auto-hide thread, add `hide_overlay_windows` command
- **Backend** (`src-tauri/src/settings.rs`): New `ui_locale` field with backward-compatible serde default
- **Frontend** (`src/i18n.js`): New file — translation map + `applyLocale()` helper
- **Frontend** (`src/settings.html`): `data-i18n` attributes on ~30 elements, new rows and footer
- **Frontend** (`src/settings.js`): Locale load/save, changelog + roadmap click handlers
- **Frontend** (`src/settings.css`): Update badge fix, `.link-btn`, `.vision` styles
- **Frontend** (`src/preview.js`): Frontend auto-hide timer logic
