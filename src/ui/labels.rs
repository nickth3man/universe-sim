//! Screen-space body labels: project 3D positions to 2D and draw names.

use crate::physics::system::PhysicsState;
use crate::render::BodyMesh;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

/// Resource to toggle body labels on/off.
#[derive(Resource, Clone, Default)]
pub struct ShowBodyLabels(pub bool);

/// Draws body names at their screen positions (above each planet).
pub fn body_labels_system(
    mut contexts: EguiContexts,
    state: Res<PhysicsState>,
    show_labels: Res<ShowBodyLabels>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    body_query: Query<(Entity, &Transform), With<BodyMesh>>,
) {
    if !show_labels.0 {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let Some((camera, camera_transform)) = camera_query.iter().next() else {
        return;
    };

    let mut body_labels: Vec<(Vec2, &str)> = Vec::new();

    for (entity, transform) in body_query.iter() {
        let Some(body) = state.bodies.get(&entity) else {
            continue;
        };

        // Offset above the sphere by its visual radius (scale) plus a small gap
        let radius = transform.scale.x.max(0.05); // scale is uniform (splat)
        let world_pos = transform.translation + Vec3::Y * (radius + 0.05);
        let Ok(viewport_pos) = camera.world_to_viewport(camera_transform, world_pos) else {
            continue; // Behind camera or outside frustum
        };

        body_labels.push((viewport_pos, body.name.as_str()));
    }

    for (pos, name) in body_labels {
        egui::Area::new(egui::Id::new("body_label").with(name))
            .fixed_pos(egui::pos2(pos.x, pos.y))
            .order(egui::Order::Foreground)
            .anchor(egui::Align2::CENTER_BOTTOM, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(egui::RichText::new(name).color(egui::Color32::WHITE));
            });
    }
}
