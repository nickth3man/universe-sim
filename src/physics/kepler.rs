/// Pure Keplerian orbital mechanics — no ECS, no Bevy, just math.
///
/// Analytic (not numerical integration) approach: given a time t, each body's
/// position is computed directly via Kepler's equation rather than by integrating
/// velocity. This makes the simulation time-step independent and free of drift.
use bevy::log::warn;
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
///
/// # Validation
/// For safe use in physics systems, ensure:
/// - `semi_major_axis_au` > 0 and finite
/// - `eccentricity` in [0, 1) for elliptical orbits
/// - `orbital_period_days` > 0 and finite
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

impl Orbit {
    /// Returns `true` if orbital elements are valid for physics computation.
    /// Invalid elements (non-finite, negative, or e ≥ 1) may cause NaN or division by zero.
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.semi_major_axis_au.is_finite()
            && self.semi_major_axis_au > 0.0
            && self.eccentricity.is_finite()
            && (0.0..1.0).contains(&self.eccentricity)
            && self.orbital_period_days.is_finite()
            && self.orbital_period_days > 0.0
    }
}

/// High-level convenience wrapper: computes a body's heliocentric position in AU
/// from its orbital elements and the current simulation time.
///
/// NOTE: This function is not used by the active ECS path (orbital_physics_system
/// inlines the same logic). It is kept as a standalone utility.
#[allow(dead_code)]
pub fn get_orbit_position(orbit: &Orbit, time_days: f64) -> DVec3 {
    if !orbit.orbital_period_days.is_finite() || orbit.orbital_period_days <= 0.0 {
        warn!(
            "Invalid orbital_period_days ({}) in get_orbit_position, using 1.0",
            orbit.orbital_period_days
        );
        return DVec3::ZERO;
    }
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

    let eccentric_anomaly = solve_kepler_equation(mean_anomaly, orbit.eccentricity, 1.0e-12, 32);

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
pub fn solve_kepler_equation(
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
    if !semi_major_axis.is_finite() || semi_major_axis < 0.0 {
        warn!(
            "Invalid semi_major_axis ({}) in orbital_to_cartesian, using 0.0",
            semi_major_axis
        );
        return DVec3::ZERO;
    }
    // Clamp eccentricity to avoid parabolic/hyperbolic (e >= 1) which causes
    // division by zero when 1 + e*cos(ν) = 0 at apoapsis (ν = π).
    let e = eccentricity.clamp(0.0, 0.999_999_999_999);

    // Radius for an ellipse as a function of true anomaly:
    // r = a(1 - e^2) / (1 + e*cos(ν))
    // For circular orbit, this reduces to r = a.
    // Guard: denominator 1 + e*cos(ν) can approach 0 for e≈1 at ν≈π; clamp to avoid inf.
    let radius = if e <= CIRCULAR_ORBIT_EPSILON {
        semi_major_axis
    } else {
        let denominator = 1.0 + (e * true_anomaly.cos());
        if denominator <= 1.0e-10 {
            warn!(
                "orbital_to_cartesian: near-singular denominator (1+e*cos(ν)={}), using apoapsis radius",
                denominator
            );
            semi_major_axis * (1.0 + e) // apoapsis radius
        } else {
            semi_major_axis * (1.0 - (e * e)) / denominator
        }
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
/// NOTE: Used by get_orbit_position (dead-code path). orbital_physics_system
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

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI, TAU};

    // =========================================================================
    // Tests for solve_kepler_equation
    // =========================================================================

    /// Test 1: Circular orbit (e = 0) - eccentric anomaly equals mean anomaly
    /// For a circular orbit, Kepler's equation reduces to E = M
    #[test]
    fn test_kepler_circular_orbit_mean_anomaly_equals_eccentric_anomaly() {
        let eccentricity = 0.0;

        // Test at various mean anomalies
        for &m in &[0.0, FRAC_PI_4, FRAC_PI_2, PI, TAU - 0.1] {
            let e_anomaly = solve_kepler_equation(m, eccentricity, 1e-12, 32);
            assert!(
                (e_anomaly - normalize_0_to_tau(m)).abs() < 1e-12,
                "For circular orbit at M={}, E should equal M (got E={})",
                m,
                e_anomaly
            );
        }
    }

    /// Test 2: Known cases e=0.2: M=0 => E≈0, M=π => E≈π
    #[test]
    fn test_kepler_known_cases_e02() {
        let eccentricity = 0.2;
        let tolerance = 1e-12;

        // M = 0 => E ≈ 0 (perihelion)
        let e_at_zero = solve_kepler_equation(0.0, eccentricity, tolerance, 32);
        assert!(
            e_at_zero.abs() < 1e-10,
            "e=0.2, M=0: E should be ≈0 (got {})",
            e_at_zero
        );

        // M = π => E ≈ π (aphelion)
        let e_at_pi = solve_kepler_equation(PI, eccentricity, tolerance, 32);
        assert!(
            (e_at_pi - PI).abs() < 1e-10,
            "e=0.2, M=π: E should be ≈π (got {})",
            e_at_pi
        );

        // Verify both satisfy Kepler's equation M = E - e*sin(E)
        let m_computed_zero = e_at_zero - eccentricity * e_at_zero.sin();
        let m_computed_pi = e_at_pi - eccentricity * e_at_pi.sin();
        assert!((m_computed_zero - 0.0).abs() < tolerance);
        assert!((m_computed_pi - PI).abs() < tolerance);
    }

    /// Test 3: Elliptical orbit with moderate eccentricity (e = 0.5)
    /// Verifies Newton-Raphson convergence for typical elliptical orbit
    #[test]
    fn test_kepler_elliptical_moderate_eccentricity() {
        let eccentricity = 0.5;
        let tolerance = 1e-12;

        // At M = 0, E should be 0 (perihelion)
        let e_anomaly = solve_kepler_equation(0.0, eccentricity, tolerance, 32);
        assert!(
            e_anomaly.abs() < 1e-10,
            "At M=0, E should be 0 (got {})",
            e_anomaly
        );

        // At M = PI, verify we get a reasonable eccentric anomaly
        let e_anomaly = solve_kepler_equation(PI, eccentricity, tolerance, 32);
        // For e=0.5 at M=PI, E should be > PI (aphelion side)
        // Verify the solution satisfies Kepler's equation
        let m_computed = e_anomaly - eccentricity * e_anomaly.sin();
        assert!(
            (m_computed - PI).abs() < tolerance,
            "Solution should satisfy M = E - e*sin(E)"
        );
    }

    /// Test 4: Edge case - mean anomaly at 0 (perihelion)
    #[test]
    fn test_kepler_edge_case_mean_anomaly_zero() {
        let e_anomaly = solve_kepler_equation(0.0, 0.3, 1e-12, 32);
        assert!(
            e_anomaly.abs() < 1e-10,
            "At M=0, E should be 0 for any eccentricity (got {})",
            e_anomaly
        );
    }

    /// Test 5: Edge case - mean anomaly at PI (aphelion direction)
    #[test]
    fn test_kepler_edge_case_mean_anomaly_pi() {
        let eccentricity = 0.3;
        let e_anomaly = solve_kepler_equation(PI, eccentricity, 1e-12, 32);

        // Verify Kepler's equation is satisfied: M = E - e*sin(E)
        let m_computed = e_anomaly - eccentricity * e_anomaly.sin();
        assert!(
            (m_computed - PI).abs() < 1e-10,
            "Solution must satisfy Kepler's equation (M_computed={}, expected π)",
            m_computed
        );
    }

    /// Test 6: Edge case - mean anomaly at 2π (full orbit, equivalent to 0)
    #[test]
    fn test_kepler_edge_case_mean_anomaly_tau() {
        let eccentricity = 0.4;
        let e_anomaly = solve_kepler_equation(TAU, eccentricity, 1e-12, 32);

        // Result should be normalized to [0, 2π), essentially 0
        assert!(
            e_anomaly.abs() < 1e-10 || (TAU - e_anomaly).abs() < 1e-10,
            "At M=2π, E should be 0 (or 2π) (got {})",
            e_anomaly
        );
    }

    /// Test 7: High eccentricity (e = 0.9) - verify convergence within tolerance
    /// This tests the special initial guess handling for e >= 0.8
    #[test]
    fn test_kepler_high_eccentricity_convergence() {
        let eccentricity = 0.9;
        let tolerance = 1e-12;

        // Test at multiple mean anomalies
        for &m in &[0.1, FRAC_PI_2, PI, 3.0 * FRAC_PI_2] {
            let e_anomaly = solve_kepler_equation(m, eccentricity, tolerance, 32);

            // Verify solution satisfies Kepler's equation
            let m_computed = e_anomaly - eccentricity * e_anomaly.sin();
            assert!(
                (normalize_minus_pi_to_pi(m_computed) - normalize_minus_pi_to_pi(m)).abs() < 1e-10,
                "High eccentricity solution must satisfy Kepler's equation (M={}, M_computed={})",
                m,
                m_computed
            );
        }
    }

    /// Test 8: Near-circular orbit (e just above CIRCULAR_ORBIT_EPSILON)
    #[test]
    fn test_kepler_near_circular_orbit() {
        let eccentricity = 1.5e-10; // Just above the epsilon threshold
        let mean_anomaly = FRAC_PI_2;

        let e_anomaly = solve_kepler_equation(mean_anomaly, eccentricity, 1e-12, 32);

        // Should converge quickly and be very close to mean anomaly
        assert!(
            (e_anomaly - mean_anomaly).abs() < 1e-8,
            "Near-circular orbit should have E ≈ M"
        );
    }

    /// Test 9: Verify solution is always in [0, 2π) range
    #[test]
    fn test_kepler_output_range_normalization() {
        let eccentricity = 0.5;

        // Test with negative mean anomaly
        let e_anomaly = solve_kepler_equation(-PI, eccentricity, 1e-12, 32);
        assert!(
            e_anomaly >= 0.0 && e_anomaly < TAU,
            "Output should be in [0, 2π), got {}",
            e_anomaly
        );

        // Test with large mean anomaly
        let e_anomaly = solve_kepler_equation(10.0 * TAU + 1.0, eccentricity, 1e-12, 32);
        assert!(
            e_anomaly >= 0.0 && e_anomaly < TAU,
            "Output should be in [0, 2π), got {}",
            e_anomaly
        );
    }

    // =========================================================================
    // Tests for orbital_to_cartesian
    // =========================================================================

    /// Test 9: Circular equatorial (e=0, i=0): position at (a,0,0) for ν=0, (-a,0,0) for ν=π
    #[test]
    fn test_orbital_to_cartesian_circular_equatorial_cardinal() {
        let a = 1.0; // 1 AU
        let e = 0.0;
        let i = 0.0;
        let omega = 0.0;
        let omega_peri = 0.0;

        // ν=0 (perihelion for circular): position at (a, 0, 0)
        let pos_nu0 = orbital_to_cartesian(a, e, i, omega, omega_peri, 0.0);
        assert!(
            (pos_nu0.x - a).abs() < 1e-12 && pos_nu0.y.abs() < 1e-12 && pos_nu0.z.abs() < 1e-12,
            "Circular equatorial ν=0: expected ({},0,0), got ({},{},{})",
            a,
            pos_nu0.x,
            pos_nu0.y,
            pos_nu0.z
        );

        // ν=π (aphelion for circular): position at (-a, 0, 0)
        let pos_nu_pi = orbital_to_cartesian(a, e, i, omega, omega_peri, PI);
        assert!(
            (pos_nu_pi.x + a).abs() < 1e-12 && pos_nu_pi.y.abs() < 1e-12 && pos_nu_pi.z.abs() < 1e-12,
            "Circular equatorial ν=π: expected (-{},0,0), got ({},{},{})",
            a,
            pos_nu_pi.x,
            pos_nu_pi.y,
            pos_nu_pi.z
        );
    }

    /// Test 10: Circular orbit in x-y plane (zero inclination)
    /// For i=0, the orbit should lie entirely in the ecliptic (z=0)
    #[test]
    fn test_orbital_to_cartesian_zero_inclination_xy_plane() {
        let semi_major_axis = 1.0; // 1 AU
        let eccentricity = 0.0; // Circular
        let inclination = 0.0; // Zero inclination
        let longitude_ascending = 0.0;
        let argument_of_periapsis = 0.0;

        // Test at multiple true anomalies
        for &true_anomaly in &[0.0, FRAC_PI_2, PI, 3.0 * FRAC_PI_2] {
            let pos = orbital_to_cartesian(
                semi_major_axis,
                eccentricity,
                inclination,
                longitude_ascending,
                argument_of_periapsis,
                true_anomaly,
            );

            assert!(
                pos.z.abs() < 1e-12,
                "Zero inclination orbit should have z=0 at ν={} (got z={})",
                true_anomaly,
                pos.z
            );

            // For circular orbit, radius should be constant
            let radius = (pos.x * pos.x + pos.y * pos.y + pos.z * pos.z).sqrt();
            assert!(
                (radius - semi_major_axis).abs() < 1e-12,
                "Circular orbit radius should be constant (got {})",
                radius
            );
        }
    }

    /// Test 10: Earth-like orbit at epoch
    /// Earth: a ≈ 1 AU, e ≈ 0.0167, i ≈ 0°
    #[test]
    fn test_orbital_to_cartesian_earth_like_orbit() {
        let semi_major_axis = 1.0; // 1 AU
        let eccentricity = 0.0167; // Earth's eccentricity
        let inclination = 0.0;
        let longitude_ascending = 0.0;
        let argument_of_periapsis = 0.0;

        // At true anomaly = 0 (perihelion), position should be at (a(1-e), 0, 0)
        let pos_perihelion = orbital_to_cartesian(
            semi_major_axis,
            eccentricity,
            inclination,
            longitude_ascending,
            argument_of_periapsis,
            0.0,
        );

        let expected_r_perihelion = semi_major_axis * (1.0 - eccentricity);
        assert!(
            (pos_perihelion.x - expected_r_perihelion).abs() < 1e-10,
            "Perihelion x-coordinate should be a(1-e) (expected {}, got {})",
            expected_r_perihelion,
            pos_perihelion.x
        );
        assert!(
            pos_perihelion.y.abs() < 1e-10,
            "Perihelion y-coordinate should be 0 (got {})",
            pos_perihelion.y
        );

        // At true anomaly = π (aphelion), position should be at (-a(1+e), 0, 0)
        let pos_aphelion = orbital_to_cartesian(
            semi_major_axis,
            eccentricity,
            inclination,
            longitude_ascending,
            argument_of_periapsis,
            PI,
        );

        let expected_r_aphelion = semi_major_axis * (1.0 + eccentricity);
        assert!(
            (pos_aphelion.x + expected_r_aphelion).abs() < 1e-10,
            "Aphelion x-coordinate should be -a(1+e) (expected {}, got {})",
            -expected_r_aphelion,
            pos_aphelion.x
        );
    }

    /// Test 11: Elliptical orbit - verify r = a(1-e²)/(1+e*cos(ν)) at arbitrary ν
    #[test]
    fn test_orbital_to_cartesian_radius_formula() {
        let a = 2.0;
        let e = 0.3;
        let i = 0.0;
        let omega = 0.0;
        let omega_peri = 0.0;

        for &nu in &[0.0, FRAC_PI_4, FRAC_PI_2, PI, 3.0 * FRAC_PI_2] {
            let pos = orbital_to_cartesian(a, e, i, omega, omega_peri, nu);
            let r_actual = pos.length();
            let r_expected = a * (1.0 - e * e) / (1.0 + e * nu.cos());
            assert!(
                (r_actual - r_expected).abs() < 1e-10,
                "At ν={}: r should be a(1-e²)/(1+e*cos(ν)) = {} (got {})",
                nu,
                r_expected,
                r_actual
            );
        }
    }

    /// Test 12: Elliptical orbit radius variation with true anomaly
    #[test]
    fn test_orbital_to_cartesian_elliptical_radius_variation() {
        let semi_major_axis = 2.0; // 2 AU
        let eccentricity = 0.5;
        let inclination = 0.0;
        let longitude_ascending = 0.0;
        let argument_of_periapsis = 0.0;

        // Perihelion (ν = 0): r = a(1-e)
        let pos_peri = orbital_to_cartesian(
            semi_major_axis,
            eccentricity,
            inclination,
            longitude_ascending,
            argument_of_periapsis,
            0.0,
        );
        let r_peri = pos_peri.length();
        let expected_r_peri = semi_major_axis * (1.0 - eccentricity);
        assert!(
            (r_peri - expected_r_peri).abs() < 1e-10,
            "Perihelion radius should be a(1-e)"
        );

        // Aphelion (ν = π): r = a(1+e)
        let pos_aph = orbital_to_cartesian(
            semi_major_axis,
            eccentricity,
            inclination,
            longitude_ascending,
            argument_of_periapsis,
            PI,
        );
        let r_aph = pos_aph.length();
        let expected_r_aph = semi_major_axis * (1.0 + eccentricity);
        assert!(
            (r_aph - expected_r_aph).abs() < 1e-10,
            "Aphelion radius should be a(1+e)"
        );
    }

    /// Test 13: Inclined orbit produces non-zero z component
    #[test]
    fn test_orbital_to_cartesian_inclined_orbit_z_component() {
        let semi_major_axis = 1.0;
        let eccentricity = 0.0;
        let inclination = FRAC_PI_4; // 45 degrees
        let longitude_ascending = 0.0;
        let argument_of_periapsis = 0.0;

        // At ν = π/2, z should be non-zero for inclined orbit
        let pos = orbital_to_cartesian(
            semi_major_axis,
            eccentricity,
            inclination,
            longitude_ascending,
            argument_of_periapsis,
            FRAC_PI_2,
        );

        assert!(
            pos.z.abs() > 0.1,
            "Inclined orbit should have non-zero z at ν=π/2 (got z={})",
            pos.z
        );

        // Expected z = r * sin(u) * sin(i) where u = ω + ν
        let expected_z = semi_major_axis * (FRAC_PI_2).sin() * inclination.sin();
        assert!(
            (pos.z - expected_z).abs() < 1e-10,
            "Z component should match formula (expected {}, got {})",
            expected_z,
            pos.z
        );
    }

    // =========================================================================
    // Tests for get_orbit_position
    // =========================================================================

    /// Test 14: Earth-like orbit position at epoch
    #[test]
    fn test_get_orbit_position_earth_at_epoch() {
        let orbit = Orbit {
            semi_major_axis_au: 1.0,
            eccentricity: 0.0167,
            inclination_rad: 0.0,
            longitude_ascending_rad: 0.0,
            argument_of_periapsis_rad: 0.0,
            mean_anomaly_at_epoch_rad: 0.0, // At perihelion
            epoch_days: 0.0,
            orbital_period_days: 365.25,
        };

        // At epoch (time = 0), M = 0, so we're at perihelion
        let pos = get_orbit_position(&orbit, 0.0);

        // Should be near perihelion position
        let expected_r = orbit.semi_major_axis_au * (1.0 - orbit.eccentricity);
        let actual_r = pos.length();

        assert!(
            (actual_r - expected_r).abs() < 0.01,
            "Earth at epoch should be near perihelion (expected r={}, got {})",
            expected_r,
            actual_r
        );
    }

    /// Test 15: Circular orbit position at quarter period
    #[test]
    fn test_get_orbit_position_circular_quarter_orbit() {
        let orbit = Orbit {
            semi_major_axis_au: 1.0,
            eccentricity: 0.0, // Circular
            inclination_rad: 0.0,
            longitude_ascending_rad: 0.0,
            argument_of_periapsis_rad: 0.0,
            mean_anomaly_at_epoch_rad: 0.0,
            epoch_days: 0.0,
            orbital_period_days: 365.25,
        };

        // After quarter period, should be at 90 degrees
        let quarter_period = orbit.orbital_period_days / 4.0;
        let pos = get_orbit_position(&orbit, quarter_period);

        // For circular orbit in x-y plane: x≈0, y≈a
        assert!(
            pos.x.abs() < 0.01,
            "After quarter period, x should be ~0 (got {})",
            pos.x
        );
        assert!(
            (pos.y - 1.0).abs() < 0.01,
            "After quarter period, y should be ~1 (got {})",
            pos.y
        );
    }

    /// Test 16: Invalid orbital period returns zero
    #[test]
    fn test_get_orbit_position_invalid_period() {
        let orbit = Orbit {
            semi_major_axis_au: 1.0,
            eccentricity: 0.1,
            inclination_rad: 0.0,
            longitude_ascending_rad: 0.0,
            argument_of_periapsis_rad: 0.0,
            mean_anomaly_at_epoch_rad: 0.0,
            epoch_days: 0.0,
            orbital_period_days: -1.0, // Invalid
        };

        let pos = get_orbit_position(&orbit, 0.0);
        assert_eq!(
            pos,
            DVec3::ZERO,
            "Invalid orbital period should return zero position"
        );
    }

    // =========================================================================
    // Tests for eccentric_to_true_anomaly
    // =========================================================================

    /// Test 17: Eccentric anomaly to true anomaly conversion
    #[test]
    fn test_eccentric_to_true_anomaly_circular() {
        let eccentricity = 0.0;

        // For circular orbit, E = ν
        for &e_anomaly in &[0.0, FRAC_PI_4, FRAC_PI_2, PI, 3.0 * FRAC_PI_2] {
            let t_anomaly = eccentric_to_true_anomaly(e_anomaly, eccentricity);
            assert!(
                (t_anomaly - e_anomaly).abs() < 1e-10,
                "For circular orbit, true anomaly should equal eccentric anomaly"
            );
        }
    }

    /// Test 18: At E=0 and E=π, verify special cases
    #[test]
    fn test_eccentric_to_true_anomaly_special_cases() {
        let eccentricity = 0.5;

        // At E=0, ν should be 0
        let t_anomaly = eccentric_to_true_anomaly(0.0, eccentricity);
        assert!(t_anomaly.abs() < 1e-10, "At E=0, ν should be 0");

        // At E=π, ν should be π (apoapsis)
        let t_anomaly = eccentric_to_true_anomaly(PI, eccentricity);
        assert!(
            (t_anomaly - PI).abs() < 1e-10,
            "At E=π, ν should be π (got {})",
            t_anomaly
        );
    }

    // =========================================================================
    // Tests for normalization functions
    // =========================================================================

    /// Test 19: normalize_0_to_tau wraps angles correctly
    #[test]
    fn test_normalize_0_to_tau() {
        // Already in range
        assert!((normalize_0_to_tau(0.0) - 0.0).abs() < 1e-15);
        assert!((normalize_0_to_tau(PI) - PI).abs() < 1e-15);
        assert!((normalize_0_to_tau(TAU - 0.001) - (TAU - 0.001)).abs() < 1e-15);

        // Negative angles
        assert!((normalize_0_to_tau(-PI) - PI).abs() < 1e-15);
        assert!((normalize_0_to_tau(-TAU) - 0.0).abs() < 1e-15);

        // Multiple rotations
        assert!((normalize_0_to_tau(3.0 * TAU + 1.0) - 1.0).abs() < 1e-15);
    }

    /// Test 20: normalize_minus_pi_to_pi wraps angles correctly
    #[test]
    fn test_normalize_minus_pi_to_pi() {
        // Already in range
        assert!((normalize_minus_pi_to_pi(0.0) - 0.0).abs() < 1e-10);
        assert!((normalize_minus_pi_to_pi(PI) - PI).abs() < 1e-10);
        // Note: -PI wraps to PI (range is (-π, π], not [-π, π))
        assert!((normalize_minus_pi_to_pi(-PI) - PI).abs() < 1e-10);

        // Angles > π
        assert!((normalize_minus_pi_to_pi(3.0 * FRAC_PI_2) - (-FRAC_PI_2)).abs() < 1e-10);
        assert!((normalize_minus_pi_to_pi(2.0 * PI) - 0.0).abs() < 1e-10);
    }

    // =========================================================================
    // Integration tests
    // =========================================================================

    /// Test 21: Round-trip orbital position consistency
    /// Calculate position at epoch and after one full orbit - should be same
    #[test]
    fn test_orbital_position_period_consistency() {
        let orbit = Orbit {
            semi_major_axis_au: 1.5,
            eccentricity: 0.3,
            inclination_rad: 0.1,
            longitude_ascending_rad: 0.5,
            argument_of_periapsis_rad: 0.2,
            mean_anomaly_at_epoch_rad: 0.7,
            epoch_days: 0.0,
            orbital_period_days: 500.0,
        };

        let pos_epoch = get_orbit_position(&orbit, 0.0);
        let pos_full_orbit = get_orbit_position(&orbit, orbit.orbital_period_days);

        // After one full orbit, position should be the same
        let diff = (pos_epoch - pos_full_orbit).length();
        assert!(
            diff < 1e-10,
            "Position after one orbit should match epoch position (diff={})",
            diff
        );
    }
}
