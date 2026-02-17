## Context

Murmur Voice v0.3.0 settings UI works but has two bugs and lacks internationalization. The settings window is static HTML/JS/CSS with no build step. Backend is Rust/Tauri 2. Preview auto-hide currently runs as a backend `std::thread` timer that cannot be cancelled by frontend editing state.

## Goals / Non-Goals

**Goals:**
- Fix update badge alignment (CSS-only)
- Move auto-hide to frontend where editing state is known
- Add UI language switching (en / zh-TW) using DOM `data-i18n` attributes
- Add changelog button and vision footer to settings

**Non-Goals:**
- Full i18n framework (no ICU, no pluralization, no RTL)
- Translating preview window or main bar (only settings window)
- Dynamic locale detection from OS
- Translating option values (Groq, Ollama, engine names are proper nouns)

## Decisions

### 1. Frontend auto-hide instead of backend timer
**Decision**: Remove backend `std::thread::spawn` timer, add frontend `setTimeout` in preview.js.
**Rationale**: Frontend knows editing state (focus/blur). Backend timer fires regardless, causing the bug. New `hide_overlay_windows` Tauri command lets frontend hide both preview + main windows.
**Alternative considered**: Backend polling frontend state via events — adds unnecessary IPC complexity.

### 2. DOM-based i18n with `data-i18n` attributes
**Decision**: Single `i18n.js` file with translation maps. `applyLocale()` walks DOM for `[data-i18n]` elements.
**Rationale**: Settings window has ~30 static labels. No framework needed. Scales to 2-3 languages without complexity.
**Alternative considered**: i18next library — overkill for static HTML with no build step.

### 3. `ui_locale` persisted in settings.json
**Decision**: New `ui_locale: String` field with `#[serde(default = "default_en")]` for backward compatibility.
**Rationale**: Follows existing pattern (all new fields use serde defaults). Settings already round-trip through `get_settings`/`save_settings`.

### 4. Auto-hide timing: 5s instead of 10s
**Decision**: Reduce from 10s to 5s since user can now cancel by clicking into text.
**Rationale**: With reliable cancel-on-focus, shorter timeout feels more responsive. Old 10s was conservative because it couldn't be cancelled.

## Risks / Trade-offs

- [Frontend timer less precise than backend] → Acceptable for UX timer; not safety-critical
- [Adding i18n.js increases page load] → ~3KB, negligible for local file serving
- [zh-TW translations may need refinement] → Can iterate; initial set covers all visible labels
