use bevy::prelude::*;

pub mod sphere;

/// Marker component attached to every celestial body sphere entity (Sun + 8 planets).
/// Queried by `update_body_transforms` to sync physics positions → Bevy Transforms.
/// The mapping is positional: entity N corresponds to `AppState.bodies[N]`.
#[derive(Component)]
pub struct BodyMesh;

/// Marker component reserved for future orbit trail rendering (line strip behind planets).
/// Defined but never spawned — no system currently creates OrbitTrail entities.
#[derive(Component)]
#[allow(dead_code)]
pub struct OrbitTrail;

/// Marker component on the scene's DirectionalLight entity.
/// Currently unused by any query; kept as a semantic tag for future light control systems.
#[derive(Component)]
pub struct SunLight;
