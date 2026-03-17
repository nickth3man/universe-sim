//! Orbit trail rendering: time-based position history with faded gradient.

use crate::error::validate_positive_or;
use crate::physics::system::PhysicsState;
use crate::types::AU_TO_WORLD;
use bevy::prelude::*;
use tracing::info_span;
use std::collections::HashMap;

/// Default trail length in simulation days when enabled.
pub const DEFAULT_TRAIL_LENGTH_DAYS: f64 = 30.0;

/// Single sample in a body's trail history.
#[derive(Clone, Copy)]
pub(crate) struct TrailPoint {
    position: Vec3,
    elapsed_days: f64,
}

/// Per-body trail history. Pruned by simulation time.
#[derive(Resource, Default)]
pub struct TrailState {
    /// Trail samples per entity. Only bodies with orbits are stored.
    history: HashMap<Entity, Vec<TrailPoint>>,
}

impl TrailState {
    /// Record current positions and prune old samples.
    pub fn update(&mut self, physics: &PhysicsState, trail_length_days: f64) {
        let elapsed = physics.elapsed_days;
        let trail_length_days =
            validate_positive_or(trail_length_days, DEFAULT_TRAIL_LENGTH_DAYS, "trail_length_days");
        let cutoff = elapsed - trail_length_days;

        for body in physics.bodies.values() {
            let Some(orbit) = body.orbit.as_ref() else {
                continue; // Sun has no trail
            };
            if !orbit.is_valid() {
                continue;
            }

            let pos = body.position;
            // Skip non-finite positions to avoid corrupting trail geometry
            if !pos.x.is_finite() || !pos.y.is_finite() || !pos.z.is_finite() {
                continue;
            }

            let world_pos = Vec3::new(
                pos.x as f32 * AU_TO_WORLD,
                pos.y as f32 * AU_TO_WORLD,
                pos.z as f32 * AU_TO_WORLD,
            );

            let entry = self.history.entry(body.entity).or_default();
            entry.push(TrailPoint {
                position: world_pos,
                elapsed_days: elapsed,
            });

            // Prune points older than trail_length_days
            entry.retain(|p| p.elapsed_days >= cutoff);
        }
    }

    /// Iterate over all trails (entity, points).
    pub(crate) fn iter_trails(&self) -> impl Iterator<Item = (Entity, &[TrailPoint])> {
        self.history
            .iter()
            .filter_map(|(e, v)| if v.len() >= 2 { Some((*e, v.as_slice())) } else { None })
    }
}

/// Renders orbit trails using Gizmos with a faded gradient (opaque at body, transparent at tail).
pub fn render_orbit_trails_system(
    mut gizmos: Gizmos,
    trail_state: Res<TrailState>,
    trail_config: Res<TrailConfig>,
) {
    let _span = info_span!("render_orbit_trails_system").entered();
    if !trail_config.enabled {
        return;
    }

    for (_entity, points) in trail_state.iter_trails() {
        let n = points.len();
        if n < 2 {
            continue;
        }

        // Gradient: oldest (tail) = transparent, newest (head) = opaque
        let gradient_points: Vec<(Vec3, bevy::color::Color)> = points
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let t = (i as f32 + 1.0) / n as f32; // 0..1 from tail to head
                let alpha = t * 0.3 + 0.1; // min 0.1 at tail, max 0.4 at head (subtle fade)
                (p.position, Color::srgba(1.0, 1.0, 1.0, alpha))
            })
            .collect();

        gizmos.linestrip_gradient(gradient_points);
    }
}

/// Configuration for orbit trails (UI-driven).
#[derive(Resource, Clone)]
pub struct TrailConfig {
    pub enabled: bool,
    pub length_days: f64,
}

impl Default for TrailConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            length_days: DEFAULT_TRAIL_LENGTH_DAYS,
        }
    }
}

/// Bevy system: samples current positions into trail history and prunes old points.
pub fn trail_update_system(
    physics: Res<PhysicsState>,
    trail_config: Res<TrailConfig>,
    mut trail_state: ResMut<TrailState>,
) {
    let _span = info_span!("trail_update_system").entered();
    let length_days = validate_positive_or(
        trail_config.length_days,
        DEFAULT_TRAIL_LENGTH_DAYS,
        "trail_config.length_days",
    );
    trail_state.update(&physics, length_days);
}
