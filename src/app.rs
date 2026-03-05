use crate::camera::{camera_follow_system, CameraController};
use crate::physics::kepler::Orbit;
use crate::physics::system::orbital_physics_system;
use crate::physics::system::{AppState, BodyState};
use crate::render::sphere::{calculate_visual_radius, create_sphere_mesh};
use crate::render::{BodyMesh, SunLight};
use crate::ui::controls::ui_controls_system;
use bevy::prelude::*;
// bevy_egui 0.39 requires UI systems to run in EguiPrimaryContextPass (not Update).
// This schedule runs after begin_pass_system has initialized the egui context and fonts.
use bevy_egui::EguiPrimaryContextPass;

/// Spawns 9 sphere entities (1 Sun + 8 planets) in the same order as `init_solar_system`.
///
/// IMPORTANT: The spawn order must match the order of `bodies` in `AppState` because
/// `update_body_transforms` maps bodies to entities by parallel index (no name lookup).
/// If entities and bodies fall out of sync, all planet positions will be corrupted silently.
fn spawn_celestial_bodies(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Sun: higher mesh resolution (32×16) and emissive material so it glows
    // regardless of the scene's directional light position.
    let sun_mesh = create_sphere_mesh(1.0, 32, 16);
    let sun_material = StandardMaterial {
        base_color: Color::srgb(1.0, 0.95, 0.2),
        emissive: LinearRgba::rgb(1.0, 0.9, 0.2),
        ..default()
    };

    commands.spawn((
        Mesh3d(meshes.add(sun_mesh)),
        MeshMaterial3d(materials.add(sun_material)),
        Transform::from_scale(Vec3::splat(0.5)),
        BodyMesh,
    ));

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

        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(material)),
            Transform::IDENTITY,
            BodyMesh,
        ));
    }
}

/// Bevy system: syncs every BodyMesh entity's Transform from the physics state.
///
/// Position scale: physics positions are in AU; world units use 1 AU = 10.0 world units.
/// This keeps inner planets (0.4–1.5 AU) at a comfortable distance from the origin
/// while keeping Neptune (~30 AU) reachable at camera distances the egui slider allows.
fn update_body_transforms(state: Res<AppState>, mut query: Query<&mut Transform, With<BodyMesh>>) {
    let mut iter = query.iter_mut();

    for (i, body) in state.bodies.iter().enumerate() {
        if let Some(mut transform) = iter.next() {
            let visual_scale = if i == 0 {
                // Sun is index 0 — fixed radius of 0.5 world units (not physics-accurate).
                0.5
            } else {
                // NOTE: All planets use 6000.0 km regardless of their actual radius.
                // Earth's real radius is 6371 km, but Jupiter's is ~11x larger.
                // This makes all planets appear the same size. Passing each body's
                // true radius_km here would give size-accurate visuals.
                calculate_visual_radius(6000.0) * 2.0
            };

            // Convert AU → world units. Factor 10.0 matches camera.rs and ui/controls.rs.
            transform.translation = Vec3::new(
                body.position.x as f32 * 10.0,
                body.position.y as f32 * 10.0,
                body.position.z as f32 * 10.0,
            );
            transform.scale = Vec3::splat(visual_scale);
        }
    }
}

/// Setup system run once at startup: spawns geometry, lighting, and the camera.
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_celestial_bodies(&mut commands, &mut meshes, &mut materials);

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
        // init_resource is a no-op here because main.rs calls insert_resource(init_solar_system())
        // before add_plugins(SolarSystemPlugin). Bevy skips init_resource when the resource
        // already exists, so the populated solar system data is preserved.
        app.init_resource::<AppState>()
            .init_resource::<CameraController>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    // Declaration order is the implicit execution order here.
                    // Physics must run before transforms, which must run before camera.
                    orbital_physics_system,   // 1. Advance time; update body positions (AU)
                    update_body_transforms,   // 2. Copy AU positions → Bevy Transforms
                    camera_follow_system,     // 3. Point camera at focused body
                ),
            )
            // bevy_egui 0.39: UI systems must live in EguiPrimaryContextPass, which runs
            // after begin_pass_system has called egui::Context::begin_pass() and loaded fonts.
            // Running in Update would panic with "No fonts available" on the first frame.
            .add_systems(EguiPrimaryContextPass, ui_controls_system);
    }
}

