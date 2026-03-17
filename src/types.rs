use bevy::prelude::*;
use nalgebra::Vector3;

/// 64-bit precision 3D vector for physics calculations
pub type DVec3 = Vector3<f64>;

/// Converts DVec3 (nalgebra f64) to Bevy's Vec3 (f32).
pub fn convert_dvec3_to_vec3(dvec: DVec3) -> Vec3 {
    Vec3::new(dvec.x as f32, dvec.y as f32, dvec.z as f32)
}

/// Converts Bevy's Vec3 (f32) to DVec3 (nalgebra f64).
pub fn convert_vec3_to_dvec3(vec: Vec3) -> DVec3 {
    DVec3::new(vec.x as f64, vec.y as f64, vec.z as f64)
}

/// Scale factor: 1 AU = X simulation units
pub const AU_TO_UNITS: f64 = 100.0;

/// Scale from AU (physics) to world units for transforms and trails.
/// Matches the multiplier used in sync_physics_to_transforms.
pub const AU_TO_WORLD: f32 = 10.0;

/// Scale factor: 1 km = X simulation units
pub const KM_TO_UNITS: f64 = 0.0001;

/// Gravitational constant in SI units (m^3 kg^-1 s^-2)
pub const G: f64 = 6.67430e-11;

/// Mass of the Sun in kg
pub const SUN_MASS_KG: f64 = 1.98847e30;

/// Astronomical Unit in meters
pub const AU_METERS: f64 = 1.495978707e11;

/// Days per year
pub const DAYS_PER_YEAR: f64 = 365.25;
