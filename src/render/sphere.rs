use bevy::log::warn;
use bevy::prelude::*;

/// Minimum sphere resolution to avoid degenerate meshes.
const MIN_UV_RESOLUTION: u32 = 4;

/// Creates a UV sphere mesh with the given radius, sector count (longitude slices),
/// and stack count (latitude slices). Delegates to Bevy's built-in sphere builder.
///
/// Validates inputs: uses 1.0 for invalid radius, clamps sector/stack to valid ranges
/// to avoid degenerate or non-renderable meshes.
pub fn create_sphere_mesh(radius: f32, sectors: u32, stacks: u32) -> Mesh {
    let radius = if radius.is_finite() && radius > 0.0 {
        radius
    } else {
        warn!(
            "create_sphere_mesh: invalid radius ({}), using 1.0",
            radius
        );
        1.0
    };
    let sectors = sectors.max(MIN_UV_RESOLUTION);
    let stacks = stacks.max(MIN_UV_RESOLUTION);
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

    // Guard against non-finite or non-positive radius (ln(0) and ln(neg) are invalid)
    let radius_km = if radius_km.is_finite() && radius_km > 0.0 {
        radius_km
    } else {
        warn!(
            "calculate_visual_radius: invalid radius_km ({}), using MIN_VISUAL_RADIUS",
            radius_km
        );
        return MIN_VISUAL_RADIUS as f32;
    };

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
