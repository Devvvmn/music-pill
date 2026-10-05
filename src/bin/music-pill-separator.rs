// A thin vertical separator applet for the COSMIC panel, meant to sit
// between the Music Pill and the dock icons.

use cosmic::app::{Core, Task};
use cosmic::iced::{Border, Color, Length};
use cosmic::theme::Container as ContainerStyle;
use cosmic::{applet, widget, Application, Element};

struct Separator {
    core: Core,
}

impl Application for Separator {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = ();
    const APP_ID: &'static str = "dev.cosmic.applet.music-pill-separator";

    fn init(core: Core, _flags: ()) -> (Self, Task<()>) {
        (Self { core }, Task::none())
    }

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn view(&self) -> Element<'_, ()> {
        let line = widget::container(widget::Space::new().width(Length::Fixed(1.0)).height(Length::Fixed(46.0)))
            .class(ContainerStyle::custom(|_| cosmic::iced::widget::container::Style {
                background: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.7).into()),
                border: Border {
                    radius: 1.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }));
        let slot = widget::container(line)
            .padding([0, 10])
            .height(Length::Fixed(48.0))
            .align_y(cosmic::iced::Alignment::Center);
        self.core.applet.autosize_window(slot).into()
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(applet::style())
    }
}

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<Separator>(())
}
