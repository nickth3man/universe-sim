//! Error handling utilities for the solar system simulation.
//!
//! This module provides validation helpers and logging for non-fatal failures.
//! Bevy systems cannot return `Result`, so we use defensive checks and `warn!`/`error!`
//! to surface issues while allowing the simulation to continue where possible.

#![allow(dead_code)]

use bevy::log::{error, warn};

/// Logs an error and returns `None` for optional chaining.
#[inline]
pub fn log_and_none<T, E: std::fmt::Display>(result: Result<T, E>, context: &str) -> Option<T> {
    match result {
        Ok(v) => Some(v),
        Err(e) => {
            error!("{context}: {e}");
            None
        }
    }
}

/// Logs a warning if the condition is false.
#[inline]
pub fn warn_if(condition: bool, message: &str) {
    if !condition {
        warn!("{message}");
    }
}

/// Validates that a value is finite and non-negative.
#[inline]
pub fn validate_non_negative_finite(value: f64, name: &str) -> bool {
    if !value.is_finite() {
        warn!("{name} is not finite (got {value}), using 0");
        return false;
    }
    if value < 0.0 {
        warn!("{name} is negative (got {value}), using 0");
        return false;
    }
    true
}

/// Validates that a value is finite and in the range [min, max].
#[inline]
pub fn validate_range(value: f64, min: f64, max: f64, name: &str) -> f64 {
    if !value.is_finite() {
        warn!("{name} is not finite (got {value}), clamping to {min}");
        return min;
    }
    value.clamp(min, max)
}
