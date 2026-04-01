use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin, TouchControls};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PanOrbitCameraPlugin)
            .add_systems(Startup, setup_camera);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        // Camera positioned to see a model at the origin
        Transform::from_xyz(3.0, 3.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera {
            // Mouse buttons
            button_orbit: MouseButton::Left,
            button_pan: MouseButton::Right,
            zoom_sensitivity: 0.5,
            // Pitch bounds: ~80° above to ~-3° below horizon
            pitch_upper_limit: Some(std::f32::consts::FRAC_PI_2 - 0.1),
            pitch_lower_limit: Some(-0.05),
            allow_upside_down: false,
            // Zoom bounds: don't clip through model or fly away
            zoom_upper_limit: Some(50.0),
            zoom_lower_limit: 0.5,
            // Smooth damping on all axes
            orbit_smoothness: 0.2,
            pan_smoothness: 0.1,
            zoom_smoothness: 0.15,
            // Touch: single finger orbit, two-finger pan, pinch zoom
            touch_enabled: true,
            touch_controls: TouchControls::OneFingerOrbit,
            ..default()
        },
        Camera3d::default(),
    ));
}
