use bevy::prelude::*;

/// Core simulation data for a planet entity in the solar system.
///
/// This component stores both visual metadata (such as `name` and `color`)
/// and orbital/physical parameters used by physics and rendering systems.
/// All distance and timing values are stored as `f64` to preserve precision
/// at solar-system scale.
///
/// # Example
///
/// ```ignore
/// use bevy::prelude::*;
/// use crate::components::Planet;
///
/// commands.spawn(Planet {
///     name: "Earth".to_string(),
///     color: Color::srgb(0.2, 0.4, 1.0),
///     radius: 1.0,
///     orbital_radius: 1.0,
///     orbital_period: 1.0,
///     current_angle: 0.0,
/// });
/// ```
#[derive(Component, Debug, Clone)]
pub struct Planet {
    /// Human-readable planet name for UI labels and debugging.
    pub name: String,
    /// Display color used by rendering systems.
    pub color: Color,
    /// Scaled physical radius of the planet in kilometers.
    pub radius: f64,
    /// Scaled orbital distance from the sun in astronomical units.
    pub orbital_radius: f64,
    /// Orbital period in Earth years.
    pub orbital_period: f64,
    /// Current orbital angle in radians.
    pub current_angle: f64,
}

/// Marker data for entities that visualize orbital paths.
///
/// Attach this component to ring/line mesh entities representing a planet's
/// orbit around the sun. The `radius` value should match the corresponding
/// planet's `orbital_radius` scale.
///
/// # Example
///
/// ```ignore
/// use crate::components::OrbitRing;
///
/// commands.spawn(OrbitRing { radius: 9.58 });
/// ```
#[derive(Component, Debug, Clone)]
pub struct OrbitRing {
    /// Scaled orbit radius used to draw the orbit ring.
    pub radius: f64,
}

/// Marker component identifying the sun entity.
///
/// This zero-sized component allows systems to query the sun directly without
/// relying on names or other mutable data.
///
/// # Example
///
/// ```ignore
/// use crate::components::Sun;
///
/// commands.spawn(Sun);
/// ```
#[derive(Component, Debug, Clone)]
pub struct Sun;

/// Marker component for Saturn's ring visual entity.
///
/// Use this marker to apply Saturn-specific rendering behavior (material,
/// orientation, scaling, or visibility toggles) independently from planets.
///
/// # Example
///
/// ```ignore
/// use crate::components::SaturnRing;
///
/// commands.spawn(SaturnRing);
/// ```
#[derive(Component, Debug, Clone)]
pub struct SaturnRing;
