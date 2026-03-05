# Modular Solar System Refactor - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Transform fragile index-based architecture into modular, entity-driven system with clean separation between physics, rendering, and UI, fixing pause button and removing all dead code.

**Architecture:** Replace Vec-based index mapping with HashMap<Entity, BodyState>. Camera stores Entity instead of usize index. UI uses entity references. Modules have clean responsibilities.

**Tech Stack:** Rust, Bevy 0.12, egui for UI, Keplerian orbital mechanics

---

## Task 1: Add Entity Field to BodyState

**Files:**
- Modify: `src/physics/system.rs:28-50`

**Step 1: Update BodyState struct**

Add `entity` field to track the Bevy entity for each body:

```rust
use bevy::prelude::Entity;

#[derive(Debug, Clone)]
pub struct BodyState {
    pub entity: Entity,  // NEW: Track entity for entity-based lookups
    pub name: String,
    pub orbit: Option<Orbit>,
    pub position: DVec3,
    pub mean_anomaly_rad: f64,
}
```

**Step 2: Update BodyState::new to accept entity parameter**

```rust
impl BodyState {
    pub fn new(entity: Entity, name: impl Into<String>, orbit: Option<Orbit>) -> Self {
        Self {
            entity,
            name: name.into(),
            orbit,
            position: DVec3::ZERO,
            mean_anomaly_rad: 0.0,
        }
    }
}
```

**Step 3: Run cargo check to verify compilation**

```bash
cd "C:\Users\nicolas\Documents\GitHub\sites\universe sim"
cargo check
```

Expected: Compilation errors about missing `entity` parameter in calls to `BodyState::new()`

**Step 4: Commit**

```bash
git add src/physics/system.rs
git commit -m "refactor: add entity field to BodyState for entity-based lookups"
```

---

## Task 2: Create New PhysicsState with HashMap

**Files:**
- Modify: `src/physics/system.rs:52-80`

**Step 1: Add new PhysicsState struct below BodyState**

```rust
use std::collections::HashMap;

/// New physics state using entity-based lookups (replaces Vec-based AppState)
#[derive(Debug, Resource, Clone)]
pub struct PhysicsState {
    /// Accumulated simulation time in days since the simulation began.
    pub elapsed_days: f64,

    /// Time multiplier: 1.0 = real-time, 1000.0 = 1000 days per real second.
    /// Clamped to [0.0, MAX_SIMULATION_SPEED] each frame (0.0 enables pause).
    pub simulation_speed: f64,

    /// Map from entity to body state. Enables dynamic add/remove of bodies.
    pub bodies: HashMap<Entity, BodyState>,
}

impl Default for PhysicsState {
    fn default() -> Self {
        Self {
            elapsed_days: 0.0,
            simulation_speed: 1.0,
            bodies: HashMap::new(),
        }
    }
}
```

**Step 2: Run cargo check**

```bash
cargo check
```

