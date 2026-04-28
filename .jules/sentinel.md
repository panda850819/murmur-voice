## 2024-05-24 - [Fix Time-of-Check to Time-of-Use (TOCTOU) during settings save]
**Vulnerability:** A TOCTOU vulnerability was found in `save_settings` where `std::fs::write` was used to create the settings file containing sensitive API keys with default permissions, followed by a separate call to `std::fs::set_permissions` to restrict them.
**Learning:** This leaves a race condition window where an attacker on the same machine could read the file before its permissions are secured.
**Prevention:** Always use `std::fs::OpenOptions` with the `.mode(0o600)` builder method (via `std::os::unix::fs::OpenOptionsExt` on Unix) to create sensitive files atomically with the correct permissions from the start.