/// Constructs the initial AppState with real orbital elements for all 8 planets.
///
/// All angular elements are in radians; distances in AU; periods in days.
/// Source: NASA planetary fact sheets / JPL Horizons epoch J2000.0.
pub fn init_solar_system(mut commands: Commands) -> (Vec<BodyState>, Vec<Entity>) {
    // Index 0 — the Sun stays fixed at the origin (no orbit).
    let mut bodies = vec![BodyState::new("Sun", None)];

    bodies.push(BodyState::new(
        "Mercury",
        Some(Orbit {
            semi_major_axis_au: 0.387,
            eccentricity: 0.2056,       // Highest eccentricity of the 8 planets
            inclination_rad: 0.1223,
            longitude_ascending_rad: 0.8435,
            argument_of_periapsis_rad: 1.3519,
            mean_anomaly_at_epoch_rad: 0.0,
            epoch_days: 0.0,
            orbital_period_days: 88.0,
        }),
    ));

    bodies.push(BodyState::new(
        "Venus",
        Some(Orbit {
            semi_major_axis_au: 0.723,
            eccentricity: 0.0067,       // Nearly circular
            inclination_rad: 0.0592,
            longitude_ascending_rad: 1.3382,
            argument_of_periapsis_rad: 2.2957,
            mean_anomaly_at_epoch_rad: 0.0,
            epoch_days: 0.0,
            orbital_period_days: 224.7,
        }),
    ));

    bodies.push(BodyState::new(
        "Earth",
        Some(Orbit {
            semi_major_axis_au: 1.0,    // Defines the AU
            eccentricity: 0.0167,
            inclination_rad: 0.0,       // Reference plane — ecliptic
            longitude_ascending_rad: 0.0,
            argument_of_periapsis_rad: 1.7968,
            mean_anomaly_at_epoch_rad: 0.0,
            epoch_days: 0.0,
            orbital_period_days: 365.25,
        }),
    ));

    bodies.push(BodyState::new(
        "Mars",
        Some(Orbit {
            semi_major_axis_au: 1.524,
            eccentricity: 0.0934,
            inclination_rad: 0.0323,
            longitude_ascending_rad: 0.8653,
            argument_of_periapsis_rad: 5.0000,
            mean_anomaly_at_epoch_rad: 0.0,
            epoch_days: 0.0,
            orbital_period_days: 687.0,
        }),
    ));

    bodies.push(BodyState::new(
        "Jupiter",
        Some(Orbit {
            semi_major_axis_au: 5.204,
            eccentricity: 0.0489,
            inclination_rad: 0.0228,
            longitude_ascending_rad: 1.7541,
            argument_of_periapsis_rad: 0.2575,
            mean_anomaly_at_epoch_rad: 0.0,
            epoch_days: 0.0,
            orbital_period_days: 4333.0,  // ~11.9 Earth years
        }),
    ));

    bodies.push(BodyState::new(
        "Saturn",
        Some(Orbit {
            semi_major_axis_au: 9.582,
            eccentricity: 0.0565,
            inclination_rad: 0.0434,
            longitude_ascending_rad: 1.9834,
            argument_of_periapsis_rad: 1.6132,
            mean_anomaly_at_epoch_rad: 0.0,
            epoch_days: 0.0,
            orbital_period_days: 10759.0, // ~29.5 Earth years
        }),
    ));

    bodies.push(BodyState::new(
        "Uranus",
        Some(Orbit {
            semi_major_axis_au: 19.20,
            eccentricity: 0.0457,
            inclination_rad: 0.0134,
            longitude_ascending_rad: 1.2910,
            argument_of_periapsis_rad: 2.9835,
            mean_anomaly_at_epoch_rad: 0.0,
            epoch_days: 0.0,
            orbital_period_days: 30687.0, // ~84 Earth years
        }),
    ));

    bodies.push(BodyState::new(
        "Neptune",
        Some(Orbit {
            semi_major_axis_au: 30.05,
            eccentricity: 0.0113,       // Nearly circular — lowest eccentricity
            inclination_rad: 0.0309,
            longitude_ascending_rad: 2.3001,
            argument_of_periapsis_rad: 0.7840,
            mean_anomaly_at_epoch_rad: 0.0,
            epoch_days: 0.0,
            orbital_period_days: 60190.0, // ~165 Earth years
        }),
    ));

    // For now, return empty vec for entities - will fill in next task
    (bodies, vec![])
}
