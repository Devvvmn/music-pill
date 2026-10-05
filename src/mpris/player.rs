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
/// Set once a player ignores `Volume` writes; volume then follows the player's own audio streams.
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
        app_volume(name).await.unwrap_or(info.volume.max(1.0))
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
            set_app_volume(name, volume).await;
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

/// A PipeWire/PulseAudio playback stream belonging to a player.
struct Stream {
    id: u32,
    /// Volume in 0.0..=1.0.
    volume: f64,
}

/// Find the playback streams of the player behind an MPRIS bus name,
/// e.g. `org.mpris.MediaPlayer2.brave.instance2` -> streams of the `brave` app.
async fn player_streams(bus_name: &str) -> Vec<Stream> {
    let key = bus_name
        .strip_prefix(MPRIS_PREFIX)
        .and_then(|rest| rest.split('.').next())
        .unwrap_or_default()
        .to_lowercase();
    if key.is_empty() {
        return Vec::new();
    }
    let Ok(out) = tokio::process::Command::new("pactl")
        .args(["list", "sink-inputs"])
        .env("LC_ALL", "C")
        .output()
        .await
    else {
        return Vec::new();
    };
    let text = String::from_utf8_lossy(&out.stdout);

    let mut streams = Vec::new();
    for block in text.split("Sink Input #").skip(1) {
        let Some(id) = block.lines().next().and_then(|l| l.trim().parse().ok()) else {
            continue;
        };
        let lower = block.to_lowercase();
        let belongs = ["application.name", "application.process.binary"].iter().any(|prop| {
            lower
                .lines()
                .any(|l| l.trim_start().starts_with(prop) && l.contains(&key))
        });
        let volume = block
            .lines()
            .find(|l| l.trim_start().starts_with("Volume:"))
            .and_then(|l| l.split('/').nth(1))
            .and_then(|pct| pct.trim().trim_end_matches('%').parse::<f64>().ok());
        if let (true, Some(volume)) = (belongs, volume) {
            streams.push(Stream { id, volume: volume / 100.0 });
        }
    }
    streams
}

async fn app_volume(bus_name: &str) -> Option<f64> {
    player_streams(bus_name).await.first().map(|s| s.volume)
}

/// Set the volume of the player's own audio streams (not the system volume).
async fn set_app_volume(bus_name: &str, volume: f64) {
    let percent = format!("{}%", (volume.clamp(0.0, 1.0) * 100.0).round());
    for stream in player_streams(bus_name).await {
        let _ = tokio::process::Command::new("pactl")
            .args(["set-sink-input-volume", &stream.id.to_string(), &percent])
            .status()
            .await;
    }
}
