// TODO(T-031-01): Remove this once scene.rs wires up mesh.rs and glb.rs
#![allow(dead_code)]
//! 3D scene generation from project zones and material assignments.
//!
//! Converts zone polygons + material assignments into glTF 2.0 binary (.glb)
//! scenes for the Bevy viewer. Each zone becomes a named mesh node that the
//! viewer can pick (tap-to-inspect).

pub mod error;
pub(crate) mod glb;
pub(crate) mod mesh;
pub mod scene;

pub use error::SceneError;
pub use scene::{generate_scene, SceneMetadata, SceneOutput};
