use crate::camera::CameraController;
use crate::error::{validate_finite_or, validate_in_range_or};
use crate::physics::system::PhysicsState;
use crate::render::trail::{TrailConfig, DEFAULT_TRAIL_LENGTH_DAYS};
use crate::ui::labels::ShowBodyLabels;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

/// Time preset: (label, seconds per sim day). Used by keyboard shortcuts (1-6).
pub(crate) const TIME_PRESETS: &[(f64, &str)] = &[
    (1.0, "1× Realtime"),
    (86_400.0, "1 day/sec"),
    (86_400.0 * 7.0, "1 week/sec"),
    (86_400.0 * 30.0, "1 month/sec"),
    (86_400.0 * 365.25, "1 year/sec"),
    (86_400.0 * 365.25 * 10.0, "10 years/sec"),
];

/// Bevy system: draws the egui "Solar System Controls" overlay panel.
///
/// Mutates PhysicsState, CameraController, TrailConfig, and ShowBodyLabels.
pub fn ui_controls_system(
    mut contexts: EguiContexts,
    mut state: ResMut<PhysicsState>,
    mut camera: ResMut<CameraController>,
    mut trail_config: ResMut<TrailConfig>,
    mut show_labels: ResMut<ShowBodyLabels>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    // Sanitize state after egui bindings (guards against corrupted or NaN values)
    state.simulation_speed = validate_finite_or(state.simulation_speed, 1.0, "simulation_speed")
        .max(0.0);
    trail_config.length_days = validate_in_range_or(
        trail_config.length_days,
        1.0,
        365.0,
        DEFAULT_TRAIL_LENGTH_DAYS,
        "trail_config.length_days",
    );
    camera.distance = validate_finite_or(camera.distance, 10.0, "camera.distance").clamp(1.0, 100.0);

    egui::Window::new("Solar System Controls")
        .default_pos([10.0, 10.0])
        .show(ctx, |ui| {
            ui.heading("Simulation");

            // Logarithmic slider: 0 = pause, 1 = realtime, max = 1 sim year per real second.
            const MAX_SPEED: f64 = 365.25 * 86_400.0;
            ui.add(
                egui::Slider::new(&mut state.simulation_speed, 0.0..=MAX_SPEED)
                    .text("Speed (realtime → 1 yr/sec)")
                    .logarithmic(true),
            );

            // Time presets
            ui.horizontal_wrapped(|ui| {
                for (speed, label) in TIME_PRESETS {
                    if ui.button(*label).clicked() {
                        state.simulation_speed = *speed;
                    }
                }
            });

            if ui
                .button(if state.simulation_speed > 0.0 {
                    "⏸ Pause"
                } else {
                    "▶ Resume"
                })
                .clicked()
            {
                state.simulation_speed = if state.simulation_speed > 0.0 {
                    0.0
                } else {
                    1.0
                };
            }

            ui.separator();
            ui.heading("Orbit Trails");

            ui.checkbox(&mut trail_config.enabled, "Show trails");

            if trail_config.enabled {
                ui.add(
                    egui::Slider::new(&mut trail_config.length_days, 1.0..=365.0)
                        .text("Trail length (days)")
                        .logarithmic(true),
                );
            }

            ui.separator();
            ui.heading("Camera");

            ui.checkbox(&mut show_labels.0, "Body labels");

            ui.add(
                egui::Slider::new(&mut camera.distance, 1.0..=100.0)
                    .text("Zoom (AU)")
                    .logarithmic(true),
            );

            let bodies: Vec<_> = state.bodies.values().collect();

            if !bodies.is_empty() {
                let has_valid_focus = bodies.iter().any(|b| b.entity == camera.focus);
                if !has_valid_focus {
                    camera.focus = bodies.first().map(|b| b.entity).unwrap_or(camera.focus);
                }

                let focused_body_name = bodies
                    .iter()
                    .find(|b| b.entity == camera.focus)
                    .map(|b| b.name.as_str())
                    .unwrap_or("Unknown");

                egui::ComboBox::from_label("Focus On")
                    .selected_text(focused_body_name)
                    .show_ui(ui, |ui| {
                        for body in &bodies {
                            ui.selectable_value(&mut camera.focus, body.entity, &body.name);
                        }
                    });

                if ui.button("Reset view").clicked() {
                    camera.distance = 10.0;
                    camera.focus = bodies.first().map(|b| b.entity).unwrap_or(camera.focus);
                }
            }

            ui.separator();
            ui.label(format!("Elapsed: {:.1} days", state.elapsed_days));
        });
}
