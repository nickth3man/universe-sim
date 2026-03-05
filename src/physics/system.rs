use bevy::math::DVec3;
use bevy::prelude::{Entity, Res, ResMut, Resource, Time};
use std::f64::consts::TAU;

use crate::physics::kepler::{orbital_to_cartesian, solve_keplers_equation, Orbit};

const SECONDS_PER_DAY: f64 = 86_400.0;

/// NOTE: MIN_SIMULATION_SPEED = 1.0 means the simulation cannot actually be paused
/// through the speed value alone. The UI pause button sets speed to 0.0, but
/// orbital_physics_system clamps it back to 1.0 every frame, so "pause" has no effect.
/// Changing this to 0.0 would allow true pausing.
const MIN_SIMULATION_SPEED: f64 = 1.0;
const MAX_SIMULATION_SPEED: f64 = 1_000.0;

/// Threshold for treating an orbit as circular (skip Kepler solver).
const CIRCULAR_ORBIT_EPSILON: f64 = 1.0e-10;

/// Newton-Raphson convergence tolerance in radians (~1e-12 rad = sub-nanoradian).
const KEPLER_TOLERANCE: f64 = 1.0e-12;

/// Maximum Newton-Raphson iterations. 32 is more than enough for solar-system
/// eccentricities (e ≤ 0.21); typical convergence is 3–6 iterations.
const KEPLER_MAX_ITERATIONS: u32 = 32;

/// Runtime state of a single celestial body.
#[derive(Debug, Clone)]
pub struct BodyState {
    /// Bevy entity for this body (used for entity-based lookups)
    pub entity: Entity,

    pub name: String,

    /// Orbital elements. `None` means the body is fixed at the origin (the Sun).
    pub orbit: Option<Orbit>,

    /// Heliocentric position in Astronomical Units (AU), updated every physics tick.
    pub position: DVec3,

    /// Mean anomaly at the last physics tick, cached for UI/debug display.
    pub mean_anomaly_rad: f64,
}

impl BodyState {
    pub fn new(entity: Entity, name: impl Into<String>, orbit: Option<Orbit>) -> Self {
        Self {
            entity,
            name: name.into(),
            orbit,
            position: DVec3::ZERO,
            mean_anomaly_rad: 0.0,
        }
    }
}

/// Top-level simulation state registered as a Bevy Resource.
///
/// Initialized by `init_solar_system()` in app.rs before the plugin builds,
/// so `init_resource::<AppState>()` inside `SolarSystemPlugin::build` is a no-op
/// (Bevy skips init_resource when the resource already exists).
#[derive(Debug, Resource, Clone)]
pub struct AppState {
    /// Accumulated simulation time in days since the simulation began.
    pub elapsed_days: f64,

    /// Time multiplier: 1.0 = real-time, 1000.0 = 1000 days per real second.
    /// Clamped to [MIN_SIMULATION_SPEED, MAX_SIMULATION_SPEED] each frame.
    pub simulation_speed: f64,

    /// Ordered list of all bodies. Index 0 is always the Sun (orbit = None).
    /// The order must match the entity spawn order in spawn_celestial_bodies,
    /// because update_body_transforms maps bodies to entities by parallel index.
    pub bodies: Vec<BodyState>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            elapsed_days: 0.0,
            simulation_speed: 1.0,
            bodies: Vec::new(),
        }
    }
}

/// Bevy system: advances the simulation clock and recomputes all planet positions.
///
/// Uses analytic Keplerian mechanics — positions are computed directly from the
/// current time rather than integrated from velocity/acceleration. This means:
/// - No drift accumulates over long simulation times.
/// - N-body gravitational interactions are NOT modeled.
/// - Each frame is independent; pausing and resuming produces no artifacts.
pub fn orbital_physics_system(time: Res<Time>, mut state: ResMut<AppState>) {
    // Clamp speed and write it back so the UI slider reflects the enforced range.
    // BUG: MIN = 1.0 makes it impossible to pause via simulation_speed = 0.0.
    let simulation_speed = state
        .simulation_speed
        .clamp(MIN_SIMULATION_SPEED, MAX_SIMULATION_SPEED);
    state.simulation_speed = simulation_speed;

    // Convert real frame delta to simulation days:
    // Δt_days = (Δt_seconds / 86400) * simulation_speed
    let delta_days = (time.delta().as_secs_f64() / SECONDS_PER_DAY) * simulation_speed;
    state.elapsed_days += delta_days;

    let simulation_time_days = state.elapsed_days;

    for body in &mut state.bodies {
        // Bodies without an orbit (i.e. the Sun) stay at the origin.
        let Some(orbit) = body.orbit.as_ref() else {
            continue;
        };

        // Mean motion:  n = 2π / T  (radians per day)
        // Mean anomaly: M = M₀ + n·(t − t₀)  wrapped to [0, 2π)
        // M increases linearly with time — it is the angle a fictitious body on a
        // circular orbit of the same period would have swept.
        let mean_motion = TAU / orbit.orbital_period_days;
        let mean_anomaly = (orbit.mean_anomaly_at_epoch_rad
            + (mean_motion * (simulation_time_days - orbit.epoch_days)))
            .rem_euclid(TAU);

        body.mean_anomaly_rad = mean_anomaly;

        // For circular orbits, M = E = ν directly (no solver needed).
        let true_anomaly = if orbit.eccentricity.abs() <= CIRCULAR_ORBIT_EPSILON {
            mean_anomaly
        } else {
            // Solve Kepler's equation M = E − e·sin(E) for E (eccentric anomaly).
            let eccentric_anomaly = solve_keplers_equation(
                mean_anomaly,
                orbit.eccentricity,
                KEPLER_TOLERANCE,
                KEPLER_MAX_ITERATIONS,
            );

            // Convert E → ν (true anomaly) using the stable half-angle form:
            // ν = 2·atan2(√(1+e)·sin(E/2),  √(1-e)·cos(E/2))
            // This avoids the singularity in tan(ν/2) = √((1+e)/(1-e))·tan(E/2) at ν = π.
            let half_eccentric = 0.5 * eccentric_anomaly;
            let y = (1.0 + orbit.eccentricity).sqrt() * half_eccentric.sin();
            let x = (1.0 - orbit.eccentricity).sqrt() * half_eccentric.cos();

            (2.0 * y.atan2(x)).rem_euclid(TAU)
        };

        // Convert (a, e, i, Ω, ω, ν) → heliocentric Cartesian position in AU.
        body.position = orbital_to_cartesian(
            orbit.semi_major_axis_au,
            orbit.eccentricity,
            orbit.inclination_rad,
            orbit.longitude_ascending_rad,
            orbit.argument_of_periapsis_rad,
            true_anomaly,
        );
    }
}
