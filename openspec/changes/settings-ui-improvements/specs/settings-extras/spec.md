## ADDED Requirements

### Requirement: Changelog button
The system SHALL display a "What's New" row in the System section of settings with a button that opens the GitHub releases page.

#### Scenario: Changelog button click
- **WHEN** the user clicks the Changelog button
- **THEN** the system opens `https://github.com/panda850819/murmur-voice/releases` in the default browser

### Requirement: Vision footer
The system SHALL display a vision tagline and roadmap link in the settings footer, above the Save/Cancel actions.

#### Scenario: Vision tagline displayed
- **WHEN** the settings window is open
- **THEN** the footer shows "Your voice, unheard by others." in muted italic text

#### Scenario: Roadmap link click
- **WHEN** the user clicks the Roadmap link in the footer
- **THEN** the system opens the GitHub repository URL in the default browser

### Requirement: Update badge layout fix
The system SHALL display the update-available badge with consistent width and centered text alignment.

#### Scenario: Badge alignment
- **WHEN** the "Check for Updates" button shows an available update (e.g., "v0.3.1 available")
- **THEN** the button has a minimum width of 120px and centered text, preventing layout shift
