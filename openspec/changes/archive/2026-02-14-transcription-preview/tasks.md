## 1. Preview Window Setup

- [ ] 1.1 Add preview window configuration to `tauri.conf.json` (420x280, decorations false, transparent, always_on_top, visible false, skip_taskbar)
- [ ] 1.2 Add "preview" to `capabilities/default.json` windows array
- [ ] 1.3 Create `src/preview.html` with dark translucent layout (header area, text body, footer with character count + app badge)
- [ ] 1.4 Create `src/preview.css` matching main bar dark theme (rgba background, backdrop-filter blur, rounded corners)
- [ ] 1.5 Create `src/preview.js` with event listeners for `recording_state_changed`, `transcription_complete`, `live_transcription`, `foreground_app_info`

## 2. Backend: Foreground App Event

- [ ] 2.1 Add `foreground_app_info` event emission in `lib.rs` orchestration flow — emit app name and style category when transcription begins
- [ ] 2.2 Ensure `frontapp::foreground_app_bundle_id()` result and `style_for_app()` result are both available to emit (currently only style is used internally)

## 3. Preview Window Lifecycle

- [ ] 3.1 Add preview window show/hide logic in `lib.rs` — show on recording start, position above main bar
- [ ] 3.2 Implement auto-hide timer: 3 seconds after final result, minimum 1 second display
- [ ] 3.3 Handle new recording interrupting auto-hide (cancel timer, reset preview to "Listening...")
- [ ] 3.4 Ensure preview window does not steal focus (`focused: false` on creation/show)

## 4. Preview Window Content

- [ ] 4.1 Implement "Listening..." placeholder on recording start
- [ ] 4.2 Implement live transcription text updates (local engine only, listen to `live_transcription` event)
- [ ] 4.3 Implement "Transcribing..." / "Processing..." state indicators in header
- [ ] 4.4 Implement final result display with full text and character count
- [ ] 4.5 Implement "No speech detected" message for empty transcription results
- [ ] 4.6 Display foreground app name in footer when app-aware style is enabled

## 5. Main Bar: App Badge

- [ ] 5.1 Add app badge element to `src/index.html` (small label next to status text)
- [ ] 5.2 Style app badge in `src/styles.css` (subtle, muted color, small font)
- [ ] 5.3 Add `foreground_app_info` event listener in `src/main.js` to update badge
- [ ] 5.4 Clear app badge when returning to idle state

## 6. Testing & Polish

- [ ] 6.1 Test preview window with local Whisper engine (live preview + final result)
- [ ] 6.2 Test preview window with Groq engine (no live preview, only final result)
- [ ] 6.3 Test auto-hide timing and new-recording interruption
- [ ] 6.4 Test that preview window does not steal focus (Cmd+V pastes to correct app)
- [ ] 6.5 Test app badge display with different foreground apps (Slack, VS Code, unknown)
