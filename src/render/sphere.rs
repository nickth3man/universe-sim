use bevy::prelude::*;

pub fn create_sphere_mesh(radius: f32, sectors: u32, stacks: u32) -> Mesh {
    Sphere::new(radius).mesh().uv(sectors, stacks)
}

pub fn calculate_visual_radius(radius_km: f64) -> f32 {
    const MIN_VISUAL_RADIUS: f64 = 0.05;
    const MAX_VISUAL_RADIUS: f64 = 3.0;
    const BASE_SCALE: f64 = 0.00005;

    let scaled = (radius_km * BASE_SCALE).ln().max(0.0) * 0.3 + MIN_VISUAL_RADIUS;
    scaled.min(MAX_VISUAL_RADIUS) as f32
}

#[allow(dead_code)]
pub fn calculate_visual_radius_with_min(radius_km: f64, min_radius: f32) -> f32 {
    calculate_visual_radius(radius_km).max(min_radius)
}
