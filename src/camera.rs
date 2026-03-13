use crate::error::validate_finite_or;
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
            focus: Entity::PLACEHOLDER, // Will set to Sun entity at runtime
            pitch: std::f64::consts::FRAC_PI_6, // 30° elevation
            yaw: 0.0,
        }
    }
}

/// Bevy system: positions the camera in a spherical orbit around the focused body.
///
/// Now uses entity-based lookups instead of index-based AppState access.
pub fn camera_follow_system(
    camera_controller: Res<CameraController>,
    body_query: Query<&Transform, (With<super::render::BodyMesh>, Without<Camera3d>)>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return; // No camera or multiple cameras — skip this frame
    };

    // Get the focused body's transform using entity lookup
    let Ok(target_transform) = body_query.get(camera_controller.focus) else {
        return; // Focused entity not found (despawned or invalid), skip
    };

    let focus_pos = target_transform.translation;

    // Guard against non-finite or non-positive distance (e.g. from corrupted state)
    let distance = validate_finite_or(camera_controller.distance, 10.0, "camera distance")
        .max(0.001) as f32
        * 10.0;

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

/// Bevy system: handles mouse input for camera rotation.
/// Right-click and drag to rotate the camera around the focused body.
///
/// Uses `Option<Res<AccumulatedMouseMotion>>` so the system can run in headless
/// environments (e.g. integration tests with MinimalPlugins) where input resources
/// are not available.
pub fn mouse_camera_control(
    mut camera_controller: ResMut<CameraController>,
    mouse_motion: Option<Res<bevy::input::mouse::AccumulatedMouseMotion>>,
    mouse_button_input: Option<Res<ButtonInput<MouseButton>>>,
) {
    let Some(mouse_motion) = mouse_motion else { return };
    let Some(mouse_button_input) = mouse_button_input else { return };

    if !mouse_button_input.pressed(MouseButton::Right) {
        return;
    }

    let delta = mouse_motion.delta;
    if delta.length_squared() == 0.0 {
        return;
    }

    // Guard against non-finite delta (e.g. from input driver glitches)
    if !delta.x.is_finite() || !delta.y.is_finite() {
        return;
    }

    const MOUSE_SENSITIVITY: f64 = 0.005;
    const PITCH_MIN: f64 = -1.5;
    const PITCH_MAX: f64 = 1.5;

    camera_controller.yaw -= (delta.x as f64) * MOUSE_SENSITIVITY;
    camera_controller.pitch -= (delta.y as f64) * MOUSE_SENSITIVITY;
    camera_controller.pitch = validate_finite_or(
        camera_controller.pitch,
        std::f64::consts::FRAC_PI_6,
        "camera pitch",
    )
    .clamp(PITCH_MIN, PITCH_MAX);
    camera_controller.yaw = validate_finite_or(camera_controller.yaw, 0.0, "camera yaw");
}
