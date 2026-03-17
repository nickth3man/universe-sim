use crate::physics::system::PhysicsState;
use crate::render::sphere::get_visual_radius;
use crate::render::BodyMesh;
use crate::types::AU_TO_WORLD;
use bevy::log::warn;
use bevy::prelude::*;
use tracing::info_span;

pub fn sync_physics_to_transforms(
    physics: Res<PhysicsState>,
    mut query: Query<(Entity, &mut Transform), With<BodyMesh>>,
) {
    let _span = info_span!("sync_physics_to_transforms").entered();
    for (entity, mut transform) in query.iter_mut() {
        if let Some(body_state) = physics.bodies.get(&entity) {
            // Guard against non-finite position (corrupted physics state)
            let (x, y, z) = if body_state.position.x.is_finite()
                && body_state.position.y.is_finite()
                && body_state.position.z.is_finite()
            {
                (
                    body_state.position.x as f32 * AU_TO_WORLD,
                    body_state.position.y as f32 * AU_TO_WORLD,
                    body_state.position.z as f32 * AU_TO_WORLD,
                )
            } else {
                warn!(
                    "Body '{}' has non-finite position ({:?}), using origin",
                    body_state.name, body_state.position
                );
                (0.0, 0.0, 0.0)
            };
            transform.translation = Vec3::new(x, y, z);

            // All bodies use full-range logarithmic visual scaling from radius_km.
            // Sun, planets, and moons are all visible and proportionally sized.
            let visual_scale = get_visual_radius(body_state.radius_km);
            transform.scale = Vec3::splat(visual_scale);
        }
    }
}
