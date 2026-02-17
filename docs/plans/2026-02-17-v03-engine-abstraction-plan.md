# v0.3 Engine Abstraction + Local-First Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add multi-LLM provider support (Groq, Ollama, custom OpenAI-compatible) via a trait-based abstraction, enabling fully offline voice-to-text with AI enhancement.

**Architecture:** All three LLM providers are OpenAI-compatible, so a single `OpenAICompatibleEnhancer` struct covers all cases with different config presets. A `TextEnhancer` trait enables future non-OpenAI providers. The existing `process_text()` function is replaced by trait dispatch. Groq transcription (Whisper API) remains unchanged — transcription engine abstraction is deferred to v0.4.

**Tech Stack:** Rust (Tauri 2, reqwest, serde), vanilla JS/HTML/CSS (no bundler)

---

### Task 1: Add TextEnhancer Trait and OpenAICompatibleEnhancer

**Files:**
- Modify: `src-tauri/src/llm.rs` (add trait + struct after existing code)

**Step 1: Write test for enhancer trait bounds**

Add at the bottom of `src-tauri/src/llm.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhancer_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<OpenAICompatibleEnhancer>();
    }

    #[test]
    fn test_groq_preset() {
        let enhancer = OpenAICompatibleEnhancer::groq("test-key", "llama-3.3-70b-versatile");
        assert_eq!(enhancer.name(), "Groq");
        assert!(!enhancer.is_local());
        assert_eq!(enhancer.api_url, "https://api.groq.com/openai/v1/chat/completions");
    }

    #[test]
    fn test_ollama_preset() {
        let enhancer = OpenAICompatibleEnhancer::ollama("http://localhost:11434", "llama3.2");
        assert_eq!(enhancer.name(), "Ollama");
        assert!(enhancer.is_local());
        assert_eq!(enhancer.api_url, "http://localhost:11434/v1/chat/completions");
    }

    #[test]
    fn test_custom_preset() {
        let enhancer = OpenAICompatibleEnhancer::custom(
            "https://my-server.com/v1/chat/completions",
            "sk-123",
            "my-model",
        );
        assert_eq!(enhancer.name(), "Custom");
        assert!(!enhancer.is_local());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test`
Expected: FAIL — `OpenAICompatibleEnhancer` not defined

**Step 3: Add trait and struct**

In `src-tauri/src/llm.rs`, after the `strip_llm_prefix` function (line 230) and before the Groq Whisper section (line 232), add:

```rust
// --- TextEnhancer trait ---

/// Trait for LLM post-processing providers.
/// All methods are sync — implementations that need async should use `tokio::runtime::Runtime`.
pub(crate) trait TextEnhancer: Send + Sync {
    fn name(&self) -> &str;
    fn is_local(&self) -> bool;
    fn enhance(&self, text: &str, style: &str) -> Result<String, LlmError>;
}

/// OpenAI-compatible LLM provider. Covers Groq, Ollama, and any custom endpoint.
pub(crate) struct OpenAICompatibleEnhancer {
    pub api_url: String,
    api_key: String,
    model: String,
    local: bool,
    provider_name: String,
}

impl OpenAICompatibleEnhancer {
    pub fn groq(api_key: &str, model: &str) -> Self {
        Self {
            api_url: "https://api.groq.com/openai/v1/chat/completions".to_string(),
            api_key: api_key.to_string(),
            model: model.to_string(),
            local: false,
            provider_name: "Groq".to_string(),
        }
    }

    pub fn ollama(base_url: &str, model: &str) -> Self {
        let url = base_url.trim_end_matches('/');
        Self {
            api_url: format!("{url}/v1/chat/completions"),
            api_key: String::new(),
            model: model.to_string(),
            local: true,
            provider_name: "Ollama".to_string(),
        }
    }

    pub fn custom(api_url: &str, api_key: &str, model: &str) -> Self {
        Self {
            api_url: api_url.to_string(),
            api_key: api_key.to_string(),
            model: model.to_string(),
            local: false,
            provider_name: "Custom".to_string(),
        }
    }
}

impl TextEnhancer for OpenAICompatibleEnhancer {
    fn name(&self) -> &str {
        &self.provider_name
    }

    fn is_local(&self) -> bool {
        self.local
    }

    fn enhance(&self, text: &str, style: &str) -> Result<String, LlmError> {
        let (protected_text, placeholders) = protect_english(text);

        let max_tokens = (protected_text.len() * 2).clamp(256, 2048) as u64;

        let mut body = serde_json::json!({
            "model": &self.model,
            "messages": [
                {
                    "role": "system",
                    "content": build_system_prompt(style)
                },
                {
                    "role": "user",
                    "content": format!("[Raw transcription to clean up]\n{protected_text}")
                }
            ],
            "temperature": 0.1,
            "max_tokens": max_tokens,
        });

        // Only add frequency_penalty for non-Ollama providers (Ollama may not support it)
        if !self.local {
            body["frequency_penalty"] = serde_json::json!(1.5);
        }

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| LlmError::Api(format!("failed to create runtime: {e}")))?;

        let response = rt.block_on(async {
            let client = reqwest::Client::new();
            let mut req = client
                .post(&self.api_url)
                .header("Content-Type", "application/json")
                .json(&body);

            if !self.api_key.is_empty() {
                req = req.header("Authorization", format!("Bearer {}", self.api_key));
            }

            req.send().await
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let body_text = rt.block_on(response.text()).unwrap_or_default();
            return Err(LlmError::Api(format!("{status}: {body_text}")));
        }

        let chat: ChatResponse = rt.block_on(response.json())?;
        let content = chat
            .choices
            .into_iter()
            .next()
            .ok_or(LlmError::Format)?
            .message
            .content;

        let cleaned = strip_llm_prefix(content.trim());

        let result = if placeholders.is_empty() {
            cleaned.to_string()
        } else {
            restore_english(cleaned, &placeholders)
        };

        Ok(result)
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cd src-tauri && cargo test`
Expected: All tests pass (existing 6 state tests + 4 new enhancer tests)

