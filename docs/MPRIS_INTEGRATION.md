# MPRIS (D-Bus) Integration for Music Control

## What is MPRIS?

MPRIS (Media Player Remote Interfacing Specification) is a standard D-Bus interface for controlling media players.

**Main Interface:** `org.mpris.MediaPlayer2.Player`

### Key Methods
```
PlayPause()           # Toggle play/pause
Play()                # Start playing
Pause()               # Pause
Next()                # Next track
Previous()            # Previous track
Seek(offset)          # Seek to position (microseconds)
SetPosition(trackId, position)
```

### Properties
```
PlaybackStatus        # "Playing", "Paused", "Stopped"
Metadata              # Track info (title, artist, album, artUrl)
Position              # Current playback position
Volume                # 0.0 - 1.0
CanPlay / CanPause
CanNext / CanPrevious
```

## Rust Implementation

### Using `zbus` crate
```rust
use zbus::Connection;

#[async_std::main]
async fn main() -> Result<()> {
    let connection = Connection::session().await?;
    let proxy = connection
        .call_method(
            Some("org.mpris.MediaPlayer2.spotify"),
            "/org/mpris/MediaPlayer2",
            Some("org.mpris.MediaPlayer2.Player"),
            "PlayPause",
            &(),
        )
        .await?;
    
    Ok(())
}
```

### Get Current Track
```rust
let proxy = connection
    .call_method(
        Some("org.mpris.MediaPlayer2.spotify"),
        "/org/mpris/MediaPlayer2",
        Some("org.freedesktop.DBus.Properties"),
        "Get",
        &("org.mpris.MediaPlayer2.Player", "Metadata"),
    )
    .await?;
```

## Common Players

```
org.mpris.MediaPlayer2.spotify
org.mpris.MediaPlayer2.vlc
org.mpris.MediaPlayer2.audacious
org.mpris.MediaPlayer2.elisa
org.mpris.MediaPlayer2.rhythmbox
```

## Metadata Fields

```
xesam:title              # Track title
xesam:artist             # Artist name(s) [array]
xesam:album              # Album name
xesam:albumArtist        # Album artist
mpris:artUrl             # Album art URL
xesam:length             # Duration in microseconds
```

## Example: Music Pill State Update

```rust
pub struct MusicPillState {
    player_name: String,
    track_title: String,
    track_artist: String,
    playback_status: PlaybackStatus,
    position: u64,
    duration: u64,
    volume: f64,
}

#[derive(Clone, Debug)]
pub enum Message {
    UpdateMetadata(MusicPillState),
    Play,
    Pause,
    Next,
    Previous,
    VolumeChanged(f64),
}
```

## D-Bus Signal Monitoring

Listen to `PropertiesChanged` signals to update UI in real-time:

```rust
connection
    .match_rule(
        Some("type='signal',member='PropertiesChanged'")
    )
    .stream()
    .for_each(|msg| {
        // Handle property changes
        // Update local state
    })
```
