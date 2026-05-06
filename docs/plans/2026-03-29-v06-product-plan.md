# Murmur v0.6 產品計劃

> 決策文件。定義做什麼、為什麼、不做什麼。工程細節另開。

## 1. 目標與定位

**定位**: Privacy-first 多模式語音工具
> 100% 本地，但不只是聽寫 — 翻譯、指令、改寫，一鍵切換。

**v0.6 的核心假設**: Murmur 的多模式能力已經存在（v0.5.0），但使用者看不到、找不到、不會用。v0.6 的工作是**讓既有能力被發現和使用**，同時修好最大的品質痛點（中英混輸）。

**一句話版本**: v0.6 = 可發現性 + 中英品質。不加新功能，把現有功能做到能用。

---

## 2. Feature 清單與優先級

### P0 -- 不做就不發版

| Feature | 使用者問題 (JTBD) | 為什麼 P0 |
|---------|-------------------|-----------|
| **中英混輸品質提升** | 「我講中英夾雜，出來的英文被改掉或消失」 | 台灣市場最大痛點。品質不到位，其他都白做 |
| **Main bar 顯示當前模式** | 「我不知道現在是聽寫還是翻譯模式」 | 使用者按了快捷鍵，完全沒有回饋告訴他進了什麼模式 |

### P1 -- v0.6 必須包含

| Feature | 使用者問題 (JTBD) | 為什麼 P1 |
|---------|-------------------|-----------|
| **Main bar 顯示 Local/Cloud 引擎** | 「我不確定我的資料有沒有上傳」 | Privacy-first 定位的信任基礎。使用者需要視覺確認 |
| **Transcribing/Processing 狀態區分** | 「轉譯完了嗎？還是在跑 AI？」 | 兩段延遲合在一起，使用者不知道等什麼 |
| **Error 恢復指引** | 「出錯了，然後呢？」 | 目前只顯示錯誤訊息，不告訴使用者該怎麼修 |
| **Settings 裡 LLM 功能的可發現性** | 「我不知道有翻譯/指令/改寫功能」 | 多模式是核心差異化，但深埋在 Settings 裡，等於不存在 |

### P2 -- 有時間就做

| Feature | 使用者問題 (JTBD) | 為什麼 P2 |
|---------|-------------------|-----------|
| **Onboarding 模型下載預期管理** | 「下載要多久？卡住了嗎？」 | 首次體驗問題，但只影響一次 |
| **快捷鍵設定的 UX 改善** | 「我不知道怎麼設每個模式的快捷鍵」 | 目前可設定但不直覺 |

---

## 3. Vibe Coding -- 決定：選項 B（程式碼語氣風格）

在 Settings 的「語氣風格」裡新增「程式碼」選項。選了之後，聽寫模式保留所有程式碼術語、變數名、CLI 指令不做修改。語音指令模式理解程式碼 context。

本質是現有 `technical` style 的強化版，開發量小（改 prompt），使用者不用學新概念。驗證成功後可延伸為自動偵測前景 App 切換（選項 C）。

---

## 4. 成功指標

**North Star Metric**: 每日活躍轉譯次數（daily transcriptions）

這個指標同時反映留存和價值交付。使用者每天用得越多，代表品質夠好、操作夠順。

| 指標 | 目標 | 衡量方式 |
|------|------|----------|
| 中英混輸準確率 | 英文單字零修改率 > 90% | 手動測試 20 句 benchmark（中英混合） |
| 模式使用分佈 | 翻譯/指令/改寫模式至少被 >10% 使用者啟用 | Settings 裡 hotkey 有設定值的比例（本地統計，不上傳） |
| LLM 啟用率 | > 30% 使用者開啟 LLM post-processing | Settings 裡 `llm_enabled: true` 的比例 |
| Error → 恢復率 | 使用者看到 error 後 60s 內重試 | 看 error 後是否有下一次 transcription |

**注意**: Murmur 是 privacy-first，不做雲端遙測。指標靠本地 log 分析和使用者回饋驗證。v0.6 不建立 analytics pipeline，用 benchmark test cases + GitHub issue 回饋作為 proxy。

---

## 5. 不做清單