**Step 5: Commit**

```bash
git add src-tauri/src/llm.rs
git commit -m "feat(llm): add TextEnhancer trait and OpenAICompatibleEnhancer"
```

---

### Task 2: Add Enhancer Factory Function

**Files:**
- Modify: `src-tauri/src/llm.rs` (add factory after OpenAICompatibleEnhancer impl)
- Modify: `src-tauri/src/settings.rs` (add new fields)

**Step 1: Write test for factory function**

Add to the `tests` module in `src-tauri/src/llm.rs`:

```rust
    #[test]
    fn test_create_enhancer_disabled() {
        let s = Settings::default();
        assert!(create_enhancer(&s).is_none());
    }

    #[test]
    fn test_create_enhancer_groq() {
        let mut s = Settings::default();
        s.llm_enabled = true;
        s.groq_api_key = "gsk_test".to_string();
        s.llm_provider = "groq".to_string();
        let enhancer = create_enhancer(&s);
        assert!(enhancer.is_some());
        assert_eq!(enhancer.unwrap().name(), "Groq");
    }

    #[test]
    fn test_create_enhancer_ollama() {
        let mut s = Settings::default();
        s.llm_enabled = true;
        s.llm_provider = "ollama".to_string();
        let enhancer = create_enhancer(&s);
        assert!(enhancer.is_some());
        let e = enhancer.unwrap();
        assert_eq!(e.name(), "Ollama");
        assert!(e.is_local());
    }

    #[test]
    fn test_create_enhancer_groq_no_key() {
        let mut s = Settings::default();
        s.llm_enabled = true;
        s.llm_provider = "groq".to_string();
        s.groq_api_key = String::new();
        // Groq without API key returns None
        assert!(create_enhancer(&s).is_none());
    }
```

**Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test`
Expected: FAIL — `create_enhancer` not defined, new Settings fields missing

**Step 3: Add new fields to Settings**

In `src-tauri/src/settings.rs`, add these fields to the `Settings` struct (after `llm_model` at line 34):

```rust
    #[serde(default = "default_groq")]
    pub llm_provider: String,
    #[serde(default = "default_ollama_url")]
    pub ollama_url: String,
    #[serde(default = "default_ollama_model")]
    pub ollama_model: String,
    #[serde(default)]
    pub custom_llm_url: String,
    #[serde(default)]
    pub custom_llm_key: String,
    #[serde(default)]
    pub custom_llm_model: String,
```

Add the default functions (after `default_true` at line 14):

```rust
fn default_groq() -> String {
    "groq".to_string()
}

fn default_ollama_url() -> String {
    "http://localhost:11434".to_string()
}

