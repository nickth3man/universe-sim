use bevy::prelude::*;

pub mod sphere;

#[derive(Component)]
pub struct BodyMesh;

#[derive(Component)]
#[allow(dead_code)]
pub struct OrbitTrail;

#[derive(Component)]
pub struct SunLight;
