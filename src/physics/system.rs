use bevy::math::DVec3;
use bevy::prelude::{Res, ResMut, Resource, Time};
use std::f64::consts::TAU;

use crate::physics::kepler::{orbital_to_cartesian, solve_keplers_equation, Orbit};

const SECONDS_PER_DAY: f64 = 86_400.0;
const MIN_SIMULATION_SPEED: f64 = 1.0;
const MAX_SIMULATION_SPEED: f64 = 1_000.0;
const CIRCULAR_ORBIT_EPSILON: f64 = 1.0e-10;
const KEPLER_TOLERANCE: f64 = 1.0e-12;
const KEPLER_MAX_ITERATIONS: u32 = 32;

#[derive(Debug, Clone)]
pub struct BodyState {
    pub name: String,
    pub orbit: Option<Orbit>,
    pub position: DVec3,
    pub mean_anomaly_rad: f64,
}

impl BodyState {
    pub fn new(name: impl Into<String>, orbit: Option<Orbit>) -> Self {
        Self {
            name: name.into(),
            orbit,
            position: DVec3::ZERO,
            mean_anomaly_rad: 0.0,
        }
    }
}

#[derive(Debug, Resource, Clone)]
pub struct AppState {
    pub elapsed_days: f64,
    pub simulation_speed: f64,
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

pub fn orbital_physics_system(time: Res<Time>, mut state: ResMut<AppState>) {
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
        let Some(orbit) = body.orbit.as_ref() else {
            continue;
        };

        // Mean motion and mean anomaly at current time.
        // n = 2π / T
        // M = M0 + n * (t - t0)
        let mean_motion = TAU / orbit.orbital_period_days;
        let mean_anomaly = (orbit.mean_anomaly_at_epoch_rad
            + (mean_motion * (simulation_time_days - orbit.epoch_days)))
            .rem_euclid(TAU);

        body.mean_anomaly_rad = mean_anomaly;

        // Circular orbit special case: M = E = ν.
        let true_anomaly = if orbit.eccentricity.abs() <= CIRCULAR_ORBIT_EPSILON {
            mean_anomaly
        } else {
            let eccentric_anomaly = solve_keplers_equation(
                mean_anomaly,
                orbit.eccentricity,
                KEPLER_TOLERANCE,
                KEPLER_MAX_ITERATIONS,
            );

            // tan(ν/2) = sqrt((1+e)/(1-e)) * tan(E/2)
            let half_eccentric = 0.5 * eccentric_anomaly;
            let y = (1.0 + orbit.eccentricity).sqrt() * half_eccentric.sin();
            let x = (1.0 - orbit.eccentricity).sqrt() * half_eccentric.cos();

            (2.0 * y.atan2(x)).rem_euclid(TAU)
        };

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
