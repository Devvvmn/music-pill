//! Styling and widget builders for the pill body, its hover state and the popup.

use cosmic::iced::core::{Radians, Rotation};
use cosmic::iced::{Alignment, Border, Color, Length, Shadow};
use cosmic::theme::{Button as ButtonStyle, Container as ContainerStyle};
use cosmic::widget;
use cosmic::widget::button::Style as ButtonAppearance;
use cosmic::Element;

use super::pill::{MusicPill, Message};
use crate::mpris::{player, Status};

/// A flat button style with a fixed background, border and corner radius.
fn flat_button(background: Color, border: Color, border_width: f32, radius: f32) -> ButtonStyle {
    let style = move |theme: &cosmic::Theme| {
        let on_bg: Color = theme.cosmic().on_bg_color().into();
        let mut appearance = ButtonAppearance::new();
        appearance.background = Some(background.into());
        appearance.border_radius = radius.into();
        appearance.border_width = border_width;
        appearance.border_color = border;
        appearance.icon_color = Some(on_bg);
        appearance.text_color = Some(on_bg);
        appearance
    };
    ButtonStyle::Custom {
        active: Box::new(move |_focused, theme| style(theme)),
        disabled: Box::new(move |theme| style(theme)),
        hovered: Box::new(move |_focused, theme| style(theme)),
        pressed: Box::new(move |_focused, theme| style(theme)),
    }
}

/// Translucent "glass" style for the pill body (a button).
pub fn pill_button_style() -> ButtonStyle {
    flat_button(
        Color::from_rgba(1.0, 1.0, 1.0, 0.12),
        Color::from_rgba(1.0, 1.0, 1.0, 0.35),
        1.0,
        100.0,
    )
}

/// Transparent style for the small control buttons.
fn control_button_style() -> ButtonStyle {
    flat_button(Color::TRANSPARENT, Color::TRANSPARENT, 0.0, 8.0)
}

/// Translucent rounded style for the album-art square.
fn art_style(radius: f32) -> ContainerStyle<'static> {
    ContainerStyle::custom(move |theme| {
        let on_bg: Color = theme.cosmic().on_bg_color().into();
        cosmic::iced::widget::container::Style {
            icon_color: Some(on_bg),
            text_color: Some(on_bg),
            background: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.1).into()),
            border: Border {
                width: 1.0,
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.25),
                radius: radius.into(),
            },
            shadow: Shadow::default(),
            snap: true,
        }
    })
}

/// Album art: a spinning disc in the pill, a rounded square in the popup.
/// Falls back to a music-disc placeholder when there is none.
pub fn art_widget(app: &MusicPill, size: u16) -> Element<'_, Message> {
    let disc = size <= 40;
    let px = Length::Fixed(f32::from(size));
    if let Some(art) = &app.art {
        let image = widget::image(if disc { art.disc.clone() } else { art.rounded.clone() })
            .width(px)
            .height(px);
        return if disc {
            image
                .rotation(Rotation::Floating(Radians(app.angle)))
                .into()
        } else {
            image.into()
        };
    }
    let radius = if disc { f32::from(size) / 2.0 } else { 14.0 };
    widget::container(widget::icon::from_name("media-optical-symbolic").size(size / 2))
        .class(art_style(radius))
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .width(px)
        .height(px)
        .into()
}

fn play_icon(status: Status) -> &'static str {
    match status {
        Status::Playing => "media-playback-pause-symbolic",
        _ => "media-playback-start-symbolic",
    }
}

fn volume_slider(app: &MusicPill) -> Element<'_, Message> {
    widget::Row::new()
        .spacing(6)
        .align_y(Alignment::Center)
        .push(widget::icon::from_name("audio-volume-high-symbolic").size(16))
        .push(
            widget::slider(0.0..=100.0, (app.info.volume * 100.0).round() as f32, |volume| {
                Message::Control(player::Control::Volume(f64::from(volume) / 100.0))
            })
            .width(Length::Fill),
        )
        .into()
}

/// The collapsed pill: a centered music icon, shown when nothing is playing.
pub fn idle_icon() -> Element<'static, Message> {
    widget::container(widget::icon::from_name("multimedia-audio-player-symbolic").size(26))
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// The compact pill content: art, title over artist, and transport buttons.
pub fn compact_row(app: &MusicPill) -> Element<'_, Message> {
    let title = if app.info.title.is_empty() {
        "No player".to_string()
    } else {
        truncate(&app.info.title, 14)
    };
    let artist = if app.info.artist.is_empty() {
        "—".to_string()
    } else {
        truncate(&app.info.artist, 14)
    };

    widget::Row::new()
        .spacing(10)
        .align_y(Alignment::Center)
        .push(art_widget(app, 30))
        .push(
            widget::Column::new()
                .width(Length::Fixed(90.0))
                .align_x(Alignment::Center)
                .push(widget::text::caption(artist))
                .push(widget::text::body(title).font(cosmic::font::bold())),
        )
        .push(transport(app, 2))
        .into()
}

