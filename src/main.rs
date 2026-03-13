use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod app;
mod camera;
mod error;
mod physics;
mod render;
mod ui;

use app::SolarSystemPlugin;

fn main() {
    // Install panic hook for clearer error messages; keeps default behavior after logging.
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        eprintln!("Solar System Simulator panicked:");
        eprintln!("{info}");
        default_panic(info);
    }));

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "3D Solar System Simulator".to_string(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        // The setup system in SolarSystemPlugin now handles spawning and initialization
        .add_plugins(SolarSystemPlugin)
        .run();
}
