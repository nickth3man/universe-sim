use crate::physics::CelestialBody;
use bevy::prelude::*;

/// Global application state for the solar system simulation
#[derive(Resource, Debug, Clone)]
pub struct AppState {
    /// Simulation speed multiplier (0.0 to 1000.0+)
    /// 1.0 = real-time, 10.0 = 10x speed, etc.
    pub simulation_speed: f64,

    /// Whether the simulation is paused
    pub paused: bool,

    /// Total elapsed simulation time in days
    pub elapsed_days: f64,

    /// All celestial bodies in the system
    pub bodies: Vec<CelestialBody>,

    /// Index of currently selected body (for camera focus)
    pub selected_body: Option<usize>,

    /// Camera zoom level (distance scaling factor)
    pub camera_zoom: f64,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            simulation_speed: 1.0,
            paused: false,
            elapsed_days: 0.0,
            bodies: Vec::new(),
            selected_body: None,
            camera_zoom: 1.0,
        }
    }
}

impl AppState {
    /// Create a new app state with default solar system
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new app state with a predefined solar system
    pub fn with_solar_system() -> Self {
        use crate::physics::Orbit;
        use crate::types::colors;

        let bodies = vec![
            CelestialBody::new("Sun", 1.98847e30, 696_340.0, colors::SUN),
            CelestialBody::with_orbit(
                "Mercury",
                3.3011e23,
                2_439.7,
                colors::MERCURY,
                Orbit::new(0.387, 0.2056, 87.97, 0.0, 0.0, 0.0, 0.0),
            ),
            CelestialBody::with_orbit(
                "Venus",
                4.8675e24,
                6_051.8,
                colors::VENUS,
                Orbit::new(0.723, 0.0067, 224.70, 0.0, 0.0, 0.0, 0.0),
            ),
            CelestialBody::with_orbit(
                "Earth",
                5.9722e24,
                6_371.0,
                colors::EARTH,
                Orbit::new(1.0, 0.0167, 365.25, 0.0, 0.0, 0.0, 0.0),
            ),
            CelestialBody::with_orbit(
                "Mars",
                6.4171e23,
                3_389.5,
                colors::MARS,
                Orbit::new(1.524, 0.0934, 686.98, 0.0, 0.0, 0.0, 0.0),
            ),
            CelestialBody::with_orbit(
                "Jupiter",
                1.8982e27,
                69_911.0,
                colors::JUPITER,
                Orbit::new(5.203, 0.0489, 4_332.59, 0.0, 0.0, 0.0, 0.0),
            ),
            CelestialBody::with_orbit(
                "Saturn",
                5.6834e26,
                58_232.0,
                colors::SATURN,
                Orbit::new(9.537, 0.0565, 10_759.22, 0.0, 0.0, 0.0, 0.0),
            ),
            CelestialBody::with_orbit(
                "Uranus",
                8.6810e25,
                25_362.0,
                colors::URANUS,
                Orbit::new(19.191, 0.0457, 30_688.50, 0.0, 0.0, 0.0, 0.0),
            ),
            CelestialBody::with_orbit(
                "Neptune",
                1.02413e26,
                24_622.0,
                colors::NEPTUNE,
                Orbit::new(30.07, 0.0113, 60_195.00, 0.0, 0.0, 0.0, 0.0),
            ),
        ];

        Self {
            bodies,
            ..Default::default()
        }
    }

    /// Get a reference to a body by index
    pub fn get_body(&self, index: usize) -> Option<&CelestialBody> {
        self.bodies.get(index)
    }

    /// Get a mutable reference to a body by index
    pub fn get_body_mut(&mut self, index: usize) -> Option<&mut CelestialBody> {
        self.bodies.get_mut(index)
    }

    /// Advance the simulation by the given number of days
    pub fn advance_time(&mut self, days: f64) {
        if self.paused {
            return;
        }
        let scaled_days = days * self.simulation_speed;
        self.elapsed_days += scaled_days;

        for body in &mut self.bodies {
            if let Some(ref mut orbit) = body.orbit {
                orbit.advance(scaled_days);
            }
        }
    }

    /// Toggle pause state
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    /// Set simulation speed (clamped to 0.0 - 10000.0)
    pub fn set_speed(&mut self, speed: f64) {
        self.simulation_speed = speed.clamp(0.0, 10000.0);
    }
}
