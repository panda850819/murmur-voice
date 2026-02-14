# Murmur

[![Release](https://img.shields.io/github/v/release/panda850819/murmur-voice?include_prereleases&style=flat-square)](https://github.com/panda850819/murmur-voice/releases)
[![CI](https://img.shields.io/github/actions/workflow/status/panda850819/murmur-voice/ci.yml?branch=main&style=flat-square&label=CI)](https://github.com/panda850819/murmur-voice/actions/workflows/ci.yml)
[![License](https://img.shields.io/github/license/panda850819/murmur-voice?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows-blue?style=flat-square)]()

**[English](README.md)** | **[繁體中文](README.zh-TW.md)**

> 你的聲音，只有你自己聽見。

隱私優先的語音轉文字工具，支援 macOS 與 Windows，以 Rust 打造。

<p align="center">
  <img src="assets/screenshot-settings.png" width="360" alt="設定畫面" />
  <img src="assets/screenshot-recording.png" width="360" alt="錄音畫面" />
</p>

## Murmur 是什麼？

Murmur 是一款語音聽寫工具，能將你的語音轉錄並將整理好的文字插入游標所在位置 -- 適用於任何應用程式。支援本機（裝置端）與雲端轉錄，可選擇性啟用 LLM 後處理來移除贅詞、修正標點、將簡體中文轉換為繁體中文。

## 功能特色

- **按住說話** -- 按住修飾鍵說話，放開即插入文字
- **切換模式** -- 按一次開始錄音，再按一次停止（5 分鐘自動停止）
- **自訂快捷鍵** -- 可選擇任何修飾鍵（Option、Command、Shift、Control，左右皆可）
- **雙引擎** -- 本機 Whisper（Metal GPU）或 Groq 雲端 API
- **LLM 後處理** -- 透過 Groq LLM 移除贅詞、加入標點、簡繁轉換
- **應用感知風格** -- 根據前景應用程式自動調整輸出語氣（如 Slack 用口語、VS Code 用技術風格）
- **個人詞典** -- 加入常用詞彙提升轉錄準確度
- **轉錄預覽** -- 浮動預覽視窗顯示完整轉錄結果、即時更新、字數統計與偵測到的應用程式名稱
- **即時預覽** -- 說話時即時顯示部分轉錄結果（僅限本機引擎）
- **15 種語言** -- 自動偵測或手動選擇 15 種支援語言
- **跨平台** -- macOS 與 Windows 支援，使用平台原生快捷鍵與應用偵測
- **全系統** -- 適用於所有應用程式的任何文字輸入欄位
- **輕量** -- 基於 Tauri，約 30-50MB，對比 Electron 應用 200MB+
- **開源** -- 完全可審計，無遙測，無追蹤

## 下載

從 [Releases 頁面](https://github.com/panda850819/murmur-voice/releases) 下載最新版本。

| 平台 | 檔案 | 備註 |
|------|------|------|
| macOS (Apple Silicon) | `.dmg` | 需要[移除隔離屬性](#macos-murmur-voice-已損毀無法打開) |
| Windows | `.exe` / `.msi` | 純 CPU 版本，適用於所有硬體 |
| Windows (NVIDIA GPU) | `-cuda.exe` / `-cuda.msi` | 透過 CUDA 進行 GPU 加速 |

## 運作原理

```
快捷鍵 -> 錄音 (cpal) -> 轉錄 (Whisper) -> LLM 整理 (選用) -> 貼上至游標
```

**每次錄音最多觸發 2 次 API 呼叫**（使用 Groq 時）：一次 Whisper 轉錄，一次 LLM 後處理。

## 設定指南

### 1. 安裝與執行

```bash
git clone https://github.com/panda850819/murmur-voice.git
cd murmur-voice
pnpm install
pnpm tauri dev
```

### 2. 首次啟動

首次啟動時，Murmur 會引導你完成：
1. 授予**麥克風**與**輔助使用**權限
2. 下載 Whisper 模型（約 800MB，僅需一次）
3. 設定按住說話鍵

### 3. 轉錄引擎

| 引擎 | 速度 | 品質 | 隱私 | 設定 |
|------|------|------|------|------|
| **本機 (Whisper)** | 約 1-3 秒 | 良好 | 音訊不離開裝置 | 下載模型（約 800MB） |
| **Groq API** | <1 秒 | 良好 | 音訊傳送至 Groq 伺服器 | 從 [console.groq.com](https://console.groq.com) 取得免費 API key |

切換引擎：**設定 > 轉錄 > 引擎**

### 4. LLM 後處理（建議啟用）

需要 **Groq API key**（Whisper 與 LLM 共用同一組 key）。

功能包括：
- 移除贅詞（嗯、啊、那個、就是、um、uh...）
- 移除重複與自我修正
- 加入正確標點（中文全形、英文半形）
- 簡體中文轉換為繁體中文（台灣用語）
- 中英文之間加入空格
- 適當時格式化清單與段落

啟用方式：**設定 > AI 處理 > LLM 後處理**

### 5. 個人詞典

加入常用詞彙（姓名、專有名詞、縮寫）以提升轉錄準確度。這些詞彙會注入 Whisper 的初始提示。

設定方式：**設定 > 轉錄 > 詞典**（輸入詞彙，按 Enter 加入）

### 6. 應用感知風格

啟用後，Murmur 會偵測前景應用程式並調整 LLM 輸出語氣：

| 應用程式 | 風格 |
|---------|------|
| Slack、Discord、LINE、Telegram | 口語 |
| VS Code、Terminal、Cursor | 技術 |
| Pages、Word、Google Docs | 正式 |
| 其他 | 預設（自然） |

啟用方式：**設定 > AI 處理 > 應用感知風格**

## 推薦設定

中文聽寫的最佳體驗設定：

| 設定 | 值 | 原因 |
|------|-----|------|
| 引擎 | **Groq** | 最快的轉錄速度（<1 秒） |
| 語言 | **中文（普通話）** | 比自動偵測更準確 |
| LLM 後處理 | **開啟** | 移除贅詞 + 繁體中文轉換 |
| LLM 模型 | **Llama 3.3 70B** | 中文文字處理品質最佳 |
| 應用感知風格 | **開啟** | 根據情境調整語氣 |

## 技術架構

| 元件 | 技術 | 用途 |
|------|------|------|
| 應用框架 | Tauri 2 | 輕量桌面應用 |
| 音訊擷取 | cpal | 麥克風輸入 -> 16kHz 單聲道 |
| 語音轉文字 | whisper-rs / Groq API | 本機或雲端轉錄 |
| LLM 處理 | Groq API (Llama 3.3) | 文字整理與格式化 |
| 快捷鍵偵測 | CGEventTap / SetWindowsHookEx | 全域修飾鍵監聽（依平台） |
| 文字插入 | arboard + rdev | 剪貼簿寫入 + Cmd+V / Ctrl+V 模擬 |
| 應用偵測 | NSWorkspace / Win32 API | 前景應用偵測（依平台） |

## 系統需求

### macOS
- macOS 12.0+（建議使用 Apple Silicon 以使用本機 Whisper）
- 麥克風權限
- 輔助使用權限（用於全域快捷鍵 + 文字插入）

### Windows
- Windows 10+
- 麥克風權限

### 兩個平台皆需
- Groq API key（免費，用於雲端引擎與 LLM 功能）

## 常見問題

### macOS: 「Murmur Voice 已損毀，無法打開」

這是因為應用程式未經 Apple 開發者憑證簽名。macOS Gatekeeper 預設會隔離未簽名的應用程式。解決方法：

1. 將 Murmur Voice 移至 `/Applications`
2. 打開終端機執行：
   ```bash
   xattr -d com.apple.quarantine /Applications/Murmur\ Voice.app
   ```
3. 正常打開應用程式即可

### Windows: 該下載哪個版本？

| 你的顯卡 | 下載版本 | 原因 |
|----------|---------|------|
| NVIDIA（已安裝 CUDA 驅動） | `-cuda` 版本 | GPU 加速轉錄，速度快很多 |
| AMD / Intel / 內顯 | 標準版本 | CPU 轉錄，適用於所有硬體 |
| 不確定 | 標準版本 | 一定能執行，只是本機引擎較慢 |

### 為什麼應用程式沒有簽名？

Murmur 是免費開源專案。Apple Developer Program 每年費用為 $99 美元。未來可能會加入程式碼簽章，但目前在 macOS 上需要使用上述的解決方法。

## 隱私

Murmur 誕生於一次對商業語音轉文字應用的安全審計，該應用被發現會：
- 擷取瀏覽器 URL 與視窗標題
- 透過 CGEventTap 監控所有鍵盤輸入
- 將應用程式上下文傳送至遠端伺服器
- 包含工作階段錄影分析（Microsoft Clarity）

Murmur 不做任何上述行為。使用**本機引擎**時，你的音訊不會離開你的電腦。使用 **Groq** 時，音訊僅傳送至 Groq 的 API 進行轉錄 -- 不會收集或傳送任何其他資料。

## 授權

MIT
