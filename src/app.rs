use crate::camera::{camera_follow_system, mouse_camera_control, CameraController};
use crate::error::LastError;
use crate::physics::kepler::Orbit;
use crate::physics::sync_physics_to_transforms;
use crate::physics::system::orbital_physics_system;
use crate::physics::system::{BodyState, PhysicsState};
use crate::render::sphere::create_sphere_mesh;
use crate::render::trail::{render_orbit_trails_system, trail_update_system, TrailConfig, TrailState};
use crate::render::{BodyMesh, SunLight};
use crate::ui::controls::ui_controls_system;
use crate::ui::labels::{body_labels_system, ShowBodyLabels};
use crate::ui::shortcuts::keyboard_shortcuts_system;
use bevy::log::error;
use bevy::prelude::*;
// bevy_egui 0.39 requires UI systems to run in EguiPrimaryContextPass (not Update).
// This schedule runs after begin_pass_system has initialized the egui context and fonts.
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

/// Resource to track the Sun entity for camera initialization
#[derive(Resource)]
struct SunEntity(Entity);

/// Spawns 14 sphere entities (1 Sun + 8 planets + 5 major moons) in the same order as `init_solar_system`.
///
/// IMPORTANT: The spawn order must match the order of bodies in PhysicsState because
/// `sync_physics_to_transforms` maps bodies to entities by entity ID lookup
/// (no index-based mapping).
///
/// Returns the spawned entity IDs in order (Sun first, then planets, then moons).
fn spawn_celestial_bodies(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Vec<Entity> {
    let mut entities = Vec::new();

    // Sun: higher mesh resolution (32×16) and emissive material so it glows
    // regardless of the scene's directional light position.
    let sun_mesh = create_sphere_mesh(1.0, 32, 16);
    let sun_material = StandardMaterial {
        base_color: Color::srgb(1.0, 0.95, 0.2),
        emissive: LinearRgba::rgb(1.0, 0.9, 0.2),
        ..default()
    };

    let sun_entity = commands
        .spawn((
            Mesh3d(meshes.add(sun_mesh)),
            MeshMaterial3d(materials.add(sun_material)),
            Transform::IDENTITY, // Scale set by sync from radius_km
            BodyMesh,
        ))
        .id();
    entities.push(sun_entity);

    // Planet colors chosen to visually distinguish them at a glance.
    // The tuple name is not stored on the entity — it only drives the color lookup here.
    let planet_colors = [
        ("Mercury", Color::srgb(0.7, 0.7, 0.7)),
        ("Venus", Color::srgb(0.9, 0.7, 0.4)),
        ("Earth", Color::srgb(0.2, 0.5, 0.9)),
        ("Mars", Color::srgb(0.9, 0.3, 0.1)),
        ("Jupiter", Color::srgb(0.8, 0.6, 0.4)),
        ("Saturn", Color::srgb(0.9, 0.8, 0.5)),
        ("Uranus", Color::srgb(0.4, 0.8, 0.9)),
        ("Neptune", Color::srgb(0.2, 0.4, 0.9)),
    ];

    for (_, color) in planet_colors.iter() {
        // Lower mesh resolution (16×8) for planets — they're small on screen
        // and high sector/stack counts are wasted detail at typical zoom levels.
        let mesh = create_sphere_mesh(1.0, 16, 8);
        let material = StandardMaterial {
            base_color: *color,
            ..default()
        };

        let entity = commands
            .spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(materials.add(material)),
                Transform::IDENTITY,
                BodyMesh,
            ))
            .id();
        entities.push(entity);
    }

    // Major moons: smaller spheres, distinct colors
    // Order: Moon (Earth), Io, Europa, Ganymede, Callisto (Jupiter)
    let moon_colors = [
        ("Moon", Color::srgb(0.75, 0.75, 0.8)),      // silvery grey
        ("Io", Color::srgb(0.9, 0.85, 0.5)),        // sulphur yellow
        ("Europa", Color::srgb(0.9, 0.9, 0.95)),   // ice white
        ("Ganymede", Color::srgb(0.6, 0.55, 0.5)), // grey-brown
        ("Callisto", Color::srgb(0.4, 0.35, 0.3)),  // dark grey
    ];

    for (_, color) in moon_colors.iter() {
        let mesh = create_sphere_mesh(1.0, 12, 6); // smaller resolution for moons
        let material = StandardMaterial {
            base_color: *color,
            ..default()
        };

        let entity = commands
            .spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(materials.add(material)),
                Transform::IDENTITY,
                BodyMesh,
            ))
            .id();
        entities.push(entity);
    }

    entities
}

