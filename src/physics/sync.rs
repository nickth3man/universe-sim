use bevy::prelude::*;
use crate::physics::system::PhysicsState;
use crate::render::BodyMesh;

/// Syncs physics positions to Bevy Transforms
///
/// Queries entities with BodyMesh component and updates their Transform
/// from the corresponding BodyState in PhysicsState
pub fn sync_physics_to_transforms(
    physics: Res<PhysicsState>,
    mut query: Query<(Entity, &mut Transform), With<BodyMesh>>,
) {
    for (entity, mut transform) in query.iter_mut() {
        if let Some(body_state) = physics.bodies.get(&entity) {
            // Convert DVec3 (f64) to Vec3 (f32) and apply 10.0 scale factor
            transform.translation = Vec3::new(
                body_state.position.x as f32 * 10.0,
                body_state.position.y as f32 * 10.0,
                body_state.position.z as f32 * 10.0,
            );
        }
    }
}
