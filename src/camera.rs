use crate::physics::system::AppState;
use bevy::prelude::*;

/// Camera controller resource
#[derive(Resource)]
pub struct CameraController {
    pub distance: f64,
    pub focus_index: usize,
    pub pitch: f64,
    pub yaw: f64,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            distance: 10.0,
            focus_index: 0,
            pitch: std::f64::consts::FRAC_PI_6,
            yaw: 0.0,
        }
    }
}

pub fn camera_follow_system(
    state: Res<AppState>,
    camera_controller: Res<CameraController>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    if let Ok(mut camera_transform) = camera_query.single_mut() {
        if let Some(focus_body) = state.bodies.get(camera_controller.focus_index) {
            let focus_pos = Vec3::new(
                focus_body.position.x as f32 * 10.0,
                focus_body.position.y as f32 * 10.0,
                focus_body.position.z as f32 * 10.0,
            );

            let distance = camera_controller.distance as f32 * 10.0;
            let pitch = camera_controller.pitch as f32;
            let yaw = camera_controller.yaw as f32;

            let x = distance * pitch.cos() * yaw.cos();
            let y = distance * pitch.sin();
            let z = distance * pitch.cos() * yaw.sin();

            camera_transform.translation = focus_pos + Vec3::new(x, y, z);
            camera_transform.look_at(focus_pos, Vec3::Y);
        }
    }
}
