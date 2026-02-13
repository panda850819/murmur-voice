## Context

Murmur 目前的 transcription pipeline 是：`audio → Whisper → raw text → clipboard`。輸出是未經處理的原始文字，存在語助詞、簡體中文、無格式化等問題。需要在 Whisper 輸出後加入 LLM 後處理步驟，同時擴展錄音模式和設定選項。

現有架構：
- 後端 `lib.rs` 的 `do_stop_recording()` 直接將 Whisper 結果送入 clipboard
- `settings.rs` 管理所有設定，JSON 持久化
- `hotkey.rs` 使用 CGEventTap 偵測修飾鍵按放
- 前端 `settings.html/js` 提供設定 UI

## Goals / Non-Goals

**Goals:**
- Whisper 輸出經 LLM 處理後再插入（去語助詞、格式化、簡轉繁）
- 支援 toggle 錄音模式（按一次開始，再按一次停止）
- 使用者可設定個人字典提高辨識準確度
- 擴展語言選項至 Whisper 主要支援語言
- 偵測前景 app 調整輸出風格

**Non-Goals:**
- 本地 LLM 推論（Ollama）— 僅使用 Groq API
- 即時翻譯（語言 A 說 → 語言 B 輸出）
- 語音指令（如「刪除上一句」）
- 自訂 LLM provider（僅支援 Groq）

## Decisions

### D1: LLM 後處理使用 Groq API

**選擇**: Groq LLM API（Llama 4 Scout）
**替代方案**: OpenAI API、本地 Ollama
**理由**: 使用者已有 groq_api_key 欄位；Groq 推論速度極快（LPU），延遲低適合即時場景；免費 tier 足夠個人使用。不用本地 Ollama 是因為需要額外安裝和 GPU 記憶體佔用。

### D2: 後處理管線架構

**選擇**: 新增 `llm.rs` 模組，在 `do_stop_recording()` 的 Whisper 輸出後、clipboard 插入前呼叫。
**理由**: 單一插入點，不影響現有 recording/transcription 流程。LLM 處理是可選的（設定開關），關閉時行為與現在相同。

管線流程：
```
Whisper output → (if llm_enabled) → Groq API call → processed text → clipboard
                 (if disabled)    → raw text → clipboard
```

### D3: Toggle mode 實作方式

**選擇**: 在 `hotkey.rs` 的 event handler 層面實作，根據 `recording_mode` 設定決定行為。
**替代方案**: 在前端 JS 層面控制。
**理由**: hotkey handler 已經管理 press/release 邏輯，在同一層加 toggle 狀態最自然。recording_mode 從 settings 讀取。

```
Hold mode: Press → start, Release → stop（現有行為）
Toggle mode: Press → start/stop 切換, Release → 無動作
```

### D4: 個人字典注入方式

**選擇**: 將字典詞彙組合成 Whisper `initial_prompt` 參數。
**理由**: Whisper 的 initial_prompt 可以引導模型偏向特定用語（如公司名、技術術語）。不需要修改模型，只需在 `whisper.rs` 的 transcribe 呼叫時注入。

### D5: 前景 app 偵測

**選擇**: macOS `NSWorkspace.frontmostApplication` 透過 Objective-C FFI。
**替代方案**: AppleScript、CGEvent。
**理由**: NSWorkspace 是最直接的 API，已有 objc2 crate 在 Tauri 依賴鏈中。根據 app bundle ID 對應到預設 style preset。

### D6: 簡轉繁策略

**選擇**: 由 LLM 在後處理時一併轉換，不獨立使用 OpenCC。
**替代方案**: 加入 opencc-rust crate 做獨立轉換。
**理由**: LLM 後處理已經會重寫文字，一併處理簡轉繁更自然，不需要額外依賴。LLM 關閉時仍輸出 Whisper 原始結果（簡體），這是可接受的 trade-off。

## Risks / Trade-offs

- **[LLM 延遲]** → Groq 推論約 200-500ms，加上網路延遲。可接受，因為 Whisper 轉錄本身已需 1-3 秒。加入 loading 狀態提示使用者。
- **[API 依賴]** → 無網路時 LLM 後處理不可用 → fallback 到原始輸出，不阻斷使用。
- **[Groq 免費額度]** → 免費 tier 有速率限制 → 短文字處理消耗低，個人使用足夠。
- **[前景 app 偵測權限]** → 可能需要額外的 Accessibility 權限 → 已有此權限（CGEventTap 需要）。
- **[Toggle mode 誤觸]** → 忘記按停止導致長時間錄音 → 加入最大錄音時長限制（如 5 分鐘自動停止）。
