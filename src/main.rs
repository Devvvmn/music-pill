// Music Pill — a macOS-style music pill applet for the COSMIC panel.
//
// The applet shows a compact, glassy pill in the panel with the current
// track. The pill has built-in playback controls, and clicking it
// opens an expanded popup player (artwork, seek bar, volume).
//
// Playback data is read from MPRIS (D-Bus), so any player that implements
// the MPRIS spec (Spotify, VLC, Audacious, Elisa, …) is supported.

mod mpris;
mod ui;

fn main() -> cosmic::iced::Result {
    // Starts the applet's event loop with `()` as the application's flags.
    cosmic::applet::run::<ui::MusicPill>(())
}
