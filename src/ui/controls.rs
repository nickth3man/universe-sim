use crate::camera::CameraController;
use crate::physics::system::AppState;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

pub fn ui_controls_system(
    mut contexts: EguiContexts,
    mut state: ResMut<AppState>,
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

            ui.add(
                egui::Slider::new(&mut state.simulation_speed, 0.0..=1000.0)
                    .text("Speed")
                    .logarithmic(true),
            );

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

            ui.add(
                egui::Slider::new(&mut camera.distance, 1.0..=100.0)
                    .text("Zoom (AU)")
                    .logarithmic(true),
            );

            let body_names: Vec<&str> = state.bodies.iter().map(|b| b.name.as_str()).collect();

            if !body_names.is_empty() {
                let current_name = body_names
                    .get(camera.focus_index)
                    .copied()
                    .unwrap_or("Unknown");

                egui::ComboBox::from_label("Focus On")
                    .selected_text(current_name)
                    .show_ui(ui, |ui| {
                        for (i, name) in body_names.iter().enumerate() {
                            ui.selectable_value(&mut camera.focus_index, i, *name);
                        }
                    });
            }

            ui.separator();
            ui.label(format!("Elapsed: {:.1} days", state.elapsed_days));
        });
}
