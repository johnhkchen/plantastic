use bevy::prelude::*;

use crate::bridge::{self, LoadSceneCommand, SetTierCommand};
use crate::picking::SelectedZone;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneState>()
            .add_systems(Update, handle_load_scene)
            .add_systems(Update, handle_set_tier.after(handle_load_scene))
            .add_systems(
                Update,
                track_scene_load
                    .after(handle_load_scene)
                    .after(handle_set_tier),
            );
    }
}

/// Marker component for the scene root so we can find and despawn it.
#[derive(Component)]
struct ViewerScene;

#[derive(Resource, Default)]
struct SceneState {
    /// Handle for the glTF currently being loaded (not yet spawned).
    pending_handle: Option<Handle<Gltf>>,
    /// Tier name of the pending scene (None for direct loadScene commands).
    pending_tier: Option<String>,
    /// Name of the currently displayed tier.
    current_tier: Option<String>,
    /// Whether the current/pending scene has been spawned.
    spawned: bool,
}

/// React to LoadSceneCommand messages from the bridge.
fn handle_load_scene(
    mut events: MessageReader<LoadSceneCommand>,
    mut state: ResMut<SceneState>,
    asset_server: Res<AssetServer>,
) {
    for cmd in events.read() {
        info!("Loading scene from: {}", cmd.url);
        let handle = asset_server.load(&cmd.url);
        state.pending_handle = Some(handle);
        state.pending_tier = None;
        state.spawned = false;
    }
}

/// React to SetTierCommand messages — load new scene but keep old visible.
fn handle_set_tier(
    mut events: MessageReader<SetTierCommand>,
    mut state: ResMut<SceneState>,
    asset_server: Res<AssetServer>,
) {
    for cmd in events.read() {
        info!("Switching to tier '{}' from: {}", cmd.tier, cmd.url);
        let handle = asset_server.load(&cmd.url);
        state.pending_handle = Some(handle);
        state.pending_tier = Some(cmd.tier.clone());
        state.spawned = false;
    }
}

/// Poll for glTF asset readiness and spawn the scene.
/// Uses keep-until-ready: old scene stays visible until new one is ready.
fn track_scene_load(
    mut commands: Commands,
    mut state: ResMut<SceneState>,
    gltf_assets: Res<Assets<Gltf>>,
    existing: Query<Entity, With<ViewerScene>>,
    mut selected: ResMut<SelectedZone>,
) {
    if state.spawned {
        return;
    }

    let Some(handle) = &state.pending_handle else {
        return;
    };

    let Some(gltf) = gltf_assets.get(handle) else {
        return; // Still loading — old scene remains visible
    };

    info!("glTF loaded — swapping scene");

    // Despawn old scene now that new one is ready
    for entity in &existing {
        commands.entity(entity).despawn();
    }

    // Clear selection (old entities are gone)
    *selected = SelectedZone::default();

    // Spawn new scene
    if let Some(default_scene) = &gltf.default_scene {
        commands.spawn((SceneRoot(default_scene.clone()), ViewerScene));
    } else if let Some(first_scene) = gltf.scenes.first() {
        commands.spawn((SceneRoot(first_scene.clone()), ViewerScene));
    } else {
        warn!("glTF has no scenes");
        bridge::send_error("glTF file contains no scenes");
        state.spawned = true;
        return;
    }

    // Update tier state
    if let Some(tier) = state.pending_tier.take() {
        state.current_tier = Some(tier.clone());
        bridge::send_tier_changed(&tier);
    }

    state.spawned = true;
    bridge::send_ready();
}