fn default_ollama_model() -> String {
    "llama3.2".to_string()
}
```

Update the `Default` impl to include the new fields (after `llm_model` at line 53):

```rust
            llm_provider: default_groq(),
            ollama_url: default_ollama_url(),
            ollama_model: default_ollama_model(),
            custom_llm_url: String::new(),
            custom_llm_key: String::new(),
            custom_llm_model: String::new(),
```

**Step 4: Add factory function to llm.rs**

After the `TextEnhancer` impl block, add:

```rust
/// Creates the appropriate TextEnhancer based on current settings.
/// Returns None if LLM is disabled or required config is missing.
pub(crate) fn create_enhancer(settings: &crate::settings::Settings) -> Option<Box<dyn TextEnhancer>> {
    if !settings.llm_enabled {
        return None;
    }

    match settings.llm_provider.as_str() {
        "groq" => {
            if settings.groq_api_key.is_empty() {
                return None;
            }
            Some(Box::new(OpenAICompatibleEnhancer::groq(
                &settings.groq_api_key,
                &settings.llm_model,
            )))
        }
        "ollama" => Some(Box::new(OpenAICompatibleEnhancer::ollama(
            &settings.ollama_url,
            &settings.ollama_model,
        ))),
        "custom" => {
            if settings.custom_llm_url.is_empty() {
                return None;
            }
            Some(Box::new(OpenAICompatibleEnhancer::custom(
                &settings.custom_llm_url,
                &settings.custom_llm_key,
                &settings.custom_llm_model,
            )))
        }
        _ => None,
    }
}
```

Add `use crate::settings::Settings;` at the top of the tests module.

**Step 5: Run tests to verify they pass**

Run: `cd src-tauri && cargo test`
Expected: All tests pass (6 state + 8 enhancer/factory)

**Step 6: Commit**

```bash
git add src-tauri/src/llm.rs src-tauri/src/settings.rs
git commit -m "feat(llm): add enhancer factory with Groq/Ollama/Custom presets"
```

---

### Task 3: Refactor do_stop_recording to Use TextEnhancer

**Files:**
- Modify: `src-tauri/src/lib.rs:347-390` (LLM post-processing section)

**Step 1: Replace hardcoded Groq call with trait dispatch**

In `src-tauri/src/lib.rs`, replace the LLM post-processing block (lines 347-390). Currently:

```rust
    // LLM post-processing (if enabled and API key present)
    let (llm_enabled, api_key, llm_model, app_aware_style) = state
        .settings
        .lock()
        .map(|s| (
            s.llm_enabled,
            s.groq_api_key.clone(),
            s.llm_model.clone(),
            s.app_aware_style,
        ))
        .unwrap_or_else(|_| (false, String::new(), String::new(), false));
    ...
```

Replace with:

```rust
    // LLM post-processing via TextEnhancer trait
    let (enhancer, app_aware_style) = {
        let s = state
            .settings
            .lock()
            .map_err(|e| format!("settings mutex poisoned: {e}"))?;
        (llm::create_enhancer(&s), s.app_aware_style)
    };

    eprintln!("[whisper raw] {}", raw_text);

    let text = if let Some(enhancer) = enhancer {
        if raw_text.is_empty() {
            raw_text
        } else {
            let _ = state
                .app_state
                .transition(state::RecordingState::Processing);
            let _ = app.emit("recording_state_changed", "processing");

            // Emit enhancer info for data flow indicator
            let _ = app.emit(
                "enhancer_info",
                serde_json::json!({
                    "name": enhancer.name(),
                    "local": enhancer.is_local(),
                }),
            );

            let style = if app_aware_style {
                frontapp::foreground_app_bundle_id()
                    .as_deref()
                    .map(frontapp::style_for_app)
                    .unwrap_or("default")
            } else {
                "default"
            };

            match enhancer.enhance(&raw_text, style) {
                Ok(processed) => {
                    eprintln!("[llm output] {}", processed);
                    processed
                }
                Err(e) => {
                    log::error!("LLM post-processing failed: {}", e);
                    let _ = app.emit(
                        "recording_error",
                        format!("LLM processing failed, using raw text: {e}"),
                    );
                    raw_text
                }
            }
        }
    } else {
        raw_text
    };
