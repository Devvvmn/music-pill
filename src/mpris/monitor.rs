//! MPRIS state monitoring: polls the active player on the session bus
//! and pushes snapshots into the UI.

use std::time::Duration;

use cosmic::widget::image::Handle;
use futures::{channel::mpsc, SinkExt};
use zbus::Connection;

use super::player::{find_players, pick_active, read_info};
use super::PlayerInfo;

/// Album art prepared for the two places it is shown.
#[derive(Clone, Debug)]
pub struct ArtSet {
    /// Square crop with rounded corners (popup).
    pub rounded: Handle,
    /// Circular crop (spinning disc in the pill).
    pub disc: Handle,
}

/// Updates flowing from the MPRIS world into the UI.
#[derive(Clone, Debug)]
pub enum Update {
    /// A full state snapshot of the active player (`None` when no player runs).
    State(Option<PlayerInfo>),
    /// Album art was fetched (or cleared).
    Art(Option<ArtSet>),
}

/// Run the MPRIS monitor until the UI goes away.
pub async fn run(mut tx: mpsc::Sender<Update>) {
    let conn = match Connection::session().await {
        Ok(conn) => conn,
        Err(error) => {
            eprintln!("music-pill: could not connect to the session D-Bus: {error}");
            return;
        }
    };

    let mut interval = tokio::time::interval(Duration::from_secs(1));
    let mut art_url = String::new();

    loop {
        interval.tick().await;

        let players = find_players(&conn).await;
        let info = match pick_active(&conn, &players).await {
            Some(name) => read_info(&conn, &name).await.ok(),
            None => None,
        };

        let new_art_url = info.as_ref().map(|i| i.art_url.clone()).unwrap_or_default();
        if tx.send(Update::State(info)).await.is_err() {
            return;
        }

        if new_art_url != art_url {
            art_url = new_art_url;
            let art = if art_url.is_empty() {
                None
            } else {
                fetch_art(&art_url).await.ok()
            };
            if tx.send(Update::Art(art)).await.is_err() {
                return;
            }
        }
    }
}

/// Fetch album art from an `mpris:artUrl` and decode it into an image handle.
pub async fn fetch_art(url: &str) -> Result<ArtSet, String> {
    let bytes: Vec<u8> = if let Some(path) = url.strip_prefix("file://") {
        std::fs::read(path).map_err(|error| format!("failed to read {path}: {error}"))?
    } else if let Some((header, encoded)) = url.split_once(";base64,").filter(|_| url.starts_with("data:")) {
        let _ = header;
        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .map_err(|error| format!("failed to decode art data: {error}"))?
    } else if url.starts_with("http://") || url.starts_with("https://") {
        reqwest::get(url)
            .await
            .map_err(|error| format!("failed to fetch art: {error}"))?
            .bytes()
            .await
            .map_err(|error| format!("failed to read art: {error}"))?
            .to_vec()
    } else {
        return Err(format!("unsupported art URL scheme: {url}"));
    };

    let decoded = image::load_from_memory(&bytes)
        .map_err(|error| format!("failed to decode art: {error}"))?;

    // Center-crop to a square so the art fills its frame, then downscale.
    const SIZE: u32 = 192;
    let side = decoded.width().min(decoded.height());
    let square = decoded
        .crop_imm((decoded.width() - side) / 2, (decoded.height() - side) / 2, side, side)
        .resize_exact(SIZE, SIZE, image::imageops::FilterType::Triangle)
        .to_rgba8();

    let mask = |inside: &dyn Fn(f32, f32) -> bool| {
        let mut img = square.clone();
        for (x, y, px) in img.enumerate_pixels_mut() {
            if !inside(x as f32 + 0.5, y as f32 + 0.5) {
                px.0 = [0, 0, 0, 0];
            }
        }
        Handle::from_rgba(SIZE, SIZE, img.into_raw())
    };

    let half = SIZE as f32 / 2.0;
    let corner = SIZE as f32 * 0.14;
    Ok(ArtSet {
        disc: mask(&|x, y| (x - half).hypot(y - half) <= half),
        rounded: mask(&|x, y| {
            // Distance from the nearest corner-circle center, if in a corner zone.
            let cx = x.clamp(corner, SIZE as f32 - corner);
            let cy = y.clamp(corner, SIZE as f32 - corner);
            (x - cx).hypot(y - cy) <= corner
        }),
    })
}
