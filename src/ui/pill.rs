//! The Music Pill application model.

use cosmic::app::{Core, Task};
use cosmic::iced::{window::Id, Length, Limits, Rectangle, Subscription};
use cosmic::surface::action::{app_popup, destroy_popup};
use cosmic::widget;
use cosmic::{applet, Application, Element};
use futures::StreamExt;
use std::time::{Duration, Instant};

use crate::mpris::{self, monitor, player, Status};
use crate::ui::controls;

/// Rest size of the pill window.
const PILL_HEIGHT: f32 = 40.0;
/// Diameter of the collapsed (idle) pill, matching the dock icons.
const IDLE_SIZE: f32 = 46.0;

/// Size limits for the expanded popup.
const POPUP_MAX_WIDTH: f32 = 360.0;
const POPUP_MIN_HEIGHT: f32 = 240.0;
const POPUP_MAX_HEIGHT: f32 = 360.0;

/// Messages of the pill.
#[derive(Clone, Debug)]
pub enum Message {
    /// A popup window was closed.
    PopupClosed(Id),
    /// An MPRIS update from the monitor task.
    Update(monitor::Update),
    /// A transport/volume control was pressed.
    Control(player::Control),
    /// A popup/surface request for libcosmic.
    Surface(cosmic::surface::Action<Message>),
    /// Animation frame for the spinning disc.
    Tick,
    /// A no-op (keeps fire-and-forget tasks well-formed).
    Noop,
}

/// The application state.
#[derive(Default)]
pub struct MusicPill {
    core: Core,
    /// The open popup window id, if any.
    popup: Option<Id>,
    /// The active player's state.
    last_control: Option<Instant>,
    pub(super) info: mpris::PlayerInfo,
    /// Album art for the current track.
    pub(super) art: Option<monitor::ArtSet>,
    /// Rotation of the spinning disc, in radians.
    pub(super) angle: f32,
}

impl MusicPill {
    /// Apply an MPRIS update to the state.
    fn apply_update(&mut self, update: monitor::Update) {
        match update {
            monitor::Update::State(info) => {
                let mut info = info.unwrap_or_default();
                // Keep the user's slider values while a drag/seek is settling,
                // otherwise the poll snaps the sliders back.
                if self.last_control.is_some_and(|t| t.elapsed() < Duration::from_millis(1500)) {
                    info.volume = self.info.volume;
                    info.position = self.info.position;
                }
                self.info = info;
            }
            monitor::Update::Art(art) => self.art = art,
        }
    }
}

impl Application for MusicPill {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "dev.cosmic.applet.music-pill";

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Message>) {
        (
            Self {
                core,
                ..Default::default()
            },
            Task::none(),
        )
    }

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn subscription(&self) -> Subscription<Message> {
        let monitor = Subscription::run(monitor_stream);
        if self.info.status == Status::Playing {
            // ~30 fps; one revolution every 6 seconds.
            Subscription::batch([
                monitor,
                cosmic::iced::time::every(Duration::from_millis(33)).map(|_| Message::Tick),
            ])
        } else {
            monitor
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PopupClosed(id) => {
                if self.popup == Some(id) {
                    self.popup = None;
                }
            }
            Message::Update(update) => self.apply_update(update),
            Message::Surface(action) => {
                return cosmic::task::message(cosmic::Action::Surface(action));
            }
            Message::Control(action) => {
                // Optimistic local update for instant feedback;
                // the next poll confirms the real state.
                self.last_control = Some(Instant::now());
                match action {
                    player::Control::Volume(volume) => self.info.volume = volume,
                    player::Control::Seek(position) => self.info.position = position,
                    player::Control::PlayPause => {
                        self.info.status = if self.info.status == Status::Playing {
                            Status::Paused
                        } else {
                            Status::Playing
                        };
                    }
                    _ => {}
                }
                let name = self.info.name.clone();
                return cosmic::task::future(async move { perform_control(&name, action).await });
            }
            Message::Tick => {
                self.angle = (self.angle + std::f32::consts::TAU / 6.0 / 30.0) % std::f32::consts::TAU;
            }
            Message::Noop => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        // Nothing playing: shrink to a round icon button.
        let idle = self.info.name.is_empty() || self.info.status == Status::Stopped;
        let content = if idle {
            controls::idle_icon()
        } else {
            controls::compact_row(self)
        };

        let have_popup = self.popup;
        let pill = widget::button::custom(content)
            .class(controls::pill_button_style())
            .padding(if idle { [0, 0] } else { [4, 12] })
            .height(Length::Fixed(if idle { IDLE_SIZE } else { PILL_HEIGHT }))
            .width(if idle { Length::Fixed(IDLE_SIZE) } else { Length::Shrink })
            .on_press_with_rectangle(move |offset, bounds| {
                if let Some(id) = have_popup {
                    Message::Surface(destroy_popup(id))
                } else {
                    Message::Surface(app_popup::<MusicPill>(
                        |_| Default::default(),
                        move |state: &mut MusicPill| {
                            let new_id = Id::unique();
                            state.popup = Some(new_id);
                            let mut settings = state.core.applet.get_popup_settings(
                                state.core.main_window_id().unwrap(),
                                new_id,
                                None,
                                None,
                                None,
                            );
                            settings.positioner.anchor_rect = Rectangle {
                                x: (bounds.x - offset.x) as i32,
                                y: (bounds.y - offset.y) as i32,
                                width: bounds.width as i32,
                                height: bounds.height as i32,
                            };
                            settings.positioner.size_limits = Limits::NONE
                                .max_width(POPUP_MAX_WIDTH)
                                .min_width(POPUP_MAX_WIDTH)
                                .min_height(POPUP_MIN_HEIGHT)
                                .max_height(POPUP_MAX_HEIGHT);
                            settings
                        },
                        Some(Box::new(move |state: &MusicPill| {
                            Element::from(
                                state.core.applet.popup_container(controls::popup_player(state)),
                            )
                            .map(cosmic::Action::App)
                        })),
                    ))
                }
            });

        self.core.applet.autosize_window(pill).into()
    }

    fn view_window(&self, _id: Id) -> Element<'_, Message> {
        "".into()
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(applet::style())
    }
}

/// Stream of MPRIS updates, wrapped as app messages.
fn monitor_stream() -> impl futures::Stream<Item = Message> {
    cosmic::iced::stream::channel(32, async |mut output: futures::channel::mpsc::Sender<Message>| {
        use futures::SinkExt;
        let (update_tx, mut update_rx) = futures::channel::mpsc::channel(32);
        tokio::spawn(monitor::run(update_tx));
        while let Some(update) = update_rx.next().await {
            if output.send(Message::Update(update)).await.is_err() {
                break;
            }
        }
        futures::future::pending::<()>().await;
    })
}

/// Send a control action to the active player over the session bus.
async fn perform_control(name: &str, action: player::Control) -> Message {
    if name.is_empty() {
        return Message::Noop;
    }
    static CONN: tokio::sync::OnceCell<zbus::Connection> = tokio::sync::OnceCell::const_new();
    match CONN.get_or_try_init(zbus::Connection::session).await {
        Ok(conn) => {
            if let Err(error) = player::control(conn, name, action).await {
                eprintln!("music-pill: control {action:?} failed: {error}");
            }
        }
        Err(error) => {
            eprintln!("music-pill: failed to connect to the session bus: {error}");
        }
    }
    Message::Noop
}
