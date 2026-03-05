use crate::camera::{camera_follow_system, CameraController};
use crate::physics::kepler::Orbit;
use crate::physics::system::orbital_physics_system;
use crate::physics::system::{AppState, BodyState};
use crate::render::sphere::{calculate_visual_radius, create_sphere_mesh};
use crate::render::{BodyMesh, SunLight};
use crate::ui::controls::ui_controls_system;
use bevy::prelude::*;

/// System to spawn celestial bodies on startup
fn spawn_celestial_bodies(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
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

/// System to update body positions from physics state
fn update_body_transforms(state: Res<AppState>, mut query: Query<&mut Transform, With<BodyMesh>>) {
    let mut iter = query.iter_mut();

    for (i, body) in state.bodies.iter().enumerate() {
        if let Some(mut transform) = iter.next() {
            let visual_scale = if i == 0 {
                0.5
            } else {
                calculate_visual_radius(6000.0) * 2.0
            };

            transform.translation = Vec3::new(
                body.position.x as f32 * 10.0,
                body.position.y as f32 * 10.0,
                body.position.z as f32 * 10.0,
            );
            transform.scale = Vec3::splat(visual_scale);
        }
    }
}

/// Setup system to initialize the scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_celestial_bodies(&mut commands, &mut meshes, &mut materials);

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

/// Plugin to integrate all systems
pub struct SolarSystemPlugin;

impl Plugin for SolarSystemPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AppState>()
            .init_resource::<CameraController>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    orbital_physics_system,
                    update_body_transforms,
                    camera_follow_system,
                    ui_controls_system,
                ),
            );
    }
}

/// Initialize the solar system with real orbital data
pub fn init_solar_system() -> AppState {
    let mut bodies = vec![BodyState::new("Sun", None)];

    bodies.push(BodyState::new(
        "Mercury",
        Some(Orbit {
            semi_major_axis_au: 0.387,
            eccentricity: 0.2056,
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
            eccentricity: 0.0067,
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
            semi_major_axis_au: 1.0,
            eccentricity: 0.0167,
            inclination_rad: 0.0,
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
            orbital_period_days: 4333.0,
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
            orbital_period_days: 10759.0,
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
            orbital_period_days: 30687.0,
        }),
    ));

    bodies.push(BodyState::new(
        "Neptune",
        Some(Orbit {
            semi_major_axis_au: 30.05,
            eccentricity: 0.0113,
            inclination_rad: 0.0309,
            longitude_ascending_rad: 2.3001,
            argument_of_periapsis_rad: 0.7840,
            mean_anomaly_at_epoch_rad: 0.0,
            epoch_days: 0.0,
            orbital_period_days: 60190.0,
        }),
    ));

    AppState {
        elapsed_days: 0.0,
        simulation_speed: 1.0,
        bodies,
    }
}
