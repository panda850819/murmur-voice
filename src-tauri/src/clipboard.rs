use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum ClipboardError {
    #[error("clipboard access failed: {0}")]
    Access(String),
    #[error("key simulation failed: {0}")]
    Simulate(String),
}

impl serde::Serialize for ClipboardError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub(crate) fn insert_text(text: &str) -> Result<(), ClipboardError> {
    if text.is_empty() {
        return Ok(());
    }

    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| ClipboardError::Access(e.to_string()))?;

    // Save current clipboard content
    let original = clipboard.get_text().ok();

    // Set clipboard to our text
    clipboard
        .set_text(text)
        .map_err(|e| ClipboardError::Access(e.to_string()))?;

    // Wait for clipboard to settle
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Simulate Cmd+V
    let paste_result = simulate_paste();

    // Wait for paste to complete
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Restore original clipboard
    if let Some(original_text) = original {
        let _ = clipboard.set_text(original_text);
    }

    paste_result
}

/// Copies text to the system clipboard without simulating paste or restoring previous content.
pub(crate) fn copy_only(text: &str) -> Result<(), ClipboardError> {
    if text.is_empty() {
        return Ok(());
    }
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| ClipboardError::Access(e.to_string()))?;
    clipboard
        .set_text(text)
        .map_err(|e| ClipboardError::Access(e.to_string()))?;
    Ok(())
}

/// Simulates Cmd+C (macOS) / Ctrl+C (Windows) to copy the current text selection.
/// Releases all modifier keys first to prevent combos like Cmd+Option+C.
pub(crate) fn copy_selection() -> Result<(), ClipboardError> {
    release_all_modifiers();
    std::thread::sleep(std::time::Duration::from_millis(50));
    simulate_copy()?;
    std::thread::sleep(std::time::Duration::from_millis(150));
    Ok(())
}

/// Reads current clipboard text content.
pub(crate) fn read_text() -> Result<String, ClipboardError> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| ClipboardError::Access(e.to_string()))?;
    clipboard
        .get_text()
        .map_err(|e| ClipboardError::Access(e.to_string()))
}

/// Sets clipboard text and pastes via Cmd+V / Ctrl+V.
/// Unlike insert_text(), does NOT restore original clipboard content —
/// the translated text stays on the clipboard for subsequent pastes.
pub(crate) fn set_and_paste(text: &str) -> Result<(), ClipboardError> {
    if text.is_empty() {
        return Ok(());
    }

    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| ClipboardError::Access(e.to_string()))?;
    clipboard
        .set_text(text)
        .map_err(|e| ClipboardError::Access(e.to_string()))?;

    std::thread::sleep(std::time::Duration::from_millis(50));

    simulate_paste()?;

    std::thread::sleep(std::time::Duration::from_millis(100));

    Ok(())
}

#[cfg(target_os = "macos")]
fn simulate_paste() -> Result<(), ClipboardError> {
    use rdev::{simulate, EventType, Key};

    let events = [
        EventType::KeyPress(Key::MetaLeft),
        EventType::KeyPress(Key::KeyV),
        EventType::KeyRelease(Key::KeyV),
        EventType::KeyRelease(Key::MetaLeft),
    ];

    for event in &events {
        simulate(event).map_err(|e| ClipboardError::Simulate(format!("{:?}", e)))?;
        std::thread::sleep(std::time::Duration::from_millis(20));
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn simulate_paste() -> Result<(), ClipboardError> {
    use rdev::{simulate, EventType, Key};

    let events = [
        EventType::KeyPress(Key::ControlLeft),
        EventType::KeyPress(Key::KeyV),
        EventType::KeyRelease(Key::KeyV),
        EventType::KeyRelease(Key::ControlLeft),
    ];

    for event in &events {
        simulate(event).map_err(|e| ClipboardError::Simulate(format!("{:?}", e)))?;
        std::thread::sleep(std::time::Duration::from_millis(20));
    }

    Ok(())
}

/// Release all modifier keys to ensure a clean state before simulating key combos.
/// This prevents the physically-held translate hotkey modifier (e.g. Option)
/// from interfering with the simulated Cmd+C.
fn release_all_modifiers() {
    use rdev::{simulate, EventType, Key};

    let modifiers = [
        Key::Alt,
        Key::MetaLeft,
        Key::MetaRight,
        Key::ShiftLeft,
        Key::ShiftRight,
        Key::ControlLeft,
        Key::ControlRight,
    ];

    for key in &modifiers {
        let _ = simulate(&EventType::KeyRelease(*key));
    }
}

#[cfg(target_os = "macos")]
fn simulate_copy() -> Result<(), ClipboardError> {
    use rdev::{simulate, EventType, Key};

    let events = [
        EventType::KeyPress(Key::MetaLeft),
        EventType::KeyPress(Key::KeyC),
        EventType::KeyRelease(Key::KeyC),
        EventType::KeyRelease(Key::MetaLeft),
    ];

    for event in &events {
        simulate(event).map_err(|e| ClipboardError::Simulate(format!("{:?}", e)))?;
        std::thread::sleep(std::time::Duration::from_millis(20));
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn simulate_copy() -> Result<(), ClipboardError> {
    use rdev::{simulate, EventType, Key};

    let events = [
        EventType::KeyPress(Key::ControlLeft),
        EventType::KeyPress(Key::KeyC),
        EventType::KeyRelease(Key::KeyC),
        EventType::KeyRelease(Key::ControlLeft),
    ];

    for event in &events {
        simulate(event).map_err(|e| ClipboardError::Simulate(format!("{:?}", e)))?;
        std::thread::sleep(std::time::Duration::from_millis(20));
    }

    Ok(())
}
