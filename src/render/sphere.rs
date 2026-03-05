use bevy::prelude::*;

/// Creates a UV sphere mesh with the given radius, sector count (longitude slices),
/// and stack count (latitude slices). Delegates to Bevy's built-in sphere builder.
pub fn create_sphere_mesh(radius: f32, sectors: u32, stacks: u32) -> Mesh {
    Sphere::new(radius).mesh().uv(sectors, stacks)
}

/// Maps a real planetary radius in km to a visual radius in world units using a
/// logarithmic scale, so that planets spanning 3 orders of magnitude in real size
/// remain distinguishable on screen without tiny planets becoming invisible.
///
/// Formula:  visual = clamp(ln(radius_km × BASE_SCALE).max(0) × 0.3 + MIN, 0, MAX)
///
/// Example values:
///   Earth  (6 371 km) → ln(0.319) ≈ −1.14 → clamped to 0 → 0.05 (minimum)
///   Jupiter (69 911 km) → ln(3.50) ≈  1.25 → 1.25×0.3+0.05 ≈ 0.43
///   Sun  (695 700 km) → ln(34.8)  ≈  3.55 → 3.55×0.3+0.05 ≈ 1.12
pub fn calculate_visual_radius(radius_km: f64) -> f32 {
    const MIN_VISUAL_RADIUS: f64 = 0.05;
    const MAX_VISUAL_RADIUS: f64 = 3.0;
    /// Scaling factor that maps typical planetary radii into the ln() domain near 0–4.
    const BASE_SCALE: f64 = 0.00005;

    // ln() is negative for radii where radius_km × BASE_SCALE < 1 (i.e. < 20 000 km),
    // so .max(0.0) clamps those to the minimum visual size.
    let scaled = (radius_km * BASE_SCALE).ln().max(0.0) * 0.3 + MIN_VISUAL_RADIUS;
    scaled.min(MAX_VISUAL_RADIUS) as f32
}

/// Variant that enforces a caller-supplied minimum visual radius.
/// Useful when a body must remain clickable/visible regardless of its real size.
#[allow(dead_code)]
pub fn calculate_visual_radius_with_min(radius_km: f64, min_radius: f32) -> f32 {
    calculate_visual_radius(radius_km).max(min_radius)
}
