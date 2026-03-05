# Modular Solar System Simulator - Design Document

**Date**: 2025-03-05
**Author**: Claude Code
**Status**: Approved

## Overview

Transform the fragile index-based architecture into a modular, entity-driven system with clean separation between physics, rendering, and UI. This enables dynamic bodies, future visual features, and multi-system support.

## Problem Statement

### Current Issues

1. **Bug 1: Pause Button Completely Broken** (10 pts, 100% confidence)
   - Pause sets `simulation_speed = 0.0` but `MIN_SIMULATION_SPEED = 1.0` clamps it back to 1.0 every frame
   - Explicitly documented as non-functional in comments

2. **Bug 13: Multiple Unused Marker Components** (1 pt, 100% confidence)
   - `OrbitTrail` and `SunLight` components defined but never spawned

3. **Bug 16: Unused Imports** (1 pt, 95% confidence)
   - `src/physics/body.rs:1` has a self-import that's never used

4. **Fragile Architecture**: Index-based mapping requires `AppState.bodies[N]` to match entity N
5. **Duplicate Types**: Multiple versions of `AppState`, `Orbit`, camera types
6. **Dead Code**: Unused files and types add maintenance burden

## Design Goals

1. **Fix confirmed bugs** (pause button, unused components, unused imports)
2. **Eliminate index-based mapping** with entity references
3. **Consolidate duplicate types** into single authoritative versions
4. **Remove all dead code**
5. **Enable future features**:
   - Dynamic bodies (add/remove at runtime)
   - Visual enhancements (orbit trails, planet rings, lighting)
   - Multi-system support (hierarchical systems like Earth-Moon)

## Module Structure

### Physics Module (`src/physics/`)

**Responsibilities**: Owns all runtime physics state and Keplerian orbital mechanics

```rust
pub struct PhysicsState {
    pub elapsed_days: f64,
    pub simulation_speed: f64,  // Clamped to [0.0, 1000.0] - fixes pause bug
    pub bodies: HashMap<Entity, BodyState>,
}

pub struct BodyState {
    pub entity: Entity,
    pub name: String,
    pub orbit: Option<Orbit>,
    pub position: DVec3,
    pub mean_anomaly_rad: f64,
    pub parent: Option<Entity>,  // For hierarchical systems
}
```

**Systems**:
- `orbital_physics_system`: Updates positions using Kepler's equation
- `sync_physics_to_transforms`: Writes physics positions to Transform components

### Rendering Module (`src/render/`)

**Responsibilities**: Owns visual representation (meshes, materials, components)

```rust
#[derive(Component)]
pub struct BodyMesh;  // Marks renderable bodies

#[derive(Component)]
#[allow(dead_code)]
pub struct OrbitTrail;  // Reserved for future orbit trail rendering

#[derive(Component)]
pub struct SunLight;  // Marker for scene's DirectionalLight
```

**Systems**:
- `spawn_celestial_bodies`: Creates entities with BodyMesh component
- `update_body_transforms`: Syncs physics positions to Transform (refactored to query by entity)

### UI Module (`src/ui/`)

**Responsibilities**: Owns user interaction and presentation

```rust
// ui_controls_system refactored to use entity references
pub fn ui_controls_system(
    mut contexts: EguiContexts,
    mut state: ResMut<PhysicsState>,
    mut camera: ResMut<CameraController>,
)
```

**Changes**: ComboBox stores `Entity` instead of `usize` index

### Camera Module (`src/camera/`)

**Responsibilities**: Owns camera control and focus mechanics

```rust
pub struct CameraController {
    pub distance: f64,
    pub focus: Entity,  // Changed from focus_index: usize
    pub pitch: f64,
    pub yaw: f64,
}
```

## Entity Reference Architecture

### Current Problem
Index-based mapping requires maintaining parallel arrays:
- `AppState.bodies[N]` must correspond to entity N
- Spawn order must match body order
- Documented as "IMPORTANT" invariant but fragile

### Solution
Store entity references directly in physics state:

```rust
pub struct PhysicsState {
    pub bodies: HashMap<Entity, BodyState>,
}

pub struct BodyState {
    pub entity: Entity,  // Self-reference
    // ... other fields
}
```

### Benefits
- Add/remove bodies dynamically without breaking index mappings
- Camera focus uses `Entity` instead of `usize`
- UI ComboBox stores selected entity directly
- Systems query by entity: `Query<&Transform, With<BodyMesh>>`

## Bug Fixes & Cleanup

### Fix 1: Pause Button
**Change**: `MIN_SIMULATION_SPEED` from `1.0` to `0.0` in `src/physics/system.rs`

**Before**:
```rust
const MIN_SIMULATION_SPEED: f64 = 1.0;  // Cannot pause
```

**After**:
```rust
const MIN_SIMULATION_SPEED: f64 = 0.0;  // Can pause
```

Remove non-functional comments explaining the bug.

### Fix 2: Remove Dead Code

