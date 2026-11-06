//! nova-core: Foundational types, identifiers, clock, and event primitives.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Unique identifier for platform entities (experiences, places, users, assets).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Id(Uuid);

impl Id {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_u128(v: u128) -> Self {
        Self(Uuid::from_u128(v))
    }

    pub fn as_u128(&self) -> u128 {
        self.0.as_u128()
    }
}

impl Default for Id {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for Id {
    type Err = uuid::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

/// Monotonic and wall-clock time tracking for the engine.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EngineTime {
    pub frame_count: u64,
    pub elapsed_seconds: f64,
    pub delta_seconds: f32,
    pub utc_timestamp: DateTime<Utc>,
}

impl EngineTime {
    pub fn new() -> Self {
        Self {
            frame_count: 0,
            elapsed_seconds: 0.0,
            delta_seconds: 0.0,
            utc_timestamp: Utc::now(),
        }
    }

    pub fn tick(&mut self, dt: f32) {
        self.frame_count += 1;
        self.delta_seconds = dt.max(0.0);
        self.elapsed_seconds += dt as f64;
        self.utc_timestamp = Utc::now();
    }
}

impl Default for EngineTime {
    fn default() -> Self {
        Self::new()
    }
}

/// Generic engine errors.
#[derive(thiserror::Error, Debug)]
pub enum EngineError {
    #[error("Entity not found: {0}")]
    NotFound(String),
    #[error("Validation failed: {0}")]
    Validation(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Physics error: {0}")]
    Physics(String),
    #[error("Script error: {0}")]
    Script(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Platform error: {0}")]
    Platform(String),
}

pub type Result<T> = std::result::Result<T, EngineError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_uniqueness() {
        let id1 = Id::new();
        let id2 = Id::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_engine_time() {
        let mut time = EngineTime::new();
        assert_eq!(time.frame_count, 0);
        time.tick(0.016);
        assert_eq!(time.frame_count, 1);
        assert!((time.delta_seconds - 0.016).abs() < 1e-5);
    }
}
