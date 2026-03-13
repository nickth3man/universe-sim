//! Error handling utilities for the solar system simulation.
//!
//! This module provides validation helpers and logging for non-fatal failures.
//! Bevy systems cannot return `Result`, so we use defensive checks and `warn!`/`error!`
//! to surface issues while allowing the simulation to continue where possible.

use bevy::log::warn;
use bevy::prelude::Resource;

/// Validates that a value is finite; returns it or a fallback.
#[inline]
pub fn validate_finite_or(value: f64, fallback: f64, name: &str) -> f64 {
    if value.is_finite() {
        value
    } else {
        warn!("{name} is not finite (got {value}), using {fallback}");
        fallback
    }
}

/// Validates that a value is finite and within [min, max]; returns clamped value or fallback.
#[inline]
pub fn validate_in_range_or(value: f64, min: f64, max: f64, fallback: f64, name: &str) -> f64 {
    if value.is_finite() {
        value.clamp(min, max)
    } else {
        warn!("{name} is not finite (got {value}), using {fallback}");
        fallback
    }
}

/// Validates that a value is finite and positive (> 0); returns it or a fallback.
#[inline]
pub fn validate_positive_or(value: f64, fallback: f64, name: &str) -> f64 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        warn!("{name} is invalid (got {value}), using {fallback}");
        fallback
    }
}

/// Optional user-facing error message for display in the UI.
/// Systems can set this when they detect recoverable but noteworthy issues.
#[derive(Resource, Default)]
pub struct LastError {
    /// Message to show (empty = no error).
    pub message: String,
    /// Frame count when set; used to auto-clear after a few seconds.
    pub frame: u32,
}

impl LastError {
    /// Set an error message. Call from systems when detecting recoverable issues.
    pub fn set(&mut self, message: impl Into<String>, frame: u32) {
        self.message = message.into();
        self.frame = frame;
    }

    /// Clear the error. Call when user dismisses or after timeout.
    pub fn clear(&mut self) {
        self.message.clear();
    }

    /// Returns true if there is an error to display.
    pub fn has_error(&self) -> bool {
        !self.message.is_empty()
    }
}
