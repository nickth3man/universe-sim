/// Pure Keplerian orbital mechanics — no ECS, no Bevy, just math.
///
/// Analytic (not numerical integration) approach: given a time t, each body's
/// position is computed directly via Kepler's equation rather than by integrating
/// velocity. This makes the simulation time-step independent and free of drift.
use bevy::math::DVec3;
use std::f64::consts::{PI, TAU};

/// Threshold below which an orbit is treated as perfectly circular.
/// At e < 1e-10, the Newton-Raphson solver would converge in 0 iterations anyway,
/// and the true/mean anomaly distinction vanishes.
const CIRCULAR_ORBIT_EPSILON: f64 = 1.0e-10;

/// Classical Keplerian orbital elements describing a two-body elliptical orbit.
///
/// All angles are in radians and all distances are in Astronomical Units (AU).
/// The reference plane is the ecliptic, with the reference direction toward the
/// vernal equinox.
#[derive(Debug, Clone, Copy)]
pub struct Orbit {
    /// Semi-major axis (a) — half the longest diameter of the ellipse, in AU.
    /// Determines the orbital size and (via Kepler's third law) the period.
    pub semi_major_axis_au: f64,

    /// Eccentricity (e) — shape of the ellipse. 0 = circle, 0–1 = ellipse.
    /// Mercury has the highest eccentricity (0.2056) among the 8 planets.
    pub eccentricity: f64,

    /// Inclination (i) — tilt of the orbital plane relative to the ecliptic.
    pub inclination_rad: f64,

    /// Longitude of the ascending node (Ω) — the angle from the reference
    /// direction to where the orbit crosses the ecliptic going northward.
    pub longitude_ascending_rad: f64,

    /// Argument of periapsis (ω) — angle within the orbital plane from the
    /// ascending node to the point of closest approach (perihelion).
    pub argument_of_periapsis_rad: f64,

    /// Mean anomaly at epoch (M₀) — the "angle" a fictitious body on a
    /// circular orbit of period T would have at time epoch_days.
    pub mean_anomaly_at_epoch_rad: f64,

    /// Epoch (t₀) in Julian days — the reference time for mean_anomaly_at_epoch.
    pub epoch_days: f64,

    /// Orbital period (T) in days. Used to compute mean motion n = 2π/T.
    pub orbital_period_days: f64,
}

/// High-level convenience wrapper: computes a body's heliocentric position in AU
/// from its orbital elements and the current simulation time.
///
/// NOTE: This function is not used by the active ECS path (orbital_physics_system
/// inlines the same logic). It is kept as a standalone utility.
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

    // Stable half-angle conversion from eccentric anomaly E to true anomaly ν.
    // Equivalent to: tan(ν/2) = sqrt((1+e)/(1-e)) * tan(E/2),
    // but uses atan2 to avoid singularities near ν = π.
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

/// Solves Kepler's transcendental equation  M = E − e·sin(E)  for E (eccentric anomaly)
/// given M (mean anomaly) and e (eccentricity) using Newton-Raphson iteration.
///
/// Convergence: for all solar-system eccentricities (e ≤ 0.21), 32 iterations at
/// tolerance 1e-12 gives sub-nanoradian accuracy well within floating-point limits.
///
/// The iteration is:  E_{n+1} = E_n − (E_n − e·sin(E_n) − M) / (1 − e·cos(E_n))
pub fn solve_keplers_equation(
    mean_anomaly: f64,
    eccentricity: f64,
    tolerance: f64,
    max_iterations: u32,
) -> f64 {
    // Clamp away degenerate parabolic/hyperbolic trajectories (e ≥ 1).
    let e = eccentricity.clamp(0.0, 0.999_999_999_999);
    // Floor the tolerance to avoid infinite loops at machine epsilon.
    let tolerance = tolerance.max(1.0e-16);

    if e <= CIRCULAR_ORBIT_EPSILON {
        return normalize_0_to_tau(mean_anomaly);
    }

    // Bring M to [-π, π] for better Newton convergence.
    // The solver works in the signed domain; result is mapped back to [0, 2π].
    let m = normalize_minus_pi_to_pi(mean_anomaly);

    // Initial guess strategy:
    // - e < 0.8 (most solar-system bodies): start at E₀ = M (good for near-circular)
    // - e ≥ 0.8 (highly eccentric): start near ±π to avoid slow convergence
    let mut eccentric_anomaly = if e < 0.8 {
        m
    } else if m >= 0.0 {
        PI
    } else {
        -PI
    };

    for _ in 0..max_iterations.max(1) {
        // f(E)  = E − e·sin(E) − M   (Kepler's equation rearranged to zero)
        // f'(E) = 1 − e·cos(E)       (derivative for Newton step)
        let f = eccentric_anomaly - (e * eccentric_anomaly.sin()) - m;
        let f_prime = 1.0 - (e * eccentric_anomaly.cos());

        // Guard against division by near-zero (shouldn't occur for e < 1).
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

/// Converts Keplerian orbital elements + true anomaly into a heliocentric
/// Cartesian position vector in AU (x = vernal equinox, z = north ecliptic pole).
///
/// Steps:
/// 1. Compute orbital radius:  r = a(1 − e²) / (1 + e·cos(ν))
/// 2. Rotate from the perifocal frame (orbit lies in XY) to the ecliptic
///    inertial frame via the three angles Ω (longitude of ascending node),
///    i (inclination), and ω + ν (argument of latitude).
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

    // Argument of latitude u = ω + ν — the angle from the ascending node
    // measured along the orbital plane to the current body position.
    let argument_of_latitude = argument_of_periapsis + true_anomaly;

    let (sin_omega, cos_omega) = longitude_ascending.sin_cos();
    let (sin_i, cos_i) = inclination.sin_cos();
    let (sin_u, cos_u) = argument_of_latitude.sin_cos();

    // Closed-form rotation from perifocal frame to inertial ecliptic frame.
    // Equivalent to the matrix product R_z(−Ω)·R_x(−i)·R_z(−ω) applied to
    // the perifocal position (r·cos ν, r·sin ν, 0).
    let x = radius * ((cos_omega * cos_u) - (sin_omega * sin_u * cos_i));
    let y = radius * ((sin_omega * cos_u) + (cos_omega * sin_u * cos_i));
    let z = radius * (sin_u * sin_i);

    DVec3::new(x, y, z)
}

/// Converts eccentric anomaly E to true anomaly ν using the numerically stable
/// half-angle form:  ν = 2·atan2(√(1+e)·sin(E/2),  √(1-e)·cos(E/2))
///
/// This avoids the singularity in the naive formula tan(ν/2) = √((1+e)/(1-e))·tan(E/2)
/// which is undefined when E = π (ν = π, apoapsis).
///
/// NOTE: Used by calculate_orbit_position (dead-code path). orbital_physics_system
/// inlines this conversion directly for efficiency.
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

/// Maps any angle to [0, 2π).
fn normalize_0_to_tau(angle: f64) -> f64 {
    angle.rem_euclid(TAU)
}

/// Maps any angle to (−π, π] — preferred domain for Newton-Raphson on Kepler's equation
/// because the derivative f'(E) = 1 − e·cos(E) is better conditioned there.
fn normalize_minus_pi_to_pi(angle: f64) -> f64 {
    let wrapped = normalize_0_to_tau(angle);
    if wrapped > PI {
        wrapped - TAU
    } else {
        wrapped
    }
}
