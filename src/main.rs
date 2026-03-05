use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod app;
mod camera;
mod physics;
mod render;
mod ui;

use app::SolarSystemPlugin;

fn main() {
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
