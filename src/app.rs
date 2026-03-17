use crate::camera::{camera_follow_system, mouse_camera_control, CameraController};
use crate::error::LastError;
use crate::physics::sync_physics_to_transforms;
use crate::physics::system::orbital_physics_system;
use crate::physics::system::{BodyState, PhysicsState};
use crate::solar_system_data::{PLANET_DATA, MOON_DATA};
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

/// Spawns a single celestial body sphere. Scale is set later by sync from radius_km.
fn spawn_body(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    mesh: Mesh,
    material: StandardMaterial,
) -> Entity {
    commands
        .spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(material)),
            Transform::IDENTITY,
            BodyMesh,
        ))
        .id()
}

/// Spawns 14 sphere entities (1 Sun + 8 planets + 5 major moons) in the same order as `init_solar_system`.
///
/// IMPORTANT: The spawn order must match the order of bodies in PhysicsState because
/// `sync_physics_to_transforms` maps bodies to entities by entity ID lookup.
///
/// Returns the spawned entity IDs in order (Sun first, then planets, then moons).
fn spawn_celestial_bodies(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Vec<Entity> {
    let mut entities = Vec::new();

    // Sun: higher mesh resolution (32×16) and emissive material so it glows
    let sun_mesh = create_sphere_mesh(1.0, 32, 16);
    let sun_material = StandardMaterial {
        base_color: Color::srgb(1.0, 0.95, 0.2),
        emissive: LinearRgba::rgb(1.0, 0.9, 0.2),
        ..default()
    };
    entities.push(spawn_body(commands, meshes, materials, sun_mesh, sun_material));

    // Planet colors chosen to visually distinguish them at a glance
    let planet_colors: [Color; 8] = [
        Color::srgb(0.7, 0.7, 0.7),   // Mercury
        Color::srgb(0.9, 0.7, 0.4),   // Venus
        Color::srgb(0.2, 0.5, 0.9),   // Earth
        Color::srgb(0.9, 0.3, 0.1),   // Mars
        Color::srgb(0.8, 0.6, 0.4),   // Jupiter
        Color::srgb(0.9, 0.8, 0.5),   // Saturn
        Color::srgb(0.4, 0.8, 0.9),   // Uranus
        Color::srgb(0.2, 0.4, 0.9),   // Neptune
    ];

    for color in planet_colors {
        let mesh = create_sphere_mesh(1.0, 16, 8);
        let material = StandardMaterial {
            base_color: color,
            ..default()
        };
        entities.push(spawn_body(commands, meshes, materials, mesh, material));
    }

    // Major moons: Moon (Earth), Io, Europa, Ganymede, Callisto (Jupiter)
    let moon_colors: [Color; 5] = [
        Color::srgb(0.75, 0.75, 0.8),  // Moon - silvery grey
        Color::srgb(0.9, 0.85, 0.5),   // Io - sulphur yellow
        Color::srgb(0.9, 0.9, 0.95),   // Europa - ice white
        Color::srgb(0.6, 0.55, 0.5),   // Ganymede - grey-brown
        Color::srgb(0.4, 0.35, 0.3),   // Callisto - dark grey
    ];

    for color in moon_colors {
        let mesh = create_sphere_mesh(1.0, 12, 6);
        let material = StandardMaterial {
            base_color: color,
            ..default()
        };
        entities.push(spawn_body(commands, meshes, materials, mesh, material));
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

/// Constructs the initial PhysicsState with real orbital elements for all 8 planets and 5 moons.
///
/// Source: NASA planetary fact sheets / JPL Horizons epoch J2000.0.
/// Entities must be in spawn order: Sun, 8 planets, 5 moons.
///
/// # Panics
/// Logs a warning if `entities.len() < EXPECTED_BODY_COUNT`; missing entities use placeholder.
pub fn init_solar_system(entities: Vec<Entity>) -> PhysicsState {
    if entities.len() < EXPECTED_BODY_COUNT {
        error!(
            "init_solar_system: expected at least {} entities, got {}. Filling missing with placeholder.",
            EXPECTED_BODY_COUNT,
            entities.len()
        );
    }

    let get_entity = |i: usize| entities.get(i).copied().unwrap_or(Entity::PLACEHOLDER);

    let mut bodies = vec![BodyState::new(get_entity(0), "Sun", None).with_radius(695_700.0)];

    for (i, data) in PLANET_DATA.iter().enumerate() {
        bodies.push(
            BodyState::new(get_entity(i + 1), data.name, Some(data.orbit))
                .with_radius(data.radius_km),
        );
    }

    for (i, moon) in MOON_DATA.iter().enumerate() {
        let entity_index = 9 + i;
        let parent_entity = get_entity(moon.parent_index);
        bodies.push(BodyState::moon(
            get_entity(entity_index),
            moon.name,
            moon.to_orbit(),
            parent_entity,
            moon.radius_km,
        ));
    }

    let mut physics_state = PhysicsState::default();
    for body in bodies {
        physics_state.bodies.insert(body.entity, body);
    }

    physics_state
}
