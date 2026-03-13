use crate::error::validate_finite_or;
use bevy::log::warn;
use tracing::info_span;
use bevy::math::DVec3;
use bevy::prelude::{Entity, Res, ResMut, Resource, Time};
use std::collections::HashMap;
use std::f64::consts::TAU;

use crate::physics::kepler::{orbital_to_cartesian, solve_kepler_equation, Orbit};

const SECONDS_PER_DAY: f64 = 86_400.0;

/// Realtime = 1 sim day per real day (speed 1.0).
const DAYS_PER_YEAR: f64 = 365.25;

const MIN_SIMULATION_SPEED: f64 = 0.0; // 0 = paused
const MAX_SIMULATION_SPEED: f64 = DAYS_PER_YEAR * SECONDS_PER_DAY; // ~31.5M = 1 year per real second

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

    /// Parent entity for hierarchical orbits (moons). `None` = orbits the Sun (heliocentric).
    pub parent_entity: Option<Entity>,

    /// Radius in km for visual scaling. Planets ~6000, moons ~1500–2600.
    pub radius_km: f64,

    /// Heliocentric (or barycentric) position in Astronomical Units (AU), updated every physics tick.
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
            parent_entity: None,
            radius_km: 6000.0,
            position: DVec3::ZERO,
            mean_anomaly_rad: 0.0,
        }
    }

    /// Sets radius in km for visual scaling. Chain after `new()`.
    pub fn with_radius(mut self, radius_km: f64) -> Self {
        self.radius_km = radius_km;
        self
    }

    /// Create a moon orbiting a parent body (geocentric or jovicentric).
    pub fn moon(
        entity: Entity,
        name: impl Into<String>,
        orbit: Orbit,
        parent_entity: Entity,
        radius_km: f64,
    ) -> Self {
        Self {
            entity,
            name: name.into(),
            orbit: Some(orbit),
            parent_entity: Some(parent_entity),
            radius_km,
            position: DVec3::ZERO,
            mean_anomaly_rad: 0.0,
        }
    }
}

/// New physics state using entity-based lookups (replaces Vec-based AppState)
#[derive(Debug, Resource, Clone)]
pub struct PhysicsState {
    /// Accumulated simulation time in days since the simulation began.
    pub elapsed_days: f64,

    /// Time multiplier: 1.0 = real-time, 1000.0 = 1000 days per real second.
    /// Clamped to [0.0, MAX_SIMULATION_SPEED] each frame (0.0 enables pause).
    pub simulation_speed: f64,

    /// Map from entity to body state. Enables dynamic add/remove of bodies.
    pub bodies: HashMap<Entity, BodyState>,
}

impl Default for PhysicsState {
    fn default() -> Self {
        Self {
            elapsed_days: 0.0,
            simulation_speed: 1.0,
            bodies: HashMap::new(),
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
pub fn orbital_physics_system(time: Res<Time>, mut state: ResMut<PhysicsState>) {
    let _span = info_span!("orbital_physics_system").entered();
    // Clamp speed and write it back so the UI slider reflects the enforced range.
    let simulation_speed = validate_finite_or(state.simulation_speed, 1.0, "simulation_speed")
        .clamp(MIN_SIMULATION_SPEED, MAX_SIMULATION_SPEED);
    state.simulation_speed = simulation_speed;

    // Convert real frame delta to simulation days:
    // Δt_days = (Δt_seconds / 86400) * simulation_speed
    let delta_secs = time.delta().as_secs_f64();
    let delta_days = if delta_secs.is_finite() && delta_secs >= 0.0 {
        (delta_secs / SECONDS_PER_DAY) * simulation_speed
    } else {
        warn!(
            "Invalid frame delta ({} s), skipping time advance",
            delta_secs
        );
        0.0
    };
    state.elapsed_days = validate_finite_or(
        state.elapsed_days + delta_days,
        0.0,
        "elapsed_days",
    );

    let simulation_time_days = state.elapsed_days;

    /// Compute position for a body from its orbit and time; returns (position, mean_anomaly).
    fn compute_orbit_position(orbit: &Orbit, time_days: f64) -> Option<(DVec3, f64)> {
        if !orbit.is_valid() {
            return None;
        }
        let mean_motion = TAU / orbit.orbital_period_days;
        let mean_anomaly = (orbit.mean_anomaly_at_epoch_rad
            + (mean_motion * (time_days - orbit.epoch_days)))
            .rem_euclid(TAU);

        let true_anomaly = if orbit.eccentricity.abs() <= CIRCULAR_ORBIT_EPSILON {
            mean_anomaly
        } else {
            let eccentric_anomaly = solve_kepler_equation(
                mean_anomaly,
                orbit.eccentricity,
                KEPLER_TOLERANCE,
                KEPLER_MAX_ITERATIONS,
            );
            let half_eccentric = 0.5 * eccentric_anomaly;
            let y = (1.0 + orbit.eccentricity).sqrt() * half_eccentric.sin();
            let x = (1.0 - orbit.eccentricity).sqrt() * half_eccentric.cos();
            (2.0 * y.atan2(x)).rem_euclid(TAU)
        };

        let pos = orbital_to_cartesian(
            orbit.semi_major_axis_au,
            orbit.eccentricity,
            orbit.inclination_rad,
            orbit.longitude_ascending_rad,
            orbit.argument_of_periapsis_rad,
            true_anomaly,
        );
        Some((pos, mean_anomaly))
    }

    // Pass 1: compute positions for bodies without a parent (Sun stays at origin, planets heliocentric).
    for body in state.bodies.values_mut() {
        let Some(orbit) = body.orbit.as_ref() else {
            continue;
        };
        if body.parent_entity.is_some() {
            continue; // Moons handled in pass 2
        }
        if !orbit.is_valid() {
            warn!(
                "Body '{}' has invalid orbit (a={}, e={}, T={}), skipping position update",
                body.name,
                orbit.semi_major_axis_au,
                orbit.eccentricity,
                orbit.orbital_period_days
            );
            continue;
        }
        if let Some((pos, mean_anomaly)) = compute_orbit_position(orbit, simulation_time_days) {
            body.position = pos;
            body.mean_anomaly_rad = mean_anomaly;
        }
    }

    // Pass 2: compute moon positions (parent-centric orbit + parent's barycentric position).
    // Collect parent positions first to avoid borrowing state mutably and immutably.
    let parent_positions: HashMap<Entity, DVec3> = state
        .bodies
        .iter()
        .map(|(e, b)| (*e, b.position))
        .collect();

    for body in state.bodies.values_mut() {
        let Some(parent_entity) = body.parent_entity else {
            continue;
        };
        let Some(orbit) = body.orbit.as_ref() else {
            continue;
        };
        if !orbit.is_valid() {
            warn!(
                "Moon '{}' has invalid orbit, skipping position update",
                body.name
            );
            continue;
        }
        let parent_position = match parent_positions.get(&parent_entity) {
            Some(&pos) if pos.x.is_finite() && pos.y.is_finite() && pos.z.is_finite() => pos,
            Some(_) => {
                warn!(
                    "Moon '{}' parent has non-finite position, skipping position update",
                    body.name
                );
                continue;
            }
            None => {
                warn!(
                    "Moon '{}' has missing parent entity {:?}, skipping position update",
                    body.name, parent_entity
                );
                continue;
            }
        };
        if let Some((moon_pos_parent_frame, mean_anomaly)) =
            compute_orbit_position(orbit, simulation_time_days)
        {
            body.position = parent_position + moon_pos_parent_frame;
            body.mean_anomaly_rad = mean_anomaly;
        }
    }
}
