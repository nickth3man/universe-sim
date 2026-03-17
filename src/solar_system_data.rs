//! Solar system orbital data: JPL elements for Sun, 8 planets, and 5 major moons.
//!
//! Source: NASA planetary fact sheets / JPL Horizons epoch J2000.0.
//! All angular elements in degrees (converted to radians at use site).

use crate::physics::kepler::Orbit;

/// Degrees to radians conversion constant.
pub const DEG: f64 = std::f64::consts::PI / 180.0;

/// AU in km (for converting moon semi-major axes from km to AU).
const AU_KM: f64 = 149_597_870.7;

/// Planet orbital data: (name, radius_km, orbital elements).
#[derive(Clone, Copy)]
pub struct PlanetData {
    pub name: &'static str,
    pub radius_km: f64,
    pub orbit: Orbit,
}

/// Moon orbital data: (name, radius_km, semi_major_axis_km, orbit params, parent_index).
#[derive(Clone, Copy)]
pub struct MoonData {
    pub name: &'static str,
    pub radius_km: f64,
    pub semi_major_axis_km: f64,
    pub eccentricity: f64,
    pub inclination_deg: f64,
    pub longitude_ascending_deg: f64,
    pub argument_of_periapsis_deg: f64,
    pub mean_anomaly_deg: f64,
    pub orbital_period_days: f64,
    /// Index into [Sun, Mercury, Venus, Earth, Mars, Jupiter, Saturn, Uranus, Neptune] (0-8).
    pub parent_index: usize,
}

/// Orbital elements for the 8 planets (JPL Approximate Positions, J2000.0).
pub const PLANET_DATA: [PlanetData; 8] = [
    PlanetData {
        name: "Mercury",
        radius_km: 2_440.53,
        orbit: Orbit {
            semi_major_axis_au: 0.38709927,
            eccentricity: 0.20563593,
            inclination_rad: 7.00497902 * DEG,
            longitude_ascending_rad: 48.33076593 * DEG,
            argument_of_periapsis_rad: 29.12703 * DEG,
            mean_anomaly_at_epoch_rad: 174.79253 * DEG,
            epoch_days: 0.0,
            orbital_period_days: 87.969,
        },
    },
    PlanetData {
        name: "Venus",
        radius_km: 6_051.8,
        orbit: Orbit {
            semi_major_axis_au: 0.72333566,
            eccentricity: 0.00677672,
            inclination_rad: 3.39467605 * DEG,
            longitude_ascending_rad: 76.67984255 * DEG,
            argument_of_periapsis_rad: 54.92262 * DEG,
            mean_anomaly_at_epoch_rad: 50.37663 * DEG,
            epoch_days: 0.0,
            orbital_period_days: 224.701,
        },
    },
    PlanetData {
        name: "Earth",
        radius_km: 6_378.14,
        orbit: Orbit {
            semi_major_axis_au: 1.00000261,
            eccentricity: 0.01671123,
            inclination_rad: -0.00001531 * DEG,
            longitude_ascending_rad: 0.0,
            argument_of_periapsis_rad: 102.93768 * DEG,
            mean_anomaly_at_epoch_rad: 357.52689 * DEG,
            epoch_days: 0.0,
            orbital_period_days: 365.256,
        },
    },
    PlanetData {
        name: "Mars",
        radius_km: 3_396.19,
        orbit: Orbit {
            semi_major_axis_au: 1.52371034,
            eccentricity: 0.09339410,
            inclination_rad: 1.84969142 * DEG,
            longitude_ascending_rad: 49.55953891 * DEG,
            argument_of_periapsis_rad: 286.49683 * DEG,
            mean_anomaly_at_epoch_rad: 19.41248 * DEG,
            epoch_days: 0.0,
            orbital_period_days: 686.980,
        },
    },
    PlanetData {
        name: "Jupiter",
        radius_km: 71_492.0,
        orbit: Orbit {
            semi_major_axis_au: 5.20288700,
            eccentricity: 0.04838624,
            inclination_rad: 1.30439695 * DEG,
            longitude_ascending_rad: 100.47390909 * DEG,
            argument_of_periapsis_rad: 274.25452 * DEG,
            mean_anomaly_at_epoch_rad: 19.66796 * DEG,
            epoch_days: 0.0,
            orbital_period_days: 4332.82,
        },
    },
    PlanetData {
        name: "Saturn",
        radius_km: 60_268.0,
        orbit: Orbit {
            semi_major_axis_au: 9.53667594,
            eccentricity: 0.05386179,
            inclination_rad: 2.48599187 * DEG,
            longitude_ascending_rad: 113.66242448 * DEG,
            argument_of_periapsis_rad: 338.93605 * DEG,
            mean_anomaly_at_epoch_rad: 317.35537 * DEG,
            epoch_days: 0.0,
            orbital_period_days: 10759.22,
        },
    },
    PlanetData {
        name: "Uranus",
        radius_km: 25_559.0,
        orbit: Orbit {
            semi_major_axis_au: 19.18916464,
            eccentricity: 0.04725744,
            inclination_rad: 0.77263783 * DEG,
            longitude_ascending_rad: 74.01692503 * DEG,
            argument_of_periapsis_rad: 96.93735 * DEG,
            mean_anomaly_at_epoch_rad: 142.28383 * DEG,
            epoch_days: 0.0,
            orbital_period_days: 30687.15,
        },
    },
    PlanetData {
        name: "Neptune",
        radius_km: 24_764.0,
        orbit: Orbit {
            semi_major_axis_au: 30.06992276,
            eccentricity: 0.00859048,
            inclination_rad: 1.77004347 * DEG,
            longitude_ascending_rad: 131.78422574 * DEG,
            argument_of_periapsis_rad: 273.17949 * DEG,
            mean_anomaly_at_epoch_rad: 259.91521 * DEG,
            epoch_days: 0.0,
            orbital_period_days: 60190.03,
        },
    },
];