/// Previous / play-pause / next.
fn transport(app: &MusicPill, spacing: u16) -> Element<'_, Message> {
    widget::Row::new()
        .spacing(spacing)
        .align_y(Alignment::Center)
        .push(control_button(
            "media-skip-backward-symbolic",
            player::Control::Previous,
            app.info.can_previous,
        ))
        .push(
            widget::button::custom(widget::icon::from_name(play_icon(app.info.status)).size(18))
                .class(flat_button(
                    Color::from_rgba(1.0, 1.0, 1.0, 0.25),
                    Color::TRANSPARENT,
                    0.0,
                    100.0,
                ))
                .padding(5)
                .on_press_maybe(
                    (app.info.can_play || app.info.can_pause)
                        .then_some(Message::Control(player::Control::PlayPause)),
                ),
        )
        .push(control_button(
            "media-skip-forward-symbolic",
            player::Control::Next,
            app.info.can_next,
        ))
        .into()
}

/// A small transparent icon button for the transport controls.
fn control_button(
    icon: &'static str,
    action: player::Control,
    enabled: bool,
) -> Element<'static, Message> {
    widget::button::custom(widget::icon::from_name(icon).size(18))
        .class(control_button_style())
        .padding(4)
        .on_press_maybe(enabled.then_some(Message::Control(action)))
        .into()
}

/// The expanded popup: artwork, track info, a seek slider, transport and volume.
pub fn popup_player(app: &MusicPill) -> Element<'_, Message> {
    let title = if app.info.title.is_empty() {
        "No active player".to_string()
    } else {
        truncate(&app.info.title, 26)
    };
    let artist = if app.info.artist.is_empty() {
        "—".to_string()
    } else {
        truncate(&app.info.artist, 32)
    };

    let position = app.info.position as f32 / 1_000_000.0;
    let duration = app.info.duration as f32 / 1_000_000.0;

    let play = widget::button::custom(widget::icon::from_name(play_icon(app.info.status)).size(26))
        .class(flat_button(
            Color::from_rgba(1.0, 1.0, 1.0, 0.22),
            Color::TRANSPARENT,
            0.0,
            100.0,
        ))
        .padding(10)
        .on_press_maybe(
            (app.info.can_play || app.info.can_pause)
                .then_some(Message::Control(player::Control::PlayPause)),
        );

    let controls = widget::Row::new()
        .spacing(18)
        .align_y(Alignment::Center)
        .push(control_button(
            "media-skip-backward-symbolic",
            player::Control::Previous,
            app.info.can_previous,
        ))
        .push(play)
        .push(control_button(
            "media-skip-forward-symbolic",
            player::Control::Next,
            app.info.can_next,
        ));

    widget::Column::new()
        .spacing(14)
        .padding(20)
        .width(Length::Fill)
        .push(
            widget::Row::new()
                .spacing(16)
                .align_y(Alignment::Center)
                .push(art_widget(app, 104))
                .push(
                    widget::Column::new()
                        .spacing(4)
                        .width(Length::Fill)
                        .push(widget::text::title4(title))
                        .push(widget::text::body(artist)),
                ),
        )
        .push(
            widget::Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(widget::text::caption(format_time(position)).width(Length::Fixed(36.0)))
                .push(
                    widget::slider(0.0..=duration.max(1.0), position.min(duration), |value| {
                        Message::Control(player::Control::Seek((value * 1_000_000.0) as u64))
                    })
                    .width(Length::Fill),
                )
                .push(widget::text::caption(format_time(duration)).width(Length::Fixed(36.0))),
        )
        .push(widget::container(controls).center_x(Length::Fill))
        .push(volume_slider(app))
        .into()
}

/// Truncate a string to `max_chars` characters, appending an ellipsis.
fn truncate(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_owned();
    }
    let mut kept: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    kept.push('…');
    kept
}

/// Format seconds as `M:SS`.
fn format_time(seconds: f32) -> String {
    let total = seconds.max(0.0) as u64;
    format!("{}:{:02}", total / 60, total % 60)
}
