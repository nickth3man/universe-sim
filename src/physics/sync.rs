use crate::physics::system::PhysicsState;
use crate::render::sphere::calculate_visual_radius;
use crate::render::BodyMesh;
use bevy::prelude::*;

pub fn sync_physics_to_transforms(
    physics: Res<PhysicsState>,
    mut query: Query<(Entity, &mut Transform), With<BodyMesh>>,
) {
    let mut is_first = true;
    for (entity, mut transform) in query.iter_mut() {
        if let Some(body_state) = physics.bodies.get(&entity) {
            transform.translation = Vec3::new(
                body_state.position.x as f32 * 10.0,
                body_state.position.y as f32 * 10.0,
                body_state.position.z as f32 * 10.0,
            );

            let visual_scale = if is_first {
                0.5
            } else {
                calculate_visual_radius(6000.0) * 2.0
            };
            transform.scale = Vec3::splat(visual_scale);
            is_first = false;
        }
    }
}
