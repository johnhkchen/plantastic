use bevy::prelude::*;

use crate::bridge::{self, SetLightAngleCommand};

pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_lighting)
            .add_systems(Update, handle_light_angle);
    }
}

fn setup_lighting(mut commands: Commands) {
    // Directional light — sun-like, ~45 degree elevation
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::FRAC_PI_4, // -45 degrees pitch
            std::f32::consts::FRAC_PI_6,  // 30 degrees yaw (off-axis for depth)
            0.0,
        )),
    ));

    // Low global ambient fill light
    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 200.0,
        ..default()
    });

    // Report initial light angle to host (30° = π/6 radians default yaw)
    bridge::send_light_angle_changed(30.0);
}

/// Adjust the directional light yaw based on setLightAngle commands.
fn handle_light_angle(
    mut events: MessageReader<SetLightAngleCommand>,
    mut lights: Query<&mut Transform, With<DirectionalLight>>,
) {
    for cmd in events.read() {
        let yaw = cmd.degrees.to_radians();
        for mut transform in &mut lights {
            *transform = Transform::from_rotation(Quat::from_euler(
                EulerRot::XYZ,
                -std::f32::consts::FRAC_PI_4, // Keep -45° pitch
                yaw,
                0.0,
            ));
        }
        bridge::send_light_angle_changed(cmd.degrees);
    }
}