```

**Step 2: Remove the old `process_text` function**

In `src-tauri/src/llm.rs`, delete the old `process_text` function (lines 141-208). It's now replaced by `OpenAICompatibleEnhancer::enhance()`.

**Step 3: Verify with cargo check**

Run: `cd src-tauri && cargo check`
Expected: compiles. If there are unused import warnings for the old `process_text`, clean them up.

**Step 4: Run all tests**

Run: `cd src-tauri && cargo test`
Expected: All tests pass

**Step 5: Run clippy**

Run: `cd src-tauri && cargo clippy --all-targets -- -D warnings`
Expected: Zero warnings

**Step 6: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/src/llm.rs
git commit -m "refactor(llm): replace hardcoded Groq call with TextEnhancer trait dispatch"
```

---

### Task 4: Update Settings Frontend — LLM Provider Selection

**Files:**
- Modify: `src/settings.html:91-116` (AI Processing section)
- Modify: `src/settings.js` (add provider visibility logic, save/load new fields)

**Step 1: Update settings.html AI Processing section**

Replace the AI Processing section (lines 91-116) with:

```html
      <section class="group">
        <div class="group-label">AI Processing</div>
        <div class="group-card">
          <div class="row">
            <span class="row-label">LLM Post-Processing</span>
            <label class="toggle">
              <input type="checkbox" id="llm-enabled" />
              <span class="toggle-track"></span>
            </label>
          </div>
          <div id="llm-settings" style="display: none;">
            <div class="row">
              <span class="row-label">Provider</span>
              <select id="llm-provider">
                <option value="groq">Groq (Cloud)</option>
                <option value="ollama">Ollama (Local)</option>
                <option value="custom">Custom Endpoint</option>
              </select>
            </div>
            <div id="groq-llm-section">
              <div class="row">
                <span class="row-label">API Key</span>
                <input type="password" id="groq-api-key" placeholder="gsk_..." spellcheck="false" />
              </div>
              <div class="row">
                <span class="row-label">Model</span>
                <select id="llm-model">
                  <option value="llama-3.3-70b-versatile">Llama 3.3 70B</option>
                  <option value="llama-3.1-8b-instant">Llama 3.1 8B (Fast)</option>
                </select>
              </div>
            </div>
            <div id="ollama-section" style="display: none;">
              <div class="row">
                <span class="row-label">URL</span>
                <input type="text" id="ollama-url" placeholder="http://localhost:11434" spellcheck="false" />
              </div>
              <div class="row">
                <span class="row-label">Model</span>
                <input type="text" id="ollama-model" placeholder="llama3.2" spellcheck="false" />
              </div>
            </div>
            <div id="custom-llm-section" style="display: none;">
              <div class="row">
                <span class="row-label">Endpoint</span>
                <input type="text" id="custom-llm-url" placeholder="https://..." spellcheck="false" />
              </div>
              <div class="row">
                <span class="row-label">API Key</span>
                <input type="password" id="custom-llm-key" placeholder="sk-..." spellcheck="false" />
              </div>
              <div class="row">
                <span class="row-label">Model</span>
                <input type="text" id="custom-llm-model" placeholder="model-name" spellcheck="false" />
              </div>
            </div>
          </div>
          <div class="row">
            <span class="row-label">App-Aware Style</span>
            <label class="toggle">
              <input type="checkbox" id="app-aware-style" />
              <span class="toggle-track"></span>
            </label>
          </div>
        </div>
      </section>
```

Note: The Groq API Key field moved from the Transcription section into the AI Processing section under Groq provider. The old `groq-section` in the Transcription group (line 70-73) should be removed since the API key is now in AI Processing.

Wait — the Groq API key is shared between Whisper transcription and LLM. The transcription engine "groq" also needs the key. So we need to keep it accessible from the Transcription section OR share it from the AI Processing section.

Simplest approach: keep the `groq-api-key` input in the AI Processing section (under Groq provider). When the user selects engine="groq" for transcription, show a note that the API key is in AI Processing. This avoids duplicate inputs.

Update the Transcription engine section — replace the groq-section (lines 70-73) with:

```html
          <div class="row groq-hint" id="groq-section" style="display: none;">
            <span class="row-label"></span>
            <span class="row-hint">API Key is in AI Processing below</span>
          </div>
```

**Step 2: Update settings.js — add provider visibility and save/load**

In `src/settings.js`, add a new function after `updateLlmVisibility` (line 95):

```js
function updateLlmProviderVisibility() {
  const provider = el("llm-provider").value;
  el("groq-llm-section").style.display = provider === "groq" ? "block" : "none";
  el("ollama-section").style.display = provider === "ollama" ? "block" : "none";
  el("custom-llm-section").style.display = provider === "custom" ? "block" : "none";
}
```