/// Number of celestial bodies expected (1 Sun + 8 planets + 5 major moons).
const EXPECTED_BODY_COUNT: usize = 14;

/// Setup system run once at startup: spawns geometry, lighting, and the camera.
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut last_error: ResMut<LastError>,
) {
    let entities = spawn_celestial_bodies(&mut commands, &mut meshes, &mut materials);

    if entities.len() != EXPECTED_BODY_COUNT {
        let msg = format!(
            "Entity count mismatch (got {}, expected {}). Simulation may be corrupted.",
            entities.len(),
            EXPECTED_BODY_COUNT
        );
        error!("{msg}");
        last_error.set(msg, 0);
    }

    // Sun is always at index 0; use placeholder if none spawned (degraded but non-panicking)
    let sun_entity = entities.first().copied().unwrap_or(Entity::PLACEHOLDER);
    commands.insert_resource(SunEntity(sun_entity));

    let physics_state = init_solar_system(entities);
    commands.insert_resource(physics_state);

    // Single directional light simulating sunlight from roughly the Sun's direction.
    // NOTE: The light origin is at (10, 10, 10) — not at the Sun entity's position —
    // so inner planets receive light from a fixed direction rather than from behind them.
    commands.spawn((
        DirectionalLight {
            illuminance: 100000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        SunLight,
    ));

    commands.spawn(Camera3d::default());
}

/// Bevy plugin that wires all solar-system systems into the app schedule.
pub struct SolarSystemPlugin;

impl Plugin for SolarSystemPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PhysicsState>()
            .init_resource::<CameraController>()
            .init_resource::<TrailState>()
            .init_resource::<TrailConfig>()
            .init_resource::<ShowBodyLabels>()
            .init_resource::<LastError>()
            .add_systems(Startup, (setup, initialize_camera_focus.after(setup)))
            .add_systems(
                Update,
                (
                    keyboard_shortcuts_system,
                    orbital_physics_system,
                    sync_physics_to_transforms.after(orbital_physics_system),
                    trail_update_system.after(orbital_physics_system),
                    render_orbit_trails_system
                        .after(trail_update_system)
                        .run_if(resource_exists::<bevy::prelude::GizmoConfigStore>),
                    mouse_camera_control.before(camera_follow_system),
                    camera_follow_system.after(sync_physics_to_transforms),
                ),
            )
            // bevy_egui 0.39: UI systems must live in EguiPrimaryContextPass, which runs
            // after begin_pass_system has called egui::Context::begin_pass() and loaded fonts.
            // Running in Update would panic with "No fonts available" on the first frame.
            .add_systems(
                EguiPrimaryContextPass,
                (ui_controls_system, body_labels_system, error_display_system),
            );
    }
}

/// Displays user-facing error messages with a dismiss button.
fn error_display_system(
    mut contexts: EguiContexts,
    mut last_error: ResMut<LastError>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    if !last_error.has_error() {
        return;
    }

    egui::Window::new("⚠ Simulation Warning")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_TOP, [0.0, 20.0])
        .show(ctx, |ui| {
            ui.label(&last_error.message);
            if ui.button("Dismiss").clicked() {
                last_error.clear();
            }
        });
}

/// Initializes CameraController to focus on the Sun after SunEntity resource is available
fn initialize_camera_focus(mut commands: Commands, sun: Res<SunEntity>) {
    commands.insert_resource(CameraController {
        distance: 10.0,
        focus: sun.0,
        pitch: std::f64::consts::FRAC_PI_6, // 30° elevation
        yaw: 0.0,
    });
}

