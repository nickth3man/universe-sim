use crate::camera::CameraController;
use crate::physics::system::PhysicsState;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

/// Bevy system: draws the egui "Solar System Controls" overlay panel.
///
/// This is the only user-facing input mechanism in the application. It mutates
/// `PhysicsState.simulation_speed` and `CameraController.distance`/`focus`.
pub fn ui_controls_system(
    mut contexts: EguiContexts,
    mut state: ResMut<PhysicsState>,
    mut camera: ResMut<CameraController>,
) {
    let ctx = match contexts.ctx_mut() {
        Ok(ctx) => ctx,
        Err(_) => return,
    };

    egui::Window::new("Solar System Controls")
        .default_pos([10.0, 10.0])
        .show(ctx, |ui| {
            ui.heading("Simulation");

            // Logarithmic slider gives fine control at low speeds and coarse at high.
            // Range 0.0–1000.0.
            ui.add(
                egui::Slider::new(&mut state.simulation_speed, 0.0..=1000.0)
                    .text("Speed")
                    .logarithmic(true),
            );

            // NOTE: This pause button is currently non-functional.
            // It sets simulation_speed = 0.0, but orbital_physics_system clamps
            // speed to MIN_SIMULATION_SPEED (1.0) every frame, so the simulation
            // immediately resumes. Fix: lower MIN_SIMULATION_SPEED to 0.0.
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
            ui.heading("Camera");

            // Logarithmic zoom: 1 AU (close to Sun) → 100 AU (beyond Neptune at 30 AU).
            ui.add(
                egui::Slider::new(&mut camera.distance, 1.0..=100.0)
                    .text("Zoom (AU)")
                    .logarithmic(true),
            );

            // Build entity list from PhysicsState so the ComboBox always
            // reflects whatever bodies are in the system (even if bodies are added at runtime).
            let bodies: Vec<_> = state.bodies.values().collect();

            if !bodies.is_empty() {
                let current_name = bodies
                    .iter()
                    .find(|b| b.entity == camera.focus)
                    .map(|b| b.name.as_str())
                    .unwrap_or("Unknown");

                egui::ComboBox::from_label("Focus On")
                    .selected_text(current_name)
                    .show_ui(ui, |ui| {
                        for body in &bodies {
                            ui.selectable_value(&mut camera.focus, body.entity, &body.name);
                        }
                    });
            }

            ui.separator();
            // Read-only elapsed time display.
            ui.label(format!("Elapsed: {:.1} days", state.elapsed_days));
        });
}