Update `updateLlmVisibility` to also control the wrapper:

```js
function updateLlmVisibility() {
  const enabled = el("llm-enabled").checked;
  el("llm-settings").style.display = enabled ? "block" : "none";
  if (enabled) {
    updateLlmProviderVisibility();
  }
}
```

In the `DOMContentLoaded` settings load block (after line 198), add loading the new fields:

```js
    el("llm-provider").value = s.llm_provider || "groq";
    el("ollama-url").value = s.ollama_url || "http://localhost:11434";
    el("ollama-model").value = s.ollama_model || "llama3.2";
    el("custom-llm-url").value = s.custom_llm_url || "";
    el("custom-llm-key").value = s.custom_llm_key || "";
    el("custom-llm-model").value = s.custom_llm_model || "";
```

Add event listener for provider change (after line 222):

```js
  // LLM provider toggle
  el("llm-provider").addEventListener("change", updateLlmProviderVisibility);
```

In the save handler, update `newSettings` (after line 274) to include new fields:

```js
      llm_provider: el("llm-provider").value,
      ollama_url: el("ollama-url").value,
      ollama_model: el("ollama-model").value,
      custom_llm_url: el("custom-llm-url").value,
      custom_llm_key: el("custom-llm-key").value,
      custom_llm_model: el("custom-llm-model").value,
```

**Step 3: Add CSS for the hint text**

Append to `src/settings.css`:

```css
/* -- Groq hint -- */
.row-hint {
  font-size: 11px;
  color: var(--text-secondary);
  font-style: italic;
}
```

**Step 4: Verify with cargo check**

Run: `cd src-tauri && cargo check`
Expected: compiles (settings struct has new fields, frontend references them)

**Step 5: Manual verification**

1. Run `pnpm tauri dev`
2. Open Settings
3. Toggle LLM enabled → provider section appears
4. Switch provider dropdown: Groq shows API key + model; Ollama shows URL + model; Custom shows endpoint + key + model
5. Save and reopen → values persist

**Step 6: Commit**

```bash
git add src/settings.html src/settings.js src/settings.css src-tauri/src/settings.rs
git commit -m "feat(settings): add LLM provider selection (Groq/Ollama/Custom)"
```

---

### Task 5: Add Data Flow Indicator

**Files:**
- Modify: `src/preview.html` (add badge area)
- Modify: `src/preview.js` (listen for `enhancer_info` event)
- Modify: `src/preview.css` (badge styles)
- Modify: `src/index.html` (add badge to main bar)
- Modify: `src/main.js` (listen for `enhancer_info` event)
- Modify: `src/main.css` (badge styles)

**Step 1: Add data flow badge to preview window**

In `src/preview.js`, inside the `DOMContentLoaded` handler, add a listener for the new `enhancer_info` event (after the `transcription_complete` listener):

```js
  await listen("enhancer_info", (event) => {
    const { name, local } = event.payload;
    const badge = document.getElementById("app-badge");
    if (badge) {
      badge.textContent = local ? `${name} (Local)` : `${name} (Cloud)`;
      badge.className = "app-badge " + (local ? "badge-local" : "badge-cloud");
    }
  });
```

**Step 2: Add badge styles to preview.css**

Append to `src/preview.css`:

```css
/* -- Data flow badge -- */
.badge-local {
  background: rgba(52, 199, 89, 0.15) !important;
  color: #34c759 !important;
}

.badge-cloud {
  background: rgba(94, 92, 230, 0.15) !important;
  color: #a8a6ff !important;
}
```

**Step 3: Also emit engine type for transcription**

In `src-tauri/src/lib.rs`, in `do_stop_recording`, after the transcription engine type is determined (around line 292), emit the transcription engine info:

```rust
    let _ = app.emit(
        "transcription_engine_info",
        serde_json::json!({
            "engine": &engine_type,
            "local": engine_type != "groq",
        }),
    );
```

**Step 4: Manual verification**

1. Run `pnpm tauri dev`
2. Set LLM provider to Groq → record → preview shows "Groq (Cloud)" badge
3. Set LLM provider to Ollama → record → preview shows "Ollama (Local)" badge
4. Disable LLM → no enhancer badge shown

**Step 5: Commit**

```bash
git add src/preview.js src/preview.css src-tauri/src/lib.rs
git commit -m "feat(ui): add data flow indicator showing Local/Cloud provider"
```

---

### Task 6: Migrate Existing Settings (Backward Compatibility)