/// Orbital data for 5 major moons (Moon, Io, Europa, Ganymede, Callisto).
/// JPL Planetary Satellite Mean Elements, DE405/LE405 (Moon), JUP365 (Galilean).
pub const MOON_DATA: [MoonData; 5] = [
    MoonData {
        name: "Moon",
        radius_km: 1_737.4,
        semi_major_axis_km: 384_400.0,
        eccentricity: 0.0554,
        inclination_deg: 5.16,
        longitude_ascending_deg: 125.08,
        argument_of_periapsis_deg: 318.15,
        mean_anomaly_deg: 135.27,
        orbital_period_days: 27.322,
        parent_index: 3, // Earth
    },
    MoonData {
        name: "Io",
        radius_km: 1_821.49,
        semi_major_axis_km: 421_800.0,
        eccentricity: 0.004,
        inclination_deg: 0.0,
        longitude_ascending_deg: 0.0,
        argument_of_periapsis_deg: 49.1,
        mean_anomaly_deg: 330.9,
        orbital_period_days: 1.769,
        parent_index: 5, // Jupiter
    },
    MoonData {
        name: "Europa",
        radius_km: 1_560.80,
        semi_major_axis_km: 671_100.0,
        eccentricity: 0.009,
        inclination_deg: 0.5,
        longitude_ascending_deg: 184.0,
        argument_of_periapsis_deg: 45.0,
        mean_anomaly_deg: 345.4,
        orbital_period_days: 3.525,
        parent_index: 5, // Jupiter
    },
    MoonData {
        name: "Ganymede",
        radius_km: 2_631.20,
        semi_major_axis_km: 1_070_400.0,
        eccentricity: 0.001,
        inclination_deg: 0.2,
        longitude_ascending_deg: 58.5,
        argument_of_periapsis_deg: 198.3,
        mean_anomaly_deg: 324.8,
        orbital_period_days: 7.156,
        parent_index: 5, // Jupiter
    },
    MoonData {
        name: "Callisto",
        radius_km: 2_410.30,
        semi_major_axis_km: 1_882_700.0,
        eccentricity: 0.007,
        inclination_deg: 0.3,
        longitude_ascending_deg: 309.1,
        argument_of_periapsis_deg: 43.8,
        mean_anomaly_deg: 87.4,
        orbital_period_days: 16.690,
        parent_index: 5, // Jupiter
    },
];

impl MoonData {
    /// Builds an Orbit from moon data (semi-major axis in AU).
    pub fn to_orbit(&self) -> Orbit {
        Orbit {
            semi_major_axis_au: self.semi_major_axis_km / AU_KM,
            eccentricity: self.eccentricity,
            inclination_rad: self.inclination_deg * DEG,
            longitude_ascending_rad: self.longitude_ascending_deg * DEG,
            argument_of_periapsis_rad: self.argument_of_periapsis_deg * DEG,
            mean_anomaly_at_epoch_rad: self.mean_anomaly_deg * DEG,
            epoch_days: 0.0,
            orbital_period_days: self.orbital_period_days,
        }
    }
}
