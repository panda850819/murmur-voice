## Why

The current main bar (420x48) only shows a single truncated line of transcription text, making it impossible for users to see the full AI-processed result before it's pasted. Competing tools like Typeless show a full preview window with the complete processed text, character count, and detected foreground app. Adding a transcription preview window and app badge gives users confidence in what's being inserted and makes the app-aware style feature visible.

## What Changes

- **New preview window**: A floating dark-themed window that appears above the main bar during recording and transcription, showing the full processed text, character count, and auto-hiding after a delay
- **App badge in main bar**: Display the detected foreground app name/icon in the main bar so users can see which app-aware style is active
- **State machine update**: Add a "Processing" → result display flow to the preview window lifecycle

## Capabilities

### New Capabilities

- `transcription-preview`: Floating preview window that appears during recording, shows live transcription preview (local engine), displays full AI-processed result with character count, and auto-hides after configurable delay

### Modified Capabilities

- `app-lifecycle`: Main window gains an app badge display showing the detected foreground app. Preview window lifecycle (show on recording start, hide after result displayed)

## Impact

- **New files**: `src/preview.html`, `src/preview.css`, `src/preview.js` (new Tauri window)
- **Modified files**: `src-tauri/src/lib.rs` (window creation, event routing), `src-tauri/tauri.conf.json` (new window config), `src-tauri/capabilities/default.json` (add preview window), `src/index.html` + `src/styles.css` (app badge in main bar)
- **New events**: `preview_show`, `preview_hide`, `foreground_app_changed` (frontend events)
- **No breaking changes** to existing recording/transcription pipeline
