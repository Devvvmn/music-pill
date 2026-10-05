//! MPRIS player discovery, state reads and control calls.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

use zbus::{
    zvariant::{OwnedValue, Value},
    Connection, Proxy,
};

use super::{PlayerInfo, Status};

/// A control action to send to the active player.
#[derive(Clone, Copy, Debug)]
pub enum Control {
    PlayPause,
    /// Seek to an absolute position in microseconds.
    Seek(u64),
    /// Set the volume in 0.0..=1.0.
    Volume(f64),
    Next,
    Previous,
}

/// D-Bus name prefix of all MPRIS players.
pub const MPRIS_PREFIX: &str = "org.mpris.MediaPlayer2.";
/// Set once a player ignores `Volume` writes; volume then follows the system sink.
static SYSTEM_VOLUME: AtomicBool = AtomicBool::new(false);

const MPRIS_PATH: &str = "/org/mpris/MediaPlayer2";
const MPRIS_INTERFACE: &str = "org.mpris.MediaPlayer2.Player";

async fn player_proxy<'a>(conn: &Connection, name: &'a str) -> zbus::Result<Proxy<'a>> {
    Proxy::new(conn, name, MPRIS_PATH, MPRIS_INTERFACE).await
}

/// List the D-Bus names of all running MPRIS players.
pub async fn find_players(conn: &Connection) -> Vec<String> {
    let Ok(dbus) = zbus::fdo::DBusProxy::new(conn).await else {
        return Vec::new();
    };
    dbus.list_names()
        .await
        .map(|names| {
            names
                .into_iter()
                .map(|name| name.to_string())
                .filter(|name| name.starts_with(MPRIS_PREFIX))
                .collect()
        })
        .unwrap_or_default()
}

/// Pick the "active" player: the first one that is playing,
/// falling back to the first player in the list.
pub async fn pick_active(conn: &Connection, players: &[String]) -> Option<String> {
    for name in players {
        if let Ok(proxy) = player_proxy(conn, name).await {
            if let Ok(status) = proxy.get_property::<String>("PlaybackStatus").await {
                if status == "Playing" {
                    return Some(name.clone());
                }
            }
        }
    }
    players.first().cloned()
}

/// Read the full state of a player.
pub async fn read_info(conn: &Connection, name: &str) -> zbus::Result<PlayerInfo> {
    let proxy = player_proxy(conn, name).await?;

    let mut info = PlayerInfo::default();
    if let Ok(metadata) = proxy
        .get_property::<HashMap<String, OwnedValue>>("Metadata")
        .await
    {
        info = parse_metadata(&metadata);
    }
    info.name = name.to_owned();
    if let Ok(status) = proxy.get_property::<String>("PlaybackStatus").await {
        info.status = Status::from(status.as_str());
    }
    info.position = proxy
        .get_property::<i64>("Position")
        .await
        .map(|p| p.max(0) as u64)
        .unwrap_or(0);
    info.volume = if SYSTEM_VOLUME.load(Ordering::Relaxed) {
        system_volume().await.unwrap_or(1.0)
    } else {
        proxy.get_property::<f64>("Volume").await.unwrap_or(0.0)
    };
    info.can_next = proxy.get_property::<bool>("CanGoNext").await.unwrap_or(false);
    info.can_previous = proxy.get_property::<bool>("CanGoPrevious").await.unwrap_or(false);
    info.can_play = proxy.get_property::<bool>("CanPlay").await.unwrap_or(false);
    info.can_pause = proxy.get_property::<bool>("CanPause").await.unwrap_or(false);
    Ok(info)
}

/// Send a control action to the player.
pub async fn control(conn: &Connection, name: &str, action: Control) -> zbus::Result<()> {
    let proxy = player_proxy(conn, name).await?;
    match action {
        Control::PlayPause => proxy.call_method("PlayPause", &()).await.map(drop),
        Control::Next => proxy.call_method("Next", &()).await.map(drop),
        Control::Previous => proxy.call_method("Previous", &()).await.map(drop),
        // `SetPosition` needs a track id; `Seek` takes a relative offset, which always works.
        Control::Seek(target) => {
            let current = proxy.get_property::<i64>("Position").await.unwrap_or(0);
            proxy
                .call_method("Seek", &(target as i64 - current))
                .await
                .map(drop)
        }
        Control::Volume(volume) => {
            if !SYSTEM_VOLUME.load(Ordering::Relaxed) {
                let _ = proxy.set_property("Volume", volume).await;
                // Some players (e.g. browsers) accept the write but ignore it.
                let applied = proxy.get_property::<f64>("Volume").await.unwrap_or(volume);
                if (applied - volume).abs() < 0.02 {
                    return Ok(());
                }
                SYSTEM_VOLUME.store(true, Ordering::Relaxed);
            }
            set_system_volume(volume).await;
            Ok(())
        }
    }
}

/// Parse the MPRIS `Metadata` dictionary into track info.
fn parse_metadata(metadata: &HashMap<String, OwnedValue>) -> PlayerInfo {
    let mut info = PlayerInfo::default();
    let string = |key: &str| -> Option<String> {
        metadata
            .get(key)
            .and_then(|v| String::try_from(v.try_clone().ok()?).ok())
    };

    info.title = string("xesam:title").unwrap_or_default();
    info.album = string("xesam:album").unwrap_or_default();
    info.art_url = string("mpris:artUrl").unwrap_or_default();
    info.artist = metadata
        .get("xesam:artist")
        .and_then(|v| Vec::<String>::try_from(v.try_clone().ok()?).ok())
        .map(|names| names.join(", "))
        .or_else(|| string("xesam:artist"))
        .unwrap_or_default();
    info.duration = metadata
        // The spec key is `mpris:length`; some players only send `xesam:length`.
        .get("mpris:length")
        .or_else(|| metadata.get("xesam:length"))
        .and_then(|v| match &**v {
            Value::I64(n) => Some((*n).max(0) as u64),
            Value::U64(n) => Some(*n),
            Value::I32(n) => Some((*n).max(0) as u64),
            Value::U32(n) => Some(u64::from(*n)),
            _ => None,
        })
        .unwrap_or(0);
    info
}

/// Read the default sink volume via WirePlumber (0.0..=1.0).
async fn system_volume() -> Option<f64> {
    let out = tokio::process::Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
        .output()
        .await
        .ok()?;
    String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .nth(1)?
        .parse()
        .ok()
}

async fn set_system_volume(volume: f64) {
    let _ = tokio::process::Command::new("wpctl")
        .args(["set-volume", "@DEFAULT_AUDIO_SINK@", &format!("{:.2}", volume.clamp(0.0, 1.0))])
        .status()
        .await;
}
