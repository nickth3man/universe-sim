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
        warn!("create_sphere_mesh: invalid radius ({}), using 1.0", radius);
        1.0
    };
    let sectors = sectors.max(MIN_UV_RESOLUTION);
    let stacks = stacks.max(MIN_UV_RESOLUTION);
    Sphere::new(radius).mesh().uv(sectors, stacks)
}

/// Maps a real planetary radius in km to a visual radius in world units using
/// full-range logarithmic scaling. All bodies from Moon (~1,700 km) to Sun
/// (~696,000 km) are visible and distinguishable.
///
/// Formula: t = (ln(radius_km) - ln(R_MIN)) / (ln(R_MAX) - ln(R_MIN))  clamped to [0,1]
///          visual = VISUAL_MIN + t * (VISUAL_MAX - VISUAL_MIN)
///
/// Example values:
///   Moon   (1,737 km) → t≈0.00 → 0.08
///   Earth  (6,378 km) → t≈0.16 → 0.39
///   Jupiter (71,492 km) → t≈0.67 → 1.35
///   Sun   (695,700 km) → t≈1.00 → 2.0
pub fn get_visual_radius(radius_km: f64) -> f32 {
    const R_MIN_KM: f64 = 1_600.0;
    const R_MAX_KM: f64 = 696_000.0;
    const VISUAL_MIN: f32 = 0.08;
    const VISUAL_MAX: f32 = 2.0;

    let radius_km = if radius_km.is_finite() && radius_km > 0.0 {
        radius_km
    } else {
        warn!(
            "get_visual_radius: invalid radius_km ({}), using VISUAL_MIN",
            radius_km
        );
        return VISUAL_MIN;
    };

    let ln_min = R_MIN_KM.ln();
    let ln_max = R_MAX_KM.ln();
    let t = ((radius_km.ln() - ln_min) / (ln_max - ln_min)).clamp(0.0, 1.0);
    VISUAL_MIN + (t as f32) * (VISUAL_MAX - VISUAL_MIN)
}

/// Variant that enforces a caller-supplied minimum visual radius.
/// Useful when a body must remain clickable/visible regardless of its real size.
#[allow(dead_code)]
pub fn get_visual_radius_with_min(radius_km: f64, min_radius: f32) -> f32 {
    get_visual_radius(radius_km).max(min_radius)
}
