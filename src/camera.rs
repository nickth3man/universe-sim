use crate::physics::system::AppState;
use bevy::prelude::*;

/// Spherical-coordinate orbit camera that follows a selected body.
///
/// The camera hovers at `distance` world units from the focused body,
/// offset by `pitch` and `yaw` angles. Only `distance` and `focus`
/// are exposed via the UI; `pitch` and `yaw` have no input handling and
/// remain at their defaults for the entire session.
#[derive(Resource)]
pub struct CameraController {
    /// Distance from the focus body in camera-space units (1 unit = 10 AU after scaling).
    /// Controlled by the Zoom slider in the UI.
    pub distance: f64,

    /// Entity ID of the body the camera orbits around.
    /// Controlled by the "Focus On" ComboBox in the UI.
    pub focus: Entity,

    /// Elevation angle from the XZ plane in radians. Fixed at π/6 (30°).
    /// No input system writes to this field, so it never changes at runtime.
    pub pitch: f64,

    /// Azimuth angle around the Y axis in radians. Fixed at 0.
    /// No input system writes to this field, so it never changes at runtime.
    pub yaw: f64,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            distance: 10.0,
            focus: Entity::PLACEHOLDER,  // Will set to Sun entity at runtime
            pitch: std::f64::consts::FRAC_PI_6,    // 30° elevation
            yaw: 0.0,
        }
    }
}

/// Bevy system: positions the camera in a spherical orbit around the focused body.
///
/// World-space conversion: body positions from AppState are in AU;
/// multiply by 10.0 to get world units (same factor used in update_body_transforms).
pub fn camera_follow_system(
    state: Res<AppState>,
    camera_controller: Res<CameraController>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    if let Ok(mut camera_transform) = camera_query.single_mut() {
        if let Some(focus_body) = state.bodies.get(camera_controller.focus_index) {
            // Convert focus body position from AU to world units.
            let focus_pos = Vec3::new(
                focus_body.position.x as f32 * 10.0,
                focus_body.position.y as f32 * 10.0,
                focus_body.position.z as f32 * 10.0,
            );

            // Scale camera distance to world units; match the 10× AU conversion.
            let distance = camera_controller.distance as f32 * 10.0;
            let pitch = camera_controller.pitch as f32;
            let yaw = camera_controller.yaw as f32;

            // Spherical → Cartesian offset from the focus body:
            //   x = d·cos(pitch)·cos(yaw)
            //   y = d·sin(pitch)          ← vertical component (Y-up)
            //   z = d·cos(pitch)·sin(yaw)
            let x = distance * pitch.cos() * yaw.cos();
            let y = distance * pitch.sin();
            let z = distance * pitch.cos() * yaw.sin();

            camera_transform.translation = focus_pos + Vec3::new(x, y, z);
            camera_transform.look_at(focus_pos, Vec3::Y);
        }
    }
}
