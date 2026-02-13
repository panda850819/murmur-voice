use std::sync::Mutex;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub(crate) enum RecordingState {
    Idle,
    Starting,
    Recording,
    Stopping,
    Transcribing,
    Processing,
}

impl std::fmt::Display for RecordingState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecordingState::Idle => write!(f, "idle"),
            RecordingState::Starting => write!(f, "starting"),
            RecordingState::Recording => write!(f, "recording"),
            RecordingState::Stopping => write!(f, "stopping"),
            RecordingState::Transcribing => write!(f, "transcribing"),
            RecordingState::Processing => write!(f, "processing"),
        }
    }
}

#[derive(Debug, Error)]
pub(crate) enum StateError {
    #[error("invalid transition from {from} to {to}")]
    InvalidTransition {
        from: RecordingState,
        to: RecordingState,
    },
}

impl serde::Serialize for StateError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub(crate) struct AppState {
    state: Mutex<RecordingState>,
}

impl AppState {
    pub(crate) fn new() -> Self {
        Self {
            state: Mutex::new(RecordingState::Idle),
        }
    }

    pub(crate) fn current(&self) -> RecordingState {
        *self.state.lock().expect("state mutex poisoned")
    }

    pub(crate) fn transition(&self, to: RecordingState) -> Result<RecordingState, StateError> {
        let mut current = self.state.lock().expect("state mutex poisoned");
        if Self::is_valid_transition(*current, to) {
            *current = to;
            Ok(to)
        } else {
            Err(StateError::InvalidTransition { from: *current, to })
        }
    }

    fn is_valid_transition(from: RecordingState, to: RecordingState) -> bool {
        // ANY state -> Idle (error recovery)
        if to == RecordingState::Idle {
            return true;
        }

        matches!(
            (from, to),
            (RecordingState::Idle, RecordingState::Starting)
                | (RecordingState::Starting, RecordingState::Recording)
                | (RecordingState::Recording, RecordingState::Stopping)
                | (RecordingState::Stopping, RecordingState::Transcribing)
                | (RecordingState::Transcribing, RecordingState::Processing)
                | (RecordingState::Transcribing, RecordingState::Idle)
                | (RecordingState::Processing, RecordingState::Idle)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state_is_idle() {
        let state = AppState::new();
        assert_eq!(state.current(), RecordingState::Idle);
    }

    #[test]
    fn test_valid_forward_transitions() {
        let state = AppState::new();

        assert!(state.transition(RecordingState::Starting).is_ok());
        assert_eq!(state.current(), RecordingState::Starting);

        assert!(state.transition(RecordingState::Recording).is_ok());
        assert_eq!(state.current(), RecordingState::Recording);

        assert!(state.transition(RecordingState::Stopping).is_ok());
        assert_eq!(state.current(), RecordingState::Stopping);

        assert!(state.transition(RecordingState::Transcribing).is_ok());
        assert_eq!(state.current(), RecordingState::Transcribing);

        assert!(state.transition(RecordingState::Processing).is_ok());
        assert_eq!(state.current(), RecordingState::Processing);

        assert!(state.transition(RecordingState::Idle).is_ok());
        assert_eq!(state.current(), RecordingState::Idle);
    }

    #[test]
    fn test_invalid_transitions() {
        let state = AppState::new();

        // Idle -> Recording (skipping Starting)
        assert!(state.transition(RecordingState::Recording).is_err());

        // Idle -> Stopping
        assert!(state.transition(RecordingState::Stopping).is_err());

        // Idle -> Transcribing
        assert!(state.transition(RecordingState::Transcribing).is_err());

        // Idle -> Processing
        assert!(state.transition(RecordingState::Processing).is_err());
    }

    #[test]
    fn test_error_recovery_to_idle_from_any_state() {
        let state = AppState::new();

        // Starting -> Idle
        state.transition(RecordingState::Starting).unwrap();
        assert!(state.transition(RecordingState::Idle).is_ok());

        // Recording -> Idle
        state.transition(RecordingState::Starting).unwrap();
        state.transition(RecordingState::Recording).unwrap();
        assert!(state.transition(RecordingState::Idle).is_ok());

        // Stopping -> Idle
        state.transition(RecordingState::Starting).unwrap();
        state.transition(RecordingState::Recording).unwrap();
        state.transition(RecordingState::Stopping).unwrap();
        assert!(state.transition(RecordingState::Idle).is_ok());

        // Transcribing -> Idle
        state.transition(RecordingState::Starting).unwrap();
        state.transition(RecordingState::Recording).unwrap();
        state.transition(RecordingState::Stopping).unwrap();
        state.transition(RecordingState::Transcribing).unwrap();
        assert!(state.transition(RecordingState::Idle).is_ok());

        // Processing -> Idle
        state.transition(RecordingState::Starting).unwrap();
        state.transition(RecordingState::Recording).unwrap();
        state.transition(RecordingState::Stopping).unwrap();
        state.transition(RecordingState::Transcribing).unwrap();
        state.transition(RecordingState::Processing).unwrap();
        assert!(state.transition(RecordingState::Idle).is_ok());
    }

    #[test]
    fn test_backward_transitions_invalid() {
        let state = AppState::new();
        state.transition(RecordingState::Starting).unwrap();
        state.transition(RecordingState::Recording).unwrap();

        // Recording -> Starting (backward)
        assert!(state.transition(RecordingState::Starting).is_err());
    }

    #[test]
    fn test_display() {
        assert_eq!(RecordingState::Idle.to_string(), "idle");
        assert_eq!(RecordingState::Starting.to_string(), "starting");
        assert_eq!(RecordingState::Recording.to_string(), "recording");
        assert_eq!(RecordingState::Stopping.to_string(), "stopping");
        assert_eq!(RecordingState::Transcribing.to_string(), "transcribing");
        assert_eq!(RecordingState::Processing.to_string(), "processing");
    }
}
