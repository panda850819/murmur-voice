## Why

Murmur 目前輸出的是 Whisper 原始轉錄文字，存在語助詞殘留、簡體中文輸出、中英混合品質差、缺乏格式化等問題。與 Typeless 等競品相比，缺少 LLM 後處理、toggle 錄音模式、個人字典等功能。需要將 Murmur 從「raw transcription tool」升級為「intelligent dictation tool」。

## What Changes

- 新增 LLM 後處理管線：transcription 完成後透過 Groq LLM API 清理文字（去語助詞、去重複、修正改口、自動格式化、簡轉繁）
- 新增 toggle 錄音模式：按一次 hotkey 開始錄音，再按一次停止（與現有 hold-to-talk 並存）
- 新增個人字典：使用者可設定常用專有名詞，注入 Whisper initial_prompt 提高辨識準確度
- 擴展語言選擇：從 3 種語言（auto/en/zh）擴展至 Whisper 支援的主要語言
- 新增 app-aware 輸出風格：偵測前景 app，根據情境（email、chat、code）調整 LLM prompt

## Capabilities

### New Capabilities
- `llm-post-processing`: LLM 後處理管線，包含去語助詞、去重複、改口修正、格式化、簡轉繁
- `toggle-recording`: 按一次開始/再按一次停止的錄音模式
- `personal-dictionary`: 使用者自訂專有名詞字典，注入 Whisper prompt
- `app-aware-style`: 偵測前景 app 並調整 LLM 輸出風格

### Modified Capabilities
- `speech-transcription`: 新增 LLM 後處理步驟，語言列表擴展
- `push-to-talk`: 新增 toggle mode 與現有 hold mode 並存

## Impact

- **後端**: settings.rs（新欄位）、lib.rs（LLM 管線、toggle 邏輯）、新增 llm.rs 模組
- **前端**: settings UI 擴展（字典、語言、LLM 開關、錄音模式選擇）
- **依賴**: 需要 Groq LLM API 呼叫（reqwest 已有）
- **設定檔**: settings.json 新增欄位（serde default 向後相容）
