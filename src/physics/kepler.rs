use bevy::math::DVec3;
use std::f64::consts::{PI, TAU};

const CIRCULAR_ORBIT_EPSILON: f64 = 1.0e-10;

#[derive(Debug, Clone, Copy)]
pub struct Orbit {
    pub semi_major_axis_au: f64,
    pub eccentricity: f64,
    pub inclination_rad: f64,
    pub longitude_ascending_rad: f64,
    pub argument_of_periapsis_rad: f64,
    pub mean_anomaly_at_epoch_rad: f64,
    pub epoch_days: f64,
    pub orbital_period_days: f64,
}

#[allow(dead_code)]
pub fn calculate_orbit_position(orbit: &Orbit, time_days: f64) -> DVec3 {
    let mean_motion = TAU / orbit.orbital_period_days;
    let elapsed_days = time_days - orbit.epoch_days;

    // M(t) = M0 + n * Δt
    let mean_anomaly =
        normalize_0_to_tau(orbit.mean_anomaly_at_epoch_rad + (mean_motion * elapsed_days));

    // For circular orbits (e ~= 0), M = E = ν and radius is constant (r = a).
    if orbit.eccentricity.abs() <= CIRCULAR_ORBIT_EPSILON {
        return orbital_to_cartesian(
            orbit.semi_major_axis_au,
            0.0,
            orbit.inclination_rad,
            orbit.longitude_ascending_rad,
            orbit.argument_of_periapsis_rad,
            mean_anomaly,
        );
    }

    let eccentric_anomaly = solve_keplers_equation(mean_anomaly, orbit.eccentricity, 1.0e-12, 32);

    // Stable conversion from eccentric anomaly E to true anomaly ν.
    // Equivalent to: tan(ν/2) = sqrt((1+e)/(1-e)) * tan(E/2)
    let true_anomaly = eccentric_to_true_anomaly(eccentric_anomaly, orbit.eccentricity);

    orbital_to_cartesian(
        orbit.semi_major_axis_au,
        orbit.eccentricity,
        orbit.inclination_rad,
        orbit.longitude_ascending_rad,
        orbit.argument_of_periapsis_rad,
        true_anomaly,
    )
}

pub fn solve_keplers_equation(
    mean_anomaly: f64,
    eccentricity: f64,
    tolerance: f64,
    max_iterations: u32,
) -> f64 {
    let e = eccentricity.clamp(0.0, 0.999_999_999_999);
    let tolerance = tolerance.max(1.0e-16);

    if e <= CIRCULAR_ORBIT_EPSILON {
        return normalize_0_to_tau(mean_anomaly);
    }

    // Bring M to [-π, π] for better Newton convergence.
    let m = normalize_minus_pi_to_pi(mean_anomaly);

    // Initial guess:
    // - good near-circular: E0 = M
    // - high e: start near ±π to improve stability
    let mut eccentric_anomaly = if e < 0.8 {
        m
    } else if m >= 0.0 {
        PI
    } else {
        -PI
    };

    for _ in 0..max_iterations.max(1) {
        let f = eccentric_anomaly - (e * eccentric_anomaly.sin()) - m;
        let f_prime = 1.0 - (e * eccentric_anomaly.cos());

        if f_prime.abs() <= f64::EPSILON {
            break;
        }

        let delta = -f / f_prime;
        eccentric_anomaly += delta;

        if delta.abs() <= tolerance {
            break;
        }
    }

    normalize_0_to_tau(eccentric_anomaly)
}

pub fn orbital_to_cartesian(
    semi_major_axis: f64,
    eccentricity: f64,
    inclination: f64,
    longitude_ascending: f64,
    argument_of_periapsis: f64,
    true_anomaly: f64,
) -> DVec3 {
    let e = eccentricity.max(0.0);

    // Radius for an ellipse as a function of true anomaly:
    // r = a(1 - e^2) / (1 + e*cos(ν))
    // For circular orbit, this reduces to r = a.
    let radius = if e <= CIRCULAR_ORBIT_EPSILON {
        semi_major_axis
    } else {
        semi_major_axis * (1.0 - (e * e)) / (1.0 + (e * true_anomaly.cos()))
    };

    let argument_of_latitude = argument_of_periapsis + true_anomaly;

    let (sin_omega, cos_omega) = longitude_ascending.sin_cos();
    let (sin_i, cos_i) = inclination.sin_cos();
    let (sin_u, cos_u) = argument_of_latitude.sin_cos();

    // Rotation from perifocal frame to inertial reference frame.
    let x = radius * ((cos_omega * cos_u) - (sin_omega * sin_u * cos_i));
    let y = radius * ((sin_omega * cos_u) + (cos_omega * sin_u * cos_i));
    let z = radius * (sin_u * sin_i);

    DVec3::new(x, y, z)
}

#[allow(dead_code)]
fn eccentric_to_true_anomaly(eccentric_anomaly: f64, eccentricity: f64) -> f64 {
    let e = eccentricity.clamp(0.0, 0.999_999_999_999);
    let half_e = 0.5 * eccentric_anomaly;
    let sin_half_e = half_e.sin();
    let cos_half_e = half_e.cos();

    let y = (1.0 + e).sqrt() * sin_half_e;
    let x = (1.0 - e).sqrt() * cos_half_e;

    normalize_0_to_tau(2.0 * y.atan2(x))
}

fn normalize_0_to_tau(angle: f64) -> f64 {
    angle.rem_euclid(TAU)
}

fn normalize_minus_pi_to_pi(angle: f64) -> f64 {
    let wrapped = normalize_0_to_tau(angle);
    if wrapped > PI {
        wrapped - TAU
    } else {
        wrapped
    }
}