/// Constructs the initial PhysicsState with real orbital elements for all 8 planets.
///
/// All angular elements are in radians; distances in AU; periods in days.
/// Source: NASA planetary fact sheets / JPL Horizons epoch J2000.0.
///
/// Parameters: entities - Vec of 9 entity IDs in spawn order (Sun first, then 8 planets)
///
/// Returns: PhysicsState with populated body data
///
/// # Panics
/// Panics if `entities.len() < 9` (requires 1 Sun + 8 planets). Callers should
/// ensure the entity count matches before invoking.
pub fn init_solar_system(entities: Vec<Entity>) -> PhysicsState {
    if entities.len() < EXPECTED_BODY_COUNT {
        error!(
            "init_solar_system: expected at least {} entities, got {}. Filling missing with placeholder.",
            EXPECTED_BODY_COUNT,
            entities.len()
        );
    }

    // Index 0 — the Sun stays fixed at the origin (no orbit).
    // Orbital elements: JPL Approximate Positions / Planetary Satellite Mean Elements, J2000.0.
    // Radii: JPL Planetary Physical Parameters, IAU nominal solar radius.
    let sun_entity = entities.first().copied().unwrap_or(Entity::PLACEHOLDER);
    let mut bodies = vec![BodyState::new(sun_entity, "Sun", None).with_radius(695_700.0)];

    let get_entity_at = |i: usize| entities.get(i).copied().unwrap_or(Entity::PLACEHOLDER);

    const DEG: f64 = std::f64::consts::PI / 180.0;

    bodies.push(
        BodyState::new(
            get_entity_at(1),
            "Mercury",
            Some(Orbit {
                semi_major_axis_au: 0.38709927,
                eccentricity: 0.20563593,
                inclination_rad: 7.00497902 * DEG,
                longitude_ascending_rad: 48.33076593 * DEG,
                argument_of_periapsis_rad: 29.12703 * DEG,
                mean_anomaly_at_epoch_rad: 174.79253 * DEG,
                epoch_days: 0.0,
                orbital_period_days: 87.969,
            }),
        )
        .with_radius(2_440.53),
    );

    bodies.push(
        BodyState::new(
            get_entity_at(2),
            "Venus",
            Some(Orbit {
                semi_major_axis_au: 0.72333566,
                eccentricity: 0.00677672,
                inclination_rad: 3.39467605 * DEG,
                longitude_ascending_rad: 76.67984255 * DEG,
                argument_of_periapsis_rad: 54.92262 * DEG,
                mean_anomaly_at_epoch_rad: 50.37663 * DEG,
                epoch_days: 0.0,
                orbital_period_days: 224.701,
            }),
        )
        .with_radius(6_051.8),
    );

    bodies.push(
        BodyState::new(
            get_entity_at(3),
            "Earth",
            Some(Orbit {
                semi_major_axis_au: 1.00000261,
                eccentricity: 0.01671123,
                inclination_rad: -0.00001531 * DEG,
                longitude_ascending_rad: 0.0,
                argument_of_periapsis_rad: 102.93768 * DEG,
                mean_anomaly_at_epoch_rad: 357.52689 * DEG,
                epoch_days: 0.0,
                orbital_period_days: 365.256,
            }),
        )
        .with_radius(6_378.14),
    );

    bodies.push(
        BodyState::new(
            get_entity_at(4),
            "Mars",
            Some(Orbit {
                semi_major_axis_au: 1.52371034,
                eccentricity: 0.09339410,
                inclination_rad: 1.84969142 * DEG,
                longitude_ascending_rad: 49.55953891 * DEG,
                argument_of_periapsis_rad: 286.49683 * DEG,
                mean_anomaly_at_epoch_rad: 19.41248 * DEG,
                epoch_days: 0.0,
                orbital_period_days: 686.980,
            }),
        )
        .with_radius(3_396.19),
    );

    bodies.push(
        BodyState::new(
            get_entity_at(5),
            "Jupiter",
            Some(Orbit {
                semi_major_axis_au: 5.20288700,
                eccentricity: 0.04838624,
                inclination_rad: 1.30439695 * DEG,
                longitude_ascending_rad: 100.47390909 * DEG,
                argument_of_periapsis_rad: 274.25452 * DEG,
                mean_anomaly_at_epoch_rad: 19.66796 * DEG,
                epoch_days: 0.0,
                orbital_period_days: 4332.82,
            }),
        )
        .with_radius(71_492.0),
    );

    bodies.push(
        BodyState::new(
            get_entity_at(6),
            "Saturn",
            Some(Orbit {
                semi_major_axis_au: 9.53667594,
                eccentricity: 0.05386179,
                inclination_rad: 2.48599187 * DEG,
                longitude_ascending_rad: 113.66242448 * DEG,
                argument_of_periapsis_rad: 338.93605 * DEG,
                mean_anomaly_at_epoch_rad: 317.35537 * DEG,
                epoch_days: 0.0,
                orbital_period_days: 10759.22,
            }),
        )
        .with_radius(60_268.0),
    );

    bodies.push(
        BodyState::new(
            get_entity_at(7),
            "Uranus",
            Some(Orbit {
                semi_major_axis_au: 19.18916464,
                eccentricity: 0.04725744,
                inclination_rad: 0.77263783 * DEG,
                longitude_ascending_rad: 74.01692503 * DEG,
                argument_of_periapsis_rad: 96.93735 * DEG,
                mean_anomaly_at_epoch_rad: 142.28383 * DEG,
                epoch_days: 0.0,
                orbital_period_days: 30687.15,
            }),
        )
        .with_radius(25_559.0),
    );

    bodies.push(
        BodyState::new(
            get_entity_at(8),
            "Neptune",
            Some(Orbit {
                semi_major_axis_au: 30.06992276,
                eccentricity: 0.00859048,
                inclination_rad: 1.77004347 * DEG,
                longitude_ascending_rad: 131.78422574 * DEG,
                argument_of_periapsis_rad: 273.17949 * DEG,
                mean_anomaly_at_epoch_rad: 259.91521 * DEG,
                epoch_days: 0.0,
                orbital_period_days: 60190.03,
            }),
        )
        .with_radius(24_764.0),
    );

    // Major moons: geocentric (Moon) and jovicentric (Galilean moons)
    // JPL Planetary Satellite Mean Elements, DE405/LE405 (Moon), JUP365 (Galilean).
    let earth_entity = get_entity_at(3);
    let jupiter_entity = get_entity_at(5);

    bodies.push(BodyState::moon(
        get_entity_at(9),
        "Moon",
        Orbit {
            semi_major_axis_au: 384_400.0 / 149_597_870.7, // 0.0025696 AU
            eccentricity: 0.0554,
            inclination_rad: 5.16 * DEG,
            longitude_ascending_rad: 125.08 * DEG,
            argument_of_periapsis_rad: 318.15 * DEG,
            mean_anomaly_at_epoch_rad: 135.27 * DEG,
            epoch_days: 0.0,
            orbital_period_days: 27.322,
        },
        earth_entity,
        1_737.4,
    ));

    bodies.push(BodyState::moon(
        get_entity_at(10),
        "Io",
        Orbit {
            semi_major_axis_au: 421_800.0 / 149_597_870.7,
            eccentricity: 0.004,
            inclination_rad: 0.0,
            longitude_ascending_rad: 0.0,
            argument_of_periapsis_rad: 49.1 * DEG,
            mean_anomaly_at_epoch_rad: 330.9 * DEG,
            epoch_days: 0.0,
            orbital_period_days: 1.769,
        },
        jupiter_entity,
        1_821.49,
    ));

    bodies.push(BodyState::moon(
        get_entity_at(11),
        "Europa",
        Orbit {
            semi_major_axis_au: 671_100.0 / 149_597_870.7,
            eccentricity: 0.009,
            inclination_rad: 0.5 * DEG,
            longitude_ascending_rad: 184.0 * DEG,
            argument_of_periapsis_rad: 45.0 * DEG,
            mean_anomaly_at_epoch_rad: 345.4 * DEG,
            epoch_days: 0.0,
            orbital_period_days: 3.525,
        },
        jupiter_entity,
        1_560.80,
    ));

    bodies.push(BodyState::moon(
        get_entity_at(12),
        "Ganymede",
        Orbit {
            semi_major_axis_au: 1_070_400.0 / 149_597_870.7,
            eccentricity: 0.001,
            inclination_rad: 0.2 * DEG,
            longitude_ascending_rad: 58.5 * DEG,
            argument_of_periapsis_rad: 198.3 * DEG,
            mean_anomaly_at_epoch_rad: 324.8 * DEG,
            epoch_days: 0.0,
            orbital_period_days: 7.156,
        },
        jupiter_entity,
        2_631.20,
    ));

    bodies.push(BodyState::moon(
        get_entity_at(13),
        "Callisto",
        Orbit {
            semi_major_axis_au: 1_882_700.0 / 149_597_870.7,
            eccentricity: 0.007,
            inclination_rad: 0.3 * DEG,
            longitude_ascending_rad: 309.1 * DEG,
            argument_of_periapsis_rad: 43.8 * DEG,
            mean_anomaly_at_epoch_rad: 87.4 * DEG,
            epoch_days: 0.0,
            orbital_period_days: 16.690,
        },
        jupiter_entity,
        2_410.30,
    ));

    let mut physics_state = PhysicsState::default();
    for body in bodies {
        physics_state.bodies.insert(body.entity, body);
    }

    physics_state
}
