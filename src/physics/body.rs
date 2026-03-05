use crate::types::DVec3;
use bevy::prelude::*;

/// Represents a celestial body (sun, planet, moon, asteroid, etc.)
#[derive(Debug, Clone)]
pub struct CelestialBody {
    /// Name of the body (e.g., "Earth", "Mars")
    pub name: String,

    /// Mass in kilograms
    pub mass_kg: f64,

    /// Radius in kilometers
    pub radius_km: f64,

    /// Visual color for rendering
    pub color: Color,

    /// Orbital parameters (None for the central star)
    pub orbit: Option<Orbit>,

    /// Current position in 3D space (simulation units)
    pub position: DVec3,
}

impl CelestialBody {
    /// Create a new celestial body
    pub fn new(name: impl Into<String>, mass_kg: f64, radius_km: f64, color: Color) -> Self {
        Self {
            name: name.into(),
            mass_kg,
            radius_km,
            color,
            orbit: None,
            position: DVec3::zeros(),
        }
    }

    /// Create a new body with an orbit
    pub fn with_orbit(
        name: impl Into<String>,
        mass_kg: f64,
        radius_km: f64,
        color: Color,
        orbit: Orbit,
    ) -> Self {
        Self {
            name: name.into(),
            mass_kg,
            radius_km,
            color,
            orbit: Some(orbit),
            position: DVec3::zeros(),
        }
    }

    /// Get the body's position scaled for rendering
    pub fn get_scaled_position(&self) -> Vec3 {
        crate::types::dvec3_to_vec3(self.position)
    }
}

/// Keplerian orbital elements for describing orbits
#[derive(Debug, Clone, Copy)]
pub struct Orbit {
    /// Semi-major axis in AU (average distance from parent)
    pub semi_major_axis: f64,

    /// Eccentricity (0 = circular, <1 = elliptical)
    pub eccentricity: f64,

    /// Orbital period in days
    pub period_days: f64,

    /// Inclination in radians (tilt relative to reference plane)
    pub inclination: f64,

    /// Longitude of ascending node in radians
    pub longitude_ascending: f64,

    /// Argument of periapsis in radians
    pub argument_of_periapsis: f64,

    /// Mean anomaly in radians (current position in orbit)
    pub mean_anomaly: f64,
}

impl Orbit {
    /// Create a new circular orbit
    pub fn circular(semi_major_axis_au: f64, period_days: f64) -> Self {
        Self {
            semi_major_axis: semi_major_axis_au,
            eccentricity: 0.0,
            period_days,
            inclination: 0.0,
            longitude_ascending: 0.0,
            argument_of_periapsis: 0.0,
            mean_anomaly: 0.0,
        }
    }

    /// Create an elliptical orbit with all parameters
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        semi_major_axis: f64,
        eccentricity: f64,
        period_days: f64,
        inclination: f64,
        longitude_ascending: f64,
        argument_of_periapsis: f64,
        mean_anomaly: f64,
    ) -> Self {
        Self {
            semi_major_axis,
            eccentricity,
            period_days,
            inclination,
            longitude_ascending,
            argument_of_periapsis,
            mean_anomaly,
        }
    }

    /// Calculate mean motion (radians per day)
    pub fn mean_motion(&self) -> f64 {
        2.0 * std::f64::consts::PI / self.period_days
    }

    /// Update mean anomaly based on elapsed time
    pub fn advance(&mut self, days: f64) {
        self.mean_anomaly += self.mean_motion() * days;
        let two_pi = 2.0 * std::f64::consts::PI;
        self.mean_anomaly = self.mean_anomaly.rem_euclid(two_pi);
    }
}
