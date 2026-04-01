use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

mod bridge;
mod camera;
mod lighting;
mod picking;
mod scene;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        canvas: Some("#bevy-canvas".to_string()),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: true,
                        title: "Plantastic Viewer".to_string(),
                        ..default()
                    }),
                    ..default()
                })
                .set(bevy::log::LogPlugin {
                    level: bevy::log::Level::INFO,
                    ..default()
                }),
        )
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(bridge::BridgePlugin)
        .add_plugins(camera::CameraPlugin)
        .add_plugins(scene::ScenePlugin)
        .add_plugins(lighting::LightingPlugin)
        .add_plugins(picking::PickingSetupPlugin)
        .run();
}
