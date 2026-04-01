use bevy::prelude::*;
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::VecDeque;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

/// Plugin that bridges postMessage ↔ Bevy messages.
///
/// The SvelteKit host sends JSON messages via `iframe.contentWindow.postMessage()`.
/// This plugin listens for those messages on `window`, queues them, and converts
/// them to typed Bevy messages each frame. Outbound messages go back via
/// `window.parent.postMessage()`.
pub struct BridgePlugin;

impl Plugin for BridgePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<LoadSceneCommand>()
            .add_message::<SetLightAngleCommand>()
            .add_message::<SetTierCommand>()
            .add_systems(Startup, setup_message_listener)
            .add_systems(PreUpdate, drain_messages);
    }
}

// -- Inbound messages (Host → Viewer) --

#[derive(Message)]
pub struct LoadSceneCommand {
    pub url: String,
}

#[derive(Message)]
pub struct SetLightAngleCommand {
    pub degrees: f32,
}

#[derive(Message)]
pub struct SetTierCommand {
    pub tier: String,
    pub url: String,
}

// -- Message queue (thread-local, WASM is single-threaded) --

thread_local! {
    static MESSAGE_QUEUE: RefCell<VecDeque<String>> = RefCell::new(VecDeque::new());
}

fn push_message(msg: String) {
    MESSAGE_QUEUE.with(|q| q.borrow_mut().push_back(msg));
}

fn drain_queue() -> Vec<String> {
    MESSAGE_QUEUE.with(|q| q.borrow_mut().drain(..).collect())
}

// -- Inbound JSON schema --

#[derive(Deserialize)]
struct InboundMessage {
    #[serde(rename = "type")]
    msg_type: String,
    url: Option<String>,
    degrees: Option<f32>,
    tier: Option<String>,
}

// -- Systems --

fn setup_message_listener() {
    let window = match web_sys::window() {
        Some(w) => w,
        None => {
            warn!("No window object — postMessage bridge disabled");
            return;
        }
    };

    let closure = Closure::<dyn Fn(web_sys::MessageEvent)>::new(move |event: web_sys::MessageEvent| {
        let data = event.data();
        if let Some(s) = data.as_string() {
            push_message(s);
        } else if let Ok(s) = js_sys::JSON::stringify(&data) {
            if let Some(s) = s.as_string() {
                push_message(s);
            }
        }
    });

    window
        .add_event_listener_with_callback("message", closure.as_ref().unchecked_ref())
        .expect("Failed to add message event listener");

    // Leak the closure so it lives for the lifetime of the app.
    closure.forget();

    info!("postMessage bridge: listener registered");
    send_ready();
}

fn drain_messages(
    mut load_scene: MessageWriter<LoadSceneCommand>,
    mut set_light: MessageWriter<SetLightAngleCommand>,
    mut set_tier: MessageWriter<SetTierCommand>,
) {
    for raw in drain_queue() {
        let msg: InboundMessage = match serde_json::from_str(&raw) {
            Ok(m) => m,
            Err(_) => continue, // Ignore non-JSON messages (browser extensions, etc.)
        };

        match msg.msg_type.as_str() {
            "loadScene" => {
                if let Some(url) = msg.url {
                    info!("Bridge: loadScene → {url}");
                    load_scene.write(LoadSceneCommand { url });
                }
            }
            "setLightAngle" => {
                if let Some(degrees) = msg.degrees {
                    info!("Bridge: setLightAngle → {degrees}°");
                    set_light.write(SetLightAngleCommand { degrees });
                }
            }
            "setTier" => {
                if let (Some(tier), Some(url)) = (msg.tier, msg.url) {
                    info!("Bridge: setTier → {tier} ({url})");
                    set_tier.write(SetTierCommand { tier, url });
                }
            }
            _ => {
                debug!("Bridge: ignoring unknown message type '{}'", msg.msg_type);
            }
        }
    }
}

// -- Outbound helpers (Viewer → Host) --

fn post_to_parent(json: &str) {
    let Some(window) = web_sys::window() else {
        return;
    };
    // If we're in an iframe, parent != self.
    let parent = match window.parent() {
        Ok(Some(p)) => p,
        _ => return, // Not in an iframe — no parent to message.
    };
    let msg = js_sys::JSON::parse(json).unwrap_or(wasm_bindgen::JsValue::from_str(json));
    let _ = parent.post_message(&msg, "*");
}

pub fn send_ready() {
    post_to_parent(r#"{"type":"ready"}"#);
    info!("Bridge: sent ready");
}

pub fn send_error(message: &str) {
    let json = serde_json::json!({ "type": "error", "message": message }).to_string();
    post_to_parent(&json);
    warn!("Bridge: sent error — {message}");
}

pub fn send_zone_tapped(zone_id: &str) {
    let json = serde_json::json!({ "type": "zoneTapped", "zoneId": zone_id }).to_string();
    post_to_parent(&json);
    info!("Bridge: sent zoneTapped — {zone_id}");
}

pub fn send_light_angle_changed(degrees: f32) {
    let json = serde_json::json!({ "type": "lightAngleChanged", "degrees": degrees }).to_string();
    post_to_parent(&json);
}

pub fn send_tier_changed(tier: &str) {
    let json = serde_json::json!({ "type": "tierChanged", "tier": tier }).to_string();
    post_to_parent(&json);
    info!("Bridge: sent tierChanged — {tier}");
}