**Files:**
- Modify: `src-tauri/src/settings.rs` (handle migration)

**Step 1: Write test for settings migration**

Add to `src-tauri/src/settings.rs` at the bottom:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_legacy_settings() {
        // Existing settings without llm_provider field should default to "groq"
        let json = r#"{
            "ptt_key": "AltLeft",
            "language": "auto",
            "engine": "local",
            "model": "large-v3-turbo",
            "groq_api_key": "gsk_test",
            "window_opacity": 0.78,
            "auto_start": false,
            "llm_enabled": true,
            "llm_model": "llama-3.3-70b-versatile"
        }"#;

        let s: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(s.llm_provider, "groq");
        assert_eq!(s.ollama_url, "http://localhost:11434");
        assert_eq!(s.ollama_model, "llama3.2");
        assert!(s.custom_llm_url.is_empty());
    }

    #[test]
    fn test_deserialize_new_settings() {
        let json = r#"{
            "ptt_key": "AltLeft",
            "language": "auto",
            "engine": "local",
            "model": "large-v3-turbo",
            "groq_api_key": "",
            "window_opacity": 0.78,
            "auto_start": false,
            "llm_enabled": true,
            "llm_model": "llama-3.3-70b-versatile",
            "llm_provider": "ollama",
            "ollama_url": "http://192.168.1.100:11434",
            "ollama_model": "mistral"
        }"#;

        let s: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(s.llm_provider, "ollama");
        assert_eq!(s.ollama_url, "http://192.168.1.100:11434");
        assert_eq!(s.ollama_model, "mistral");
    }
}
```

**Step 2: Run test to verify**

Run: `cd src-tauri && cargo test`
Expected: PASS — serde `#[serde(default)]` handles missing fields automatically

**Step 3: Commit**

```bash
git add src-tauri/src/settings.rs
git commit -m "test(settings): add backward compatibility tests for LLM provider fields"
```

---

### Task 7: Final Verification

**Files:**
- All modified files

**Step 1: Run clippy**

Run: `cd src-tauri && cargo clippy --all-targets -- -D warnings`
Expected: Zero warnings

**Step 2: Run all tests**

Run: `cd src-tauri && cargo test`
Expected: All tests pass (6 state + 8 enhancer + 2 settings = 16 total)

**Step 3: Full manual test matrix**

| Scenario | Expected |
|----------|----------|
| LLM disabled → record | Raw transcription output, no enhancer badge |
| LLM enabled, Groq → record | Groq processes text, preview shows "Groq (Cloud)" |
| LLM enabled, Ollama (running) → record | Ollama processes text, preview shows "Ollama (Local)" |
| LLM enabled, Ollama (not running) → record | Error emitted, raw text used, error shown |
| LLM enabled, Custom endpoint → record | Custom processes text, preview shows "Custom (Cloud)" |
| Settings: switch provider → Save → Reopen | Provider selection persists |
| Settings: legacy file (no llm_provider) → load | Defaults to "groq", existing groq_api_key works |
| Transcription engine: local Whisper | Works as before |
| Transcription engine: Groq Whisper | Works as before (API key from AI Processing section) |
| Hold mode / Toggle mode | No regression |
| Smart clipboard / preview copy | No regression |

**Step 4: Remove dead code**

If the old `process_text()` function was not fully removed in Task 3, ensure it's gone. Check for any unused imports.

Run: `cd src-tauri && cargo clippy --all-targets -- -D warnings`

**Step 5: Commit any fixes**

```bash
git add -A
git commit -m "chore: cleanup unused code after LLM provider refactor"
```

---

## Summary

| Task | What | Files | Tests |
|------|------|-------|-------|
| 1 | TextEnhancer trait + OpenAICompatibleEnhancer | llm.rs | 4 |
| 2 | Factory function + Settings fields | llm.rs, settings.rs | 4 |
| 3 | Refactor do_stop_recording | lib.rs, llm.rs | 0 (integration) |
| 4 | Frontend settings UI | settings.html/js/css, settings.rs | 0 (manual) |
| 5 | Data flow indicator | preview.js/css, lib.rs | 0 (manual) |
| 6 | Settings backward compat | settings.rs | 2 |
| 7 | Final verification | all | full matrix |

**Total new tests:** 10
**Key outcome:** Users can select Groq, Ollama, or custom OpenAI-compatible endpoint for LLM post-processing. Ollama enables fully offline operation (local Whisper + local LLM).
