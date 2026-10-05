//! MPRIS (D-Bus) integration: player discovery, state monitoring and control.
//!
//! MPRIS (`org.mpris.MediaPlayer2.Player`) is the standard D-Bus interface
//! for media players. See docs/MPRIS_INTEGRATION.md for the spec overview.

pub mod monitor;
pub mod player;

/// Playback status as reported by MPRIS.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Status {
    Playing,
    Paused,
    #[default]
    Stopped,
}

impl From<&str> for Status {
    fn from(value: &str) -> Self {
        match value {
            "Playing" => Self::Playing,
            "Paused" => Self::Paused,
            _ => Self::Stopped,
        }
    }
}

/// Everything the pill needs to render the current track.
#[derive(Clone, Debug, Default)]
pub struct PlayerInfo {
    /// D-Bus name of the player, e.g. `org.mpris.MediaPlayer2.spotify`.
    pub name: String,
    /// Track title (`xesam:title`).
    pub title: String,
    /// Artist name(s) (`xesam:artist`).
    pub artist: String,
    /// Album name (`xesam:album`).
    pub album: String,
    pub status: Status,
    /// Current playback position in microseconds.
    pub position: u64,
    /// Track duration in microseconds (`xesam:length`).
    pub duration: u64,
    /// Volume in 0.0..=1.0.
    pub volume: f64,
    /// MPRIS capability flags.
    pub can_next: bool,
    pub can_previous: bool,
    pub can_play: bool,
    pub can_pause: bool,
    /// Album art URL (`mpris:artUrl`); may be empty.
    pub art_url: String,
}
