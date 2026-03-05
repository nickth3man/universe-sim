use bevy::prelude::*;

/// Controls the simulation time flow and speed.
///
/// This resource manages how time progresses in the solar system simulation.
/// It allows for pausing, speeding up, or slowing down the simulation.
///
/// # Fields
///
/// * `elapsed_time` - Total simulated time elapsed in seconds (f64 for precision)
/// * `time_scale` - Speed multiplier controlling simulation speed (0.0 to 10.0)
/// * `paused` - Whether the simulation is currently paused
///
/// # Example
///
/// ```
/// let time = SimulationTime::default();
/// assert_eq!(time.elapsed_time, 0.0);
/// assert_eq!(time.time_scale, 1.0);
/// assert_eq!(time.paused, false);
/// ```
#[derive(Resource, Debug)]
pub struct SimulationTime {
    /// Total simulated time elapsed in seconds.
    /// Uses f64 for high precision over long simulation periods.
    pub elapsed_time: f64,

    /// Speed multiplier for the simulation.
    /// - 0.0: Frozen (if not paused)
    /// - 1.0: Real-time speed
    /// - 10.0: 10x faster than real-time
    /// Range: 0.0 to 10.0
    pub time_scale: f32,

    /// Pause state of the simulation.
    /// When true, elapsed_time does not increase.
    pub paused: bool,
}

impl Default for SimulationTime {
    fn default() -> Self {
        Self {
            elapsed_time: 0.0,
            time_scale: 1.0,
            paused: false,
        }
    }
}

/// Controls visual scaling for the solar system simulation.
///
/// Real astronomical distances and sizes are impractical for visualization,
/// so this resource provides scaling factors to make the simulation viewable.
///
/// # Fields
///
/// * `distance_scale` - Multiplier for orbit distances (0.1 to 5.0)
/// * `size_scale` - Multiplier for planet and sun sizes (0.1 to 5.0)
///
/// # Example
///
/// ```
/// let scale = VisualScale::default();
/// assert_eq!(scale.distance_scale, 1.0);
/// assert_eq!(scale.size_scale, 1.0);
/// ```
#[derive(Resource, Debug)]
pub struct VisualScale {
    /// Multiplier for orbital distances.
    /// - 0.1: Compress distances to 10% of base values
    /// - 1.0: Use base distance values
    /// - 5.0: Expand distances to 5x base values
    /// Range: 0.1 to 5.0
    pub distance_scale: f32,

    /// Multiplier for celestial body sizes.
    /// - 0.1: Reduce sizes to 10% of base values
    /// - 1.0: Use base size values
    /// - 5.0: Expand sizes to 5x base values
    /// Range: 0.1 to 5.0
    pub size_scale: f32,
}

impl Default for VisualScale {
    fn default() -> Self {
        Self {
            distance_scale: 1.0,
            size_scale: 1.0,
        }
    }
}

/// Controls the camera position and orientation in the simulation.
///
/// The camera uses a spherical coordinate system centered on the origin
/// (typically the Sun's position). This allows for smooth orbital camera
/// movement around the solar system.
///
/// # Fields
///
/// * `distance` - Distance from origin/camera target (5.0 to 100.0)
/// * `azimuth` - Horizontal angle in radians (rotation around Y-axis)
/// * `elevation` - Vertical angle in radians (angle above/below horizon)
///
/// # Example
///
/// ```
/// let camera = CameraState::default();
/// assert_eq!(camera.distance, 30.0);
/// assert_eq!(camera.azimuth, 0.0);
/// assert!((camera.elevation - 0.3).abs() < 0.001);
/// ```
#[derive(Resource, Debug)]
pub struct CameraState {
    /// Distance from the camera target (origin) in world units.
    /// Controls zoom level.
    /// Range: 5.0 to 100.0
    pub distance: f32,

    /// Horizontal angle in radians.
    /// - 0.0: Looking along the +X axis
    /// - π/2: Looking along the +Z axis
    /// - π: Looking along the -X axis
    /// - 3π/2: Looking along the -Z axis
    pub azimuth: f32,

    /// Vertical angle in radians (elevation above the horizon).
    /// - 0.0: Looking horizontally
    /// - π/2: Looking straight down from above
    /// - -π/2: Looking straight up from below
    /// Default: 0.3 radians (~17 degrees) for a slight top-down view
    pub elevation: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            distance: 30.0,
            azimuth: 0.0,
            elevation: 0.3,
        }
    }
}