| 不做 | 為什麼 |
|------|--------|
| 新的轉譯引擎（MLX、Candle） | 品質優先於速度。引擎切換是 v0.7+ 的事 |
| 雲端 analytics / 遙測 | 違反 privacy-first 定位 |
| 歷史紀錄 / 轉譯存檔 | 有價值但不緊急，不解決 v0.6 的核心問題 |
| Plugin / 擴展系統 | 願景文件明確列為 non-goal |
| iOS / Android | Desktop tool，不跨平台到行動端 |
| 新的 LLM provider 整合 | 三種 provider（Groq/Ollama/Custom）已足夠涵蓋所有場景 |
| 付費功能 / 授權機制 | 太早。先驗證 PMF |
| 語音助手 / 對話式 AI | 不是 Murmur 要解的問題 |

---

## 6. Settings 介面重構

基於 WisprFlow / Spokenly / Typeless / LazyTyper / Handy 競品研究。

### 6.1 架構改動

**Sidebar tab 導航取代單欄長捲動** (視窗 460x700 -> 580x700)

| Tab | 內容 |
|-----|------|
| **General** | Hotkeys (4 mode 各一組)、Recording Mode (Hold/Toggle)、Launch at Login、Opacity、UI Language |
| **Voice** | Engine & Model (Local/Groq)、Language、Dictionary Packs、Custom Dictionary、Text Replacement |
| **AI** | LLM Toggle + Provider 欄位 (漸進揭露)、App-Aware Style (LLM 開啟時才可見)、Translation Target Language |
| **About** | Version、Check for Updates |

### 6.2 互動改動

| 改動 | 為什麼 | 影響檔案 |
|------|--------|---------|
| **即時儲存** -- 移除 Save/Cancel，每個 input change/blur 直接存 | 所有競品 + macOS 都即時儲存，手動 Save 是反模式 | `settings.js` (拆 save handler), `settings.html` (移除按鈕) |
| **Groq API Key 統一** -- 只在 AI tab 出現一次，標註「同時用於雲端轉錄和 LLM」 | 消除跨 section 指路牌 | `settings.html` |
| **LLM 漸進揭露** -- 關閉時只顯示 toggle；開啟後展開 Provider 欄位 | 降低初次使用者認知負擔 | `settings.html`, `settings.js` |
| **API Key show/hide toggle** | 標準 UX pattern | `settings.html`, `settings.js` |

### 6.3 視覺改動

| 改動 | 為什麼 | 影響檔案 |
|------|--------|---------|
| **Accent `#6c5ce7` -> `#007AFF`** (macOS system blue) | 紫色 accent 是 AI slop 標誌；系統藍融入原生語言 | `settings.css`, `styles.css`, `preview.css`, `onboarding.css` |
| **`--text-muted` contrast 修正** | `#aeaeb2` on white = 2.3:1，不及格 AA | `settings.css` |
| **Toggle 補全 5 個缺失狀態** -- hover, active, disabled, loading, error | 目前只有 default + checked | `settings.css` |
| **`record-btn` 觸擊區 38px -> 44px** | 低於最小觸擊區標準 | `settings.css` |
| **加 `:focus-visible` 指示器** | 鍵盤導航無視覺回饋 | `settings.css` |
| **Engine badge (Local/Cloud)** -- Engine 選擇旁顯示隱私含義 badge | 隱私承諾即時可見 | `settings.html`, `settings.css` |

---

## 執行計劃

### Phase 1: 品質 (1 session)
- 中英混輸 benchmark test cases (20 句)
- LLM prompt 調整 + Vibe Coding「程式碼」語氣風格

### Phase 2: Settings 重構 (2-3 sessions)
- Sidebar tab 導航 + 資訊架構重組
- 即時儲存取代 Save/Cancel
- Groq API Key 統一 + LLM 漸進揭露
- Accent color + contrast + toggle states + a11y 修正

### Phase 3: Main bar & 狀態 (1 session)
- Main bar 模式/引擎顯示
- Transcribing/Processing 狀態區分
- Error 恢復指引
- Tray menu LLM toggle

### Phase 4: 收尾 (1 session)
- Onboarding 模型下載提示
- Groq key 取得引導
- E2E 驗證
