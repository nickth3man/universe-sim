//! Keyboard shortcuts for the solar system simulator.
//!
//! Handles: Space (pause), 1-6 (speed presets), Tab/Right (next body),
//! Shift+Tab/Left (prev body), R (reset view).

use crate::camera::CameraController;
use crate::physics::system::PhysicsState;
use crate::ui::bodies::{ensure_valid_focus, focus_sun_or_first, sorted_bodies};
use crate::ui::controls::TIME_PRESETS;
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

/// Bevy system: handles keyboard shortcuts for common actions.
///
/// Runs in Update (before egui) so it can read key presses.
/// Uses `just_pressed` to avoid repeat triggers.
/// Uses `Option<Res<ButtonInput<KeyCode>>>` for headless tests (MinimalPlugins).
pub fn keyboard_shortcuts_system(
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    mut state: ResMut<PhysicsState>,
    mut camera: ResMut<CameraController>,
) {
    let Some(keyboard) = keyboard else {
        return; // Headless mode: no input
    };
    // Space: Toggle pause
    if keyboard.just_pressed(KeyCode::Space) {
        state.simulation_speed = if state.simulation_speed > 0.0 {
            0.0
        } else {
            1.0
        };
    }

    // 1-6 and Numpad 1-6: Jump to speed presets
    const SPEED_PRESET_KEYS: [[KeyCode; 2]; 6] = [
        [KeyCode::Digit1, KeyCode::Numpad1],
        [KeyCode::Digit2, KeyCode::Numpad2],
        [KeyCode::Digit3, KeyCode::Numpad3],
        [KeyCode::Digit4, KeyCode::Numpad4],
        [KeyCode::Digit5, KeyCode::Numpad5],
        [KeyCode::Digit6, KeyCode::Numpad6],
    ];
    for (i, keys) in SPEED_PRESET_KEYS.iter().enumerate() {
        if keys.iter().any(|k| keyboard.just_pressed(*k)) {
            if let Some(&(speed, _)) = TIME_PRESETS.get(i) {
                state.simulation_speed = speed;
            }
            break;
        }
    }

    // Tab / Right: Next body; Shift+Tab / Left: Previous body
    let bodies = sorted_bodies(&state);
    if !bodies.is_empty() {
        ensure_valid_focus(&bodies, &mut camera);

        let is_shift_held = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
        let should_select_next_body = keyboard.just_pressed(KeyCode::ArrowRight)
            || (keyboard.just_pressed(KeyCode::Tab) && !is_shift_held);
        let should_select_prev_body = keyboard.just_pressed(KeyCode::ArrowLeft)
            || (keyboard.just_pressed(KeyCode::Tab) && is_shift_held);

        if should_select_next_body || should_select_prev_body {
            let focused_body_index = bodies
                .iter()
                .position(|b| b.entity == camera.focus)
                .unwrap_or(0);

            let next_body_index = if should_select_prev_body {
                if focused_body_index == 0 {
                    bodies.len() - 1
                } else {
                    focused_body_index - 1
                }
            } else {
                (focused_body_index + 1) % bodies.len()
            };

            if let Some(body) = bodies.get(next_body_index) {
                camera.focus = body.entity;
            }
        }
    }

    // R: Reset view (same as Reset view button: zoom 10 AU, focus on Sun or first body)
    if keyboard.just_pressed(KeyCode::KeyR) {
        camera.distance = 10.0;
        if let Some(entity) = focus_sun_or_first(&bodies) {
            camera.focus = entity;
        }
    }
}
