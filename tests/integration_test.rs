use bevy::prelude::*;
use universe_sim::app::SolarSystemPlugin;
use universe_sim::physics::system::PhysicsState;
use universe_sim::render::BodyMesh;

fn create_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(SolarSystemPlugin)
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>();
    app
}

#[test]
fn test_plugin_builds() {
    let app = create_test_app();

    assert!(
        app.world().contains_resource::<PhysicsState>(),
        "PhysicsState should be initialized"
    );
    assert!(
        app.world().contains_resource::<Assets<Mesh>>(),
        "Assets<Mesh> should be initialized"
    );
    assert!(
        app.world().contains_resource::<Assets<StandardMaterial>>(),
        "Assets<StandardMaterial> should be initialized"
    );
}

#[test]
fn test_entities_spawned() {
    let mut app = create_test_app();

    app.update();

    let mut body_query = app.world_mut().query_filtered::<&BodyMesh, ()>();
    let body_count = body_query.iter(&app.world()).count();

    assert_eq!(
        body_count, 14,
        "Expected 14 celestial bodies (1 Sun + 8 planets + 5 moons), found {}",
        body_count
    );
}

#[test]
fn test_physics_state_has_bodies() {
    let mut app = create_test_app();
    app.update();

    let state = app.world().resource::<PhysicsState>();

    assert!(
        !state.bodies.is_empty(),
        "PhysicsState.bodies should not be empty after spawning"
    );

    assert_eq!(
        state.bodies.len(),
        14,
        "Expected 14 bodies in PhysicsState, found {}",
        state.bodies.len()
    );

    for (_entity, body_state) in &state.bodies {
        assert!(
            !body_state.name.is_empty(),
            "Body '{}' should have a name",
            body_state.name
        );
        if body_state.name == "Sun" {
            assert!(body_state.orbit.is_none(), "Sun should not have orbit data");
        } else {
            assert!(
                body_state.orbit.is_some(),
                "Body '{}' should have orbit data",
                body_state.name
            );
        }
    }
}

#[test]
fn test_sun_has_no_orbit() {
    let mut app = create_test_app();
    app.update();

    let state = app.world().resource::<PhysicsState>();

    let sun = state
        .bodies
        .values()
        .find(|b| b.name == "Sun")
        .expect("Sun should exist");
    assert_eq!(sun.name, "Sun");
    assert!(sun.orbit.is_none());
}

#[test]
fn test_planets_and_moons_have_orbital_data() {
    let mut app = create_test_app();
    app.update();

    let state = app.world().resource::<PhysicsState>();

    let bodies_with_orbits: Vec<&str> = state
        .bodies
        .values()
        .filter(|b| b.orbit.is_some())
        .map(|b| b.name.as_str())
        .collect();

    assert_eq!(
        bodies_with_orbits.len(),
        13,
        "Expected 13 bodies with orbital data (8 planets + 5 moons), found {:?}",
        bodies_with_orbits
    );

    let expected_planets = [
        "Mercury", "Venus", "Earth", "Mars", "Jupiter", "Saturn", "Uranus", "Neptune",
    ];
    for planet in expected_planets {
        assert!(
            bodies_with_orbits.iter().any(|p| *p == planet),
            "Planet '{}' should have orbital data",
            planet
        );
    }

    let expected_moons = ["Moon", "Io", "Europa", "Ganymede", "Callisto"];
    for moon in expected_moons {
        assert!(
            bodies_with_orbits.iter().any(|p| *p == moon),
            "Moon '{}' should have orbital data",
            moon
        );
    }
}

#[test]
fn test_earth_orbital_elements() {
    let mut app = create_test_app();
    app.update();

    let state = app.world().resource::<PhysicsState>();

    let earth = state
        .bodies
        .values()
        .find(|b| b.name == "Earth")
        .expect("Earth should exist");

    let orbit = earth
        .orbit
        .as_ref()
        .expect("Earth should have orbital data");

    assert!((orbit.semi_major_axis_au - 1.0).abs() < 0.01);
    assert!((orbit.orbital_period_days - 365.25).abs() < 1.0);
    assert!((orbit.inclination_rad - 0.0).abs() < 0.001);
}
