use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod app;
mod camera;
mod error;
mod physics;
mod render;
mod ui;

use app::SolarSystemPlugin;

fn main() {
    // Structured logging: RUST_LOG env filter (e.g. RUST_LOG=info, RUST_LOG=debug, universe_sim=debug)
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer())
        .init();

    // Install panic hook for clearer error messages and recovery hints.
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        eprintln!("═══════════════════════════════════════════════════════════");
        eprintln!("  Solar System Simulator — Unexpected Error");
        eprintln!("═══════════════════════════════════════════════════════════");
        eprintln!("{info}");
        eprintln!("───────────────────────────────────────────────────────────");
        eprintln!("If this persists, try: updating GPU drivers, disabling");
        eprintln!("overlays, or running with minimal plugins for debugging.");
        eprintln!("═══════════════════════════════════════════════════════════");
        default_panic(info);
    }));

    App::new()
        .add_plugins(
            DefaultPlugins
                .build()
                .disable::<LogPlugin>()
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "3D Solar System Simulator".to_string(),
                        resolution: (1280, 720).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(EguiPlugin::default())
        // The setup system in SolarSystemPlugin now handles spawning and initialization
        .add_plugins(SolarSystemPlugin)
        .run();
}