Expected: Should compile (new struct doesn't conflict with existing AppState yet)

**Step 3: Commit**

```bash
git add src/physics/system.rs
git commit -m "refactor: add PhysicsState with HashMap<Entity, BodyState>"
```

---

## Task 3: Update init_solar_system to Return Entities

**Files:**
- Modify: `src/app.rs:55-98`

**Step 1: Find the init_solar_system function**

Look for the function that initializes AppState with the solar system data.

**Step 2: Update return type to Vec<Entity>**

Change function signature to return spawned entities along with body data:

```rust
fn init_solar_system(mut commands: Commands) -> (Vec<BodyState>, Vec<Entity>) {
    // Keep all existing orbit definitions...

    // At the end, return both body states and entities
    let bodies = vec![
        BodyState::new(/* will fill in spawn_celestial_bodies */),
        // ... rest of bodies
    ];

    // For now, return empty vec for entities - will fill in next task
    (bodies, vec![])
}
```

**Step 3: Run cargo check**

```bash
cargo check
```

Expected: Error about init_solar_system signature mismatch

**Step 4: Commit**

```bash
git add src/app.rs
git commit -m "refactor: update init_solar_system signature for entity tracking"
```

---

## Task 4: Update spawn_celestial_bodies to Capture Entities

**Files:**
- Modify: `src/app.rs:100-150` (adjust based on actual location)

**Step 1: Modify spawn_celestial_bodies to return entities**

```rust
fn spawn_celestial_bodies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) -> Vec<Entity> {
    let mut entities = Vec::new();

    // Sun
    let sun_entity = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Sphere { radius: 6000.0 })),
            material: materials.add(Color::YELLOW),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        BodyMesh,
    )).id();
    entities.push(sun_entity);

    // Repeat for all 8 planets...
    // Each: commands.spawn(...).id() and push to entities

    entities
}
```

**Step 2: Update init_solar_system to use spawned entities**

```rust
fn init_solar_system(mut commands: Commands) -> (Vec<BodyState>, Vec<Entity>) {
    let entities = spawn_celestial_bodies(
        commands,
        // ... pass resources
    );

    let bodies = vec![
        BodyState::new(entities[0], "Sun", None),
        BodyState::new(entities[1], "Mercury", Some(mercury_orbit)),
        // ... rest of bodies using entities from spawn_celestial_bodies
    ];

    (bodies, entities)
}
```

**Step 3: Run cargo check**

```bash
cargo check
```

Expected: Errors about mismatched parameter counts

**Step 4: Commit**

```bash
git add src/app.rs
git commit -m "refactor: capture entities when spawning celestial bodies"
```

---

## Task 5: Initialize PhysicsState Alongside AppState

**Files:**
- Modify: `src/app.rs:1-10` (imports)
- Modify: `src/app.rs:130-160` (plugin build)

**Step 1: Add PhysicsState import**

```rust
use crate::physics::system::{AppState, PhysicsState};
```

**Step 2: Initialize both resources in plugin**

```rust
fn build(&self, app: &mut App) {
    // ... existing code ...

    // Initialize both AppState (old) and PhysicsState (new)
    app.init_resource::<AppState>();
    app.init_resource::<PhysicsState>();

    // ... rest of plugin build ...
}
```

**Step 3: Run cargo check**

```bash
cargo check
```

Expected: Should compile

**Step 4: Run the application**

```bash
cargo run
```

Expected: Application launches, planets render normally

**Step 5: Commit**

```bash
git add src/app.rs
git commit -m "refactor: initialize PhysicsState resource alongside AppState"
```

---

## Task 6: Populate PhysicsState from init_solar_system

**Files:**
- Modify: `src/app.rs:55-98`

**Step 1: Update init_solar_system to populate PhysicsState**

```rust
fn init_solar_system(mut commands: Commands, mut physics_state: ResMut<PhysicsState>) {
    let entities = spawn_celestial_bodies(/* ... */);

    let bodies = vec![
        BodyState::new(entities[0], "Sun", None),
        BodyState::new(entities[1], "Mercury", Some(mercury_orbit)),
        // ... rest of bodies
    ];

    // Populate both AppState (old) and PhysicsState (new)
    // (AppState will be removed in Phase 3)
    let app_state = AppState {
        elapsed_days: 0.0,
        simulation_speed: 1.0,
        bodies: bodies.clone(),
    };
    commands.insert_resource(app_state);

    // Populate new PhysicsState
    for body in bodies {
        physics_state.bodies.insert(body.entity, body);
    }
}
```

**Step 2: Run cargo check**

```bash
cargo check
```

Expected: Errors about init_solar_system parameters

**Step 3: Run the application**

```bash
cargo run
```

Expected: Application launches, PhysicsState is populated

**Step 4: Commit**

```bash
git add src/app.rs
git commit -m "refactor: populate PhysicsState with body data from init_solar_system"
```

---

## Task 7: Add sync_physics_to_transforms System

**Files:**
- Create: `src/physics/sync.rs`

**Step 1: Create new sync module**

```rust
use bevy::prelude::*;
use crate::physics::system::{PhysicsState, BodyState};
use crate::render::BodyMesh;
use crate::types::dvec3_to_vec3;

/// Syncs physics positions to Bevy Transforms
/// Queries entities with BodyMesh component and updates their Transform
/// from the corresponding BodyState in PhysicsState
pub fn sync_physics_to_transforms(
    physics: Res<PhysicsState>,
    mut query: Query<(&mut Transform, &BodyMesh)>,
) {
    for (mut transform, _body_mesh) in query.iter_mut() {
        let entity = query.entity(); // Get the entity ID

        if let Some(body_state) = physics.bodies.get(&entity) {
            transform.translation = dvec3_to_vec3(body_state.position);
        }
    }
}
```

**Step 2: Add mod sync to physics/mod.rs**

```rust
pub mod sync;
pub mod system;
pub mod kepler;

pub use sync::sync_physics_to_transforms;
```

**Step 3: Run cargo check**

```bash
cargo check
```

Expected: Error about query.entity()

**Step 4: Fix the query usage**

```rust
pub fn sync_physics_to_transforms(
    physics: Res<PhysicsState>,
    mut query: Query<&mut Transform, With<BodyMesh>>,
) {
    for (entity, mut transform) in query.iter_mut() {
        if let Some(body_state) = physics.bodies.get(&entity) {
            transform.translation = dvec3_to_vec3(body_state.position);
        }
    }
}
```

**Step 5: Commit**

```bash
git add src/physics/sync.rs src/physics/mod.rs
git commit -m "feat: add sync_physics_to_transforms system for entity-based updates"
```

---

## Task 8: Register sync_physics_to_transforms System

**Files:**
- Modify: `src/app.rs` (plugin build section)

**Step 1: Add system to plugin**

```rust
use crate::physics::sync_physics_to_transforms;

fn build(&self, app: &mut App) {
    app.add_systems(
        Update,
        (
            orbital_physics_system,
            sync_physics_to_transforms,  // NEW
            update_body_transforms,      // OLD (will remove in Phase 3)
            ui_controls_system,
            camera_focus_system,
        ).chain(),
    );
}
```

**Step 2: Run cargo check**

```bash
cargo check
```

Expected: Should compile

**Step 3: Run the application**

```bash
cargo run
```

Expected: Planets still render correctly (both old and new systems running)

**Step 4: Commit**

```bash
git add src/app.rs
git commit -m "feat: register sync_physics_to_transforms system"
```

---

## Task 9: Update CameraController to Use Entity

**Files:**
- Modify: `src/camera.rs:11-27`

**Step 1: Change focus_index to focus entity**

```rust
#[derive(Resource)]
pub struct CameraController {
    pub distance: f64,
    pub focus: Entity,  // Changed from focus_index: usize
    pub pitch: f64,
    pub yaw: f64,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            distance: 10.0,
            focus: Entity::PLACEHOLDER,  // Will set to Sun entity at runtime
            pitch: 0.3,
            yaw: 0.0,
        }
    }
}
```

**Step 2: Run cargo check**

```bash
cargo check
```

Expected: Errors about missing focus_index

**Step 3: Commit**

```bash
git add src/camera.rs
git commit -m "refactor: change CameraController.focus from usize to Entity"
```

---

## Task 10: Initialize CameraController with Sun Entity

**Files:**
- Modify: `src/app.rs` (plugin build or initialization)

**Step 1: Store Sun entity in a resource for camera initialization**

Create a new resource to track the Sun entity:

```rust
#[derive(Resource)]
struct SunEntity(Entity);
```

**Step 2: Update init_solar_system to insert SunEntity**

```rust
fn init_solar_system(mut commands: Commands, mut physics_state: ResMut<PhysicsState>) {
    let entities = spawn_celestial_bodies(/* ... */);

    // Sun is always at index 0
    let sun_entity = entities[0];
    commands.insert_resource(SunEntity(sun_entity));

    // ... rest of initialization
}
```

**Step 3: Update CameraController initialization**

```rust
fn build(&self, app: &mut App) {
    // ... existing code ...

    // Initialize camera after SunEntity is available
    app.add_systems(Startup, |mut commands: Commands, sun: Res<SunEntity>| {
        commands.insert_resource(CameraController {
            distance: 10.0,
            focus: sun.0,
            pitch: 0.3,
            yaw: 0.0,
        });
    });
}
```

**Step 4: Run cargo check**

```bash
cargo check
```

Expected: Compilation errors

**Step 5: Commit**

```bash
git add src/app.rs
git commit -m "feat: initialize CameraController with Sun entity"
```

---

## Task 11: Update camera_focus_system to Use Entity

**Files:**
- Modify: `src/camera.rs:30-80`

**Step 1: Rewrite camera_focus_system to query by entity**

```rust
pub fn camera_focus_system(
    mut camera: ResMut<CameraController>,
    time: Res<Time>,
    transforms: Query<&Transform, With<BodyMesh>>,
) {
    let target_transform = match transforms.get(camera.focus) {
        Ok(t) => t,
        Err(_) => return, // Focused entity not found, skip
    };

    let target_pos = target_transform.translation;

    // Calculate camera position based on distance, pitch, yaw
    let camera_pos = target_pos + Vec3::new(
        camera.distance.sin() * camera.pitch.cos(),
        camera.pitch.sin(),
        camera.distance.cos() * camera.pitch.cos(),
    );

    // Update camera transform (implementation specific)
    // ...
}
```

**Step 2: Run cargo check**

```bash
cargo check
```

Expected: Errors about system signature

**Step 3: Commit**

```bash
git add src/camera.rs
git commit -m "refactor: update camera_focus_system to use entity queries"
```

---

## Task 12: Update UI to Use Entity Selection

**Files:**
- Modify: `src/ui/controls.rs:10-86`

**Step 1: Change ui_controls_system to store selected entity**

```rust
pub fn ui_controls_system(
    mut contexts: EguiContexts,
    mut state: ResMut<PhysicsState>,  // Changed from AppState
    mut camera: ResMut<CameraController>,
) {
    let ctx = match contexts.ctx_mut() {
        Ok(ctx) => ctx,
        Err(_) => return,
    };

    egui::Window::new("Solar System Controls")
        .default_pos([10.0, 10.0])
        .show(ctx, |ui| {
            ui.heading("Simulation");

            ui.add(
                egui::Slider::new(&mut state.simulation_speed, 0.0..=1000.0)
                    .text("Speed")
                    .logarithmic(true),
            );

            if ui.button(if state.simulation_speed > 0.0 {
                "⏸ Pause"
            } else {
                "▶ Resume"
            }).clicked() {
                state.simulation_speed = if state.simulation_speed > 0.0 {
                    0.0
                } else {
                    1.0
                };
            }

            ui.separator();
            ui.heading("Camera");

            ui.add(
                egui::Slider::new(&mut camera.distance, 1.0..=100.0)
                    .text("Zoom (AU)")
                    .logarithmic(true),
            );

            // Build entity list for ComboBox
            let bodies: Vec<_> = state.bodies.values().collect();

            if !bodies.is_empty() {
                let current_name = bodies
                    .iter()
                    .find(|b| b.entity == camera.focus)
                    .map(|b| b.name.as_str())
                    .unwrap_or("Unknown");

                egui::ComboBox::from_label("Focus On")
                    .selected_text(current_name)
                    .show_ui(ui, |ui| {
                        for body in &bodies {
                            ui.selectable_value(&mut camera.focus, body.entity, &body.name);
                        }
                    });
            }

            ui.separator();
            ui.label(format!("Elapsed: {:.1} days", state.elapsed_days));
        });
}
```

**Step 2: Update imports in controls.rs**

```rust
use crate::physics::system::PhysicsState;  // Changed from AppState
```

**Step 3: Run cargo check**

```bash
cargo check
```

Expected: Errors about type mismatches

**Step 4: Commit**

```bash
git add src/ui/controls.rs
git commit -m "refactor: update UI to use entity-based selection"
```

---

## Task 13: Fix Pause Button Bug

**Files:**
- Modify: `src/physics/system.rs:13`

**Step 1: Change MIN_SIMULATION_SPEED to 0.0**

```rust
const MIN_SIMULATION_SPEED: f64 = 0.0;  // Changed from 1.0 - enables pause
```

**Step 2: Remove bug comments**

Remove lines 9-12 (the comment explaining the bug):

```rust
// DELETED: NOTE: MIN_SIMULATION_SPEED = 1.0 means the simulation cannot actually be paused
```

**Step 3: Remove bug comment in controls.rs**

Remove lines 34-37 from `src/ui/controls.rs`:

```rust
// DELETED: NOTE: This pause button is currently non-functional...
```

**Step 4: Run cargo check**

```bash
cargo check
```

Expected: Should compile

**Step 5: Test pause button**

```bash
cargo run
```

Expected: Pause button now works - simulation stops when clicked

**Step 6: Commit**

```bash
git add src/physics/system.rs src/ui/controls.rs
git commit -m "fix: enable pause button by setting MIN_SIMULATION_SPEED to 0.0"
```

---

## Task 14: Remove Old AppState and Update Systems

**Files:**
- Modify: `src/physics/system.rs:52-80`
- Modify: `src/app.rs`
- Modify: All files importing AppState

**Step 1: Delete old AppState struct**

Remove the entire AppState definition from `src/physics/system.rs` (lines 52-80)

**Step 2: Rename PhysicsState to AppState**

```rust
// Rename PhysicsState -> AppState (this is now the canonical one)
#[derive(Debug, Resource, Clone)]
pub struct AppState {
    // ... keep same fields as PhysicsState
}
```

**Step 3: Update orbital_physics_system to use HashMap**

```rust
pub fn orbital_physics_system(time: Res<Time>, mut state: ResMut<AppState>) {
    let simulation_speed = state
        .simulation_speed
        .clamp(MIN_SIMULATION_SPEED, MAX_SIMULATION_SPEED);
    state.simulation_speed = simulation_speed;

    let delta_days = (time.delta().as_secs_f64() / SECONDS_PER_DAY) * simulation_speed;
    state.elapsed_days += delta_days;

    let simulation_time_days = state.elapsed_days;

    // Iterate over HashMap values instead of Vec
    for (_entity, body) in state.bodies.iter_mut() {
        // ... rest of physics logic (unchanged)
    }
}
```

**Step 4: Run cargo check**

```bash
cargo check
```

Expected: Errors in files still using old AppState

**Step 5: Commit**

```bash
git add src/physics/system.rs
git commit -m "refactor: replace Vec-based AppState with HashMap-based version"
```

---

## Task 15: Remove Dead Code - state.rs

**Files:**
- Delete: `src/state.rs`

**Step 1: Delete state.rs file**

```bash
rm "C:\Users\nicolas\Documents\GitHub\sites\universe sim\src\state.rs"
```

**Step 2: Remove module declaration from mod.rs**

Edit `src/lib.rs` or `src/main.rs` to remove:

```rust
// DELETED: pub mod state;
```

**Step 3: Run cargo check**

```bash
cargo check
```

Expected: Errors about missing state module imports

**Step 4: Fix imports**

Search for `use crate::state::` and replace with appropriate imports from the new AppState

**Step 5: Commit**

```bash
git add src/state.rs src/lib.rs src/main.rs
git commit -m "cleanup: remove unused state.rs module"
```

---

## Task 16: Remove Dead Code - resources.rs

**Files:**
- Delete: `src/resources.rs`

**Step 1: Delete resources.rs file**

```bash
rm "C:\Users\nicolas\Documents\GitHub\sites\universe sim\src\resources.rs"
```

**Step 2: Remove module declaration**

```bash
# DELETED: pub mod resources;
```

**Step 3: Run cargo check**

```bash
cargo check
```

Expected: Errors about missing resources module

**Step 4: Commit**

```bash
git add src/resources.rs src/lib.rs
git commit -m "cleanup: remove unused resources.rs module"
```

---

## Task 17: Remove Unused Orbit from body.rs

**Files:**
- Modify: `src/physics/body.rs:65-86`

**Step 1: Delete duplicate Orbit struct**

Remove the unused Orbit definition from `src/physics/body.rs`

**Step 2: Remove unused self-import**

Remove line 1-2:

```rust
// DELETED: use crate::physics::CelestialBody;
```

**Step 3: Run cargo check**

```bash
cargo check
```

Expected: Should compile (these were never used)

**Step 4: Run clippy to find more unused code**

```bash
cargo clippy
```

**Step 5: Commit**

```bash
git add src/physics/body.rs
git commit -m "cleanup: remove unused Orbit struct and imports from body.rs"
```

---

## Task 18: Remove update_body_transforms (Old System)

**Files:**
- Modify: `src/app.rs` (system registration)

**Step 1: Remove old update_body_transforms from plugin**

```rust
fn build(&self, app: &mut App) {
    app.add_systems(
        Update,
        (
            orbital_physics_system,
            sync_physics_to_transforms,  // NEW
            // REMOVED: update_body_transforms,
            ui_controls_system,
            camera_focus_system,
        ).chain(),
    );
}
```

**Step 2: Delete update_body_transforms function**

Find and remove the old `update_body_transforms` function

**Step 3: Run cargo check**

```bash
cargo check
```

Expected: Should compile

**Step 4: Test application**

```bash
cargo run
```

Expected: Planets render and orbit correctly with new system

**Step 5: Commit**

```bash
git add src/app.rs
git commit -m "refactor: remove old update_body_transforms system, use entity-based sync"
```

---

## Task 19: Final Testing and Verification

**Files:**
- Test: All functionality

**Step 1: Run full test suite**

```bash
cargo test
```

**Step 2: Run clippy for final cleanup**

```bash
cargo clippy -- -D warnings
```

**Step 3: Verify all features work**

```bash
cargo run
```

Test checklist:
- [ ] Planets render correctly
- [ ] Planets orbit at correct speeds
- [ ] Speed slider works (0.0 to 1000.0)
- [ ] Pause button works (stops simulation)
- [ ] Resume button works (resumes simulation)
- [ ] Camera zoom works
- [ ] Camera focus works for all bodies
- [ ] Elapsed time displays correctly

**Step 4: Run cargo fmt**

```bash
cargo fmt
```

**Step 5: Final commit**

```bash
git add .
git commit -m "refactor: complete migration to entity-based architecture

- All systems use entity references instead of indices
- Pause button fixed (MIN_SIMULATION_SPEED = 0.0)
- All dead code removed (state.rs, resources.rs, unused types)
- Camera, UI, physics all use HashMap<Entity, BodyState>
- Enables future features: dynamic bodies, orbit trails, multi-system support
"
```

---

## Task 20: Documentation and cleanup

**Files:**
- Update: README.md (if exists)
- Update: docs/ directory

**Step 1: Update architecture documentation**

Add section explaining new entity-based architecture

**Step 2: Add inline code comments**

Document key design decisions:
- Why HashMap<Entity, BodyState> instead of Vec
- How entity references enable dynamic bodies
- Module responsibilities

**Step 3: Commit documentation**

```bash
git add README.md docs/
git commit -m "docs: update architecture documentation for entity-based design"
```

---

## Success Criteria

After completing all tasks:
- ✅ Pause button works correctly (simulation stops at 0.0 speed)
- ✅ All dead code removed (state.rs, resources.rs, unused imports)
- ✅ No duplicate types remain (single AppState, single Orbit)
- ✅ Entity references replace all index mappings
- ✅ Modules have clean boundaries (physics, render, ui, camera)
- ✅ All existing features work (planets render, orbit, UI controls, camera)
- ✅ Architecture supports dynamic bodies, visual features, multi-system support
- ✅ No clippy warnings
- ✅ All tests pass
