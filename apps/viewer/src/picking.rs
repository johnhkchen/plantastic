use bevy::picking::events::Click;
use bevy::picking::prelude::Pointer;
use bevy::prelude::*;

use crate::bridge;

/// Plugin that detects mesh taps, highlights the selected zone, and notifies
/// the host via postMessage.
pub struct PickingSetupPlugin;

impl Plugin for PickingSetupPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedZone>()
            .add_systems(Update, handle_click_messages);
    }
}

/// Tracks which zone entity is currently selected and its original material
/// so we can restore it on deselection.
#[derive(Resource, Default)]
pub struct SelectedZone {
    entity: Option<Entity>,
    original_material: Option<Handle<StandardMaterial>>,
}

/// Highlight color: warm golden emissive overlay (HDR values for glow).
const HIGHLIGHT_EMISSIVE: Color = Color::srgb(1.0, 0.8, 0.2);

/// Read Pointer<Click> messages to detect taps on entities with Name components.
/// Highlights tapped zones and sends zoneTapped to the host.
fn handle_click_messages(
    mut clicks: MessageReader<Pointer<Click>>,
    names: Query<&Name>,
    materials_query: Query<&MeshMaterial3d<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut selected: ResMut<SelectedZone>,
) {
    for click in clicks.read() {
        let entity = click.entity;

        if let Ok(name) = names.get(entity) {
            let zone_id = name.as_str();

            // Skip if already selected
            if selected.entity == Some(entity) {
                continue;
            }

            // Deselect previous
            deselect(&mut selected, &materials_query, &mut materials);

            // Save original material and apply highlight
            if let Ok(mat_handle) = materials_query.get(entity) {
                // Clone the original material first to avoid borrow conflict
                let original_clone = materials.get(&mat_handle.0).cloned();
                if let Some(original) = original_clone {
                    selected.original_material = Some(materials.add(original));

                    // Apply highlight emissive in-place
                    if let Some(mat) = materials.get_mut(&mat_handle.0) {
                        mat.emissive = HIGHLIGHT_EMISSIVE.into();
                    }
                }
            }

            selected.entity = Some(entity);
            bridge::send_zone_tapped(zone_id);
        } else {
            // Tapped empty space or unnamed mesh — deselect
            debug!("Tapped entity {:?} with no Name component", entity);
            deselect(&mut selected, &materials_query, &mut materials);
        }
    }
}

/// Restore the previously selected entity's original material.
fn deselect(
    selected: &mut ResMut<SelectedZone>,
    materials_query: &Query<&MeshMaterial3d<StandardMaterial>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    if let Some(prev_entity) = selected.entity {
        if let (Ok(mat_handle), Some(original_handle)) =
            (materials_query.get(prev_entity), &selected.original_material)
        {
            if let Some(original) = materials.get(original_handle) {
                let restored_emissive = original.emissive;
                if let Some(mat) = materials.get_mut(&mat_handle.0) {
                    mat.emissive = restored_emissive;
                }
            }
        }
    }
    selected.entity = None;
    selected.original_material = None;
}
