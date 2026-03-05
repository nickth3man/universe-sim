use bevy::prelude::*;
use nalgebra::Vector3;

/// 64-bit precision 3D vector for physics calculations
pub type DVec3 = Vector3<f64>;

/// Convert DVec3 (nalgebra f64) to Bevy's Vec3 (f32)
pub fn dvec3_to_vec3(dvec: DVec3) -> Vec3 {
    Vec3::new(dvec.x as f32, dvec.y as f32, dvec.z as f32)
}

/// Convert Bevy's Vec3 (f32) to DVec3 (nalgebra f64)
pub fn vec3_to_dvec3(vec: Vec3) -> DVec3 {
    DVec3::new(vec.x as f64, vec.y as f64, vec.z as f64)
}

/// Scale factor: 1 AU = X simulation units
pub const AU_TO_UNITS: f64 = 100.0;

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

/// Standard colors for celestial bodies
pub mod colors {
    use bevy::prelude::Color;

    pub const SUN: Color = Color::srgb(1.0, 0.9, 0.2);
    pub const MERCURY: Color = Color::srgb(0.6, 0.6, 0.6);
    pub const VENUS: Color = Color::srgb(0.9, 0.7, 0.3);
    pub const EARTH: Color = Color::srgb(0.2, 0.5, 0.8);
    pub const MARS: Color = Color::srgb(0.8, 0.3, 0.1);
    pub const JUPITER: Color = Color::srgb(0.8, 0.6, 0.4);
    pub const SATURN: Color = Color::srgb(0.9, 0.8, 0.5);
    pub const URANUS: Color = Color::srgb(0.4, 0.8, 0.9);
    pub const NEPTUNE: Color = Color::srgb(0.2, 0.4, 0.9);
    pub const MOON: Color = Color::srgb(0.8, 0.8, 0.8);
}