Delete entire files:
- `src/state.rs` - unused `AppState` (duplicate of physics::system::AppState)
- `src/resources.rs` - unused resource types

Delete from `src/physics/body.rs`:
- Lines 1-2: unused self-import `use crate::physics::CelestialBody;`
- Lines 65-86: unused `Orbit` struct (use `physics::kepler::Orbit` instead)

### Fix 3: Consolidate Duplicates

**Keep**:
- `physics::system::AppState` → rename to `PhysicsState`
- `physics::kepler::Orbit` (active version)
- `camera::CameraController` (active version)
- `render::OrbitTrail`, `render::SunLight` (preserve for future, keep `#[allow(dead_code)]`)

**Remove**:
- All duplicate definitions
- All unused imports
- Run `cargo clippy` to catch remaining issues

## Migration Strategy

### Phase 1: Prepare Foundation
1. Create new `PhysicsState` struct alongside existing `AppState`
2. Add `entity: Entity` field to `BodyState`
3. Update spawn system to capture entity IDs and store them
4. Run tests to ensure nothing breaks yet

### Phase 2: Migrate Camera
1. Change `CameraController.focus_index: usize` → `focus: Entity`
2. Update `camera_focus_system` to use entity queries
3. Update UI to store selected entity instead of index
4. Test camera focus works correctly

### Phase 3: Migrate Physics System
1. Replace `AppState.bodies: Vec<BodyState>` with `HashMap<Entity, BodyState>`
2. Update `orbital_physics_system` to iterate HashMap values
3. Update `sync_physics_to_transforms` to query by entity
4. Fix pause button (change MIN_SIMULATION_SPEED to 0.0)

### Phase 4: Cleanup
1. Remove old `AppState` (now unused)
2. Remove dead code files (state.rs, resources.rs, etc.)
3. Remove unused imports
4. Final verification and testing

### Testing Strategy
After each phase:
- Run the app, verify planets render and orbit correctly
- Test UI controls: speed slider, pause button, camera focus
- Verify no regressions in orbital mechanics

## System Interactions & Data Flow

### Startup Flow
1. `init_solar_system()` creates initial physics data (orbits, positions)
2. `spawn_celestial_bodies()` spawns entities, returns `Vec<Entity>`
3. For each body: insert `BodyMesh` component, create `BodyState` with entity reference
4. Store all in `PhysicsState` resource
5. Initialize `CameraController` with focus on Sun entity

### Per-Frame Flow

**orbital_physics_system**:
- Reads `Time` delta
- Iterates `PhysicsState.bodies` (HashMap values)
- Updates `BodyState.position` based on Kepler's equation
- Returns early if `simulation_speed == 0.0` (pause working)

**sync_physics_to_transforms** (new system):
- Queries `(&mut Transform, &BodyMesh)`
- For each: lookup entity in `PhysicsState.bodies`
- Copy `BodyState.position` → `Transform.translation`
- Uses `types::dvec3_to_vec3` for f64→f32 conversion

**ui_controls_system**:
- Reads `PhysicsState` to build body name list
- Stores selected entity (not index)
- Updates `PhysicsState.simulation_speed` or `CameraController.focus`

**camera_focus_system**:
- Queries `Transform` of focused entity
- Smoothly interpolates camera position

### Error Handling
- Entity not found in HashMap: log warning, skip update (shouldn't happen)
- Invalid entity in camera focus: fallback to Sun entity

## Extensibility

### Dynamic Bodies (Feature A)
- `spawn_body()`: takes `BodyState` params, spawns entity, returns `Entity`
- `despawn_body()`: removes entity and removes from `PhysicsState.bodies`
- UI can call these at runtime for asteroids, comets, spacecraft
- Camera can focus on dynamically added bodies

### Visual Enhancements (Feature B)
- `OrbitTrail` component: ready to implement with Bevy's line rendering
- Each body stores history of positions for trail drawing
- `SunLight` component: already on DirectionalLight entity, ready for intensity/angle control
- `Planet` component: can be added for planet-specific features (rings, atmosphere)

### Multi-System Support (Feature C)
- `PhysicsState` can store multiple independent systems
- Each `BodyState` has optional `parent: Option<Entity>` for hierarchical systems
- Example: Earth has `parent = Some(sun_entity)`, Moon has `parent = Some(earth_entity)`
- UI can group bodies by system/hierarchy
- Camera can focus at any level (Sun → Earth → Moon)

## Success Criteria

1. ✅ Pause button works correctly
2. ✅ All dead code removed
3. ✅ No duplicate types remain
4. ✅ Entity references replace index mapping
5. ✅ Modules have clean boundaries
6. ✅ All existing features work (planets render, orbit correctly, UI controls work)
7. ✅ Architecture supports dynamic bodies, visual features, and multi-system support

## Next Steps

After this design is approved:
1. Invoke `writing-plans` skill to create detailed implementation plan
2. Execute implementation plan in incremental phases
3. Verify each phase with testing
4. Commit and push changes
