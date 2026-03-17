//! Shared helpers for body list and camera focus logic.
//!
//! Used by both the controls panel and keyboard shortcuts to ensure consistent
//! behavior when selecting/resetting the focused body.

use crate::camera::CameraController;
use crate::physics::system::BodyState;
use crate::physics::system::PhysicsState;
use bevy::prelude::Entity;

/// Returns bodies sorted by name for consistent UI ordering and Tab/Arrow navigation.
pub fn sorted_bodies(state: &PhysicsState) -> Vec<&BodyState> {
    let mut bodies: Vec<_> = state.bodies.values().collect();
    bodies.sort_by(|a, b| a.name.cmp(&b.name));
    bodies
}

/// Ensures the camera focus points to a valid body. If the current focus is
/// invalid (e.g. entity despawned), resets to the first body in sorted order.
pub fn ensure_valid_focus(bodies: &[&BodyState], camera: &mut CameraController) {
    if bodies.is_empty() {
        return;
    }
    let has_valid = bodies.iter().any(|b| b.entity == camera.focus);
    if !has_valid {
        camera.focus = bodies.first().map(|b| b.entity).unwrap_or(camera.focus);
    }
}

/// Returns the Sun entity if present, otherwise the first body. Used for reset view.
pub fn focus_sun_or_first(bodies: &[&BodyState]) -> Option<Entity> {
    bodies
        .iter()
        .find(|b| b.name == "Sun")
        .or_else(|| bodies.first())
        .map(|b| b.entity)
}
