//! Keyboard shortcuts for the solar system simulator.
//!
//! Handles: Space (pause), 1-6 (speed presets), Tab/Right (next body),
//! Shift+Tab/Left (prev body), R (reset view).

use crate::camera::CameraController;
use crate::physics::system::PhysicsState;
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

    // 1-6: Jump to speed presets
    for (i, key) in [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
    ]
    .iter()
    .enumerate()
    {
        if keyboard.just_pressed(*key) {
            if let Some(&(speed, _)) = TIME_PRESETS.get(i) {
                state.simulation_speed = speed;
            }
            break;
        }
    }

    // Numpad 1-6: Same as digit keys
    for (i, key) in [
        KeyCode::Numpad1,
        KeyCode::Numpad2,
        KeyCode::Numpad3,
        KeyCode::Numpad4,
        KeyCode::Numpad5,
        KeyCode::Numpad6,
    ]
    .iter()
    .enumerate()
    {
        if keyboard.just_pressed(*key) {
            if let Some(&(speed, _)) = TIME_PRESETS.get(i) {
                state.simulation_speed = speed;
            }
            break;
        }
    }

    // Tab / Right: Next body; Shift+Tab / Left: Previous body
    let mut bodies: Vec<_> = state.bodies.values().collect();
    bodies.sort_by(|a, b| a.name.cmp(&b.name));
    if !bodies.is_empty() {
        let has_valid_focus = bodies.iter().any(|b| b.entity == camera.focus);
        if !has_valid_focus {
            camera.focus = bodies.first().map(|b| b.entity).unwrap_or(camera.focus);
        }

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
        let focus_entity_on_reset = bodies
            .iter()
            .find(|b| b.name == "Sun")
            .or_else(|| bodies.first())
            .map(|b| b.entity);
        if let Some(entity) = focus_entity_on_reset {
            camera.focus = entity;
        }
    }
}
