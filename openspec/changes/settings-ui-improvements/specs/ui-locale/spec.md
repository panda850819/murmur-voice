## ADDED Requirements

### Requirement: UI language setting
The system SHALL provide a UI Language selector in the System section of settings that allows switching between English and zh-TW (Traditional Chinese).

#### Scenario: Default language
- **WHEN** a user opens settings for the first time (no `ui_locale` in saved settings)
- **THEN** the UI displays in English (default)

#### Scenario: Switch to zh-TW
- **WHEN** the user selects "繁體中文" from the UI Language dropdown
- **THEN** all static labels in the settings window update to Traditional Chinese immediately

#### Scenario: Switch back to English
- **WHEN** the user selects "English" from the UI Language dropdown
- **THEN** all static labels revert to English immediately

#### Scenario: Locale persists across sessions
- **WHEN** the user saves settings with `ui_locale` set to "zh-TW" and reopens settings
- **THEN** the settings window loads in zh-TW

### Requirement: DOM-based localization
The system SHALL use `data-i18n` attributes on static HTML elements to apply translations via a `applyLocale()` function.

#### Scenario: Translation applied to labeled elements
- **WHEN** `applyLocale("zh-TW")` is called
- **THEN** every element with a `data-i18n` attribute has its `textContent` set to the zh-TW translation for that key

#### Scenario: Missing translation key
- **WHEN** an element has a `data-i18n` key that does not exist in the translation map
- **THEN** the element's text remains unchanged (no crash, no empty string)

#### Scenario: Dynamic elements excluded
- **WHEN** the locale is applied
- **THEN** PTT key display names, opacity percentage values, dictionary tags, and `<option>` values for provider/engine names remain unchanged (they are not translated)

### Requirement: Backward-compatible settings field
The system SHALL add a `ui_locale` field to the Settings struct with `#[serde(default)]` defaulting to `"en"`, so existing settings.json files without this field deserialize correctly.

#### Scenario: Legacy settings file
- **WHEN** a settings.json file from v0.3.0 (without `ui_locale`) is loaded
- **THEN** `ui_locale` defaults to `"en"` and all other fields load correctly
