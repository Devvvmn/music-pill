// Basic Cosmic Applet Template

use cosmic::app::{Command, Core};
use cosmic::widget::{button, column, container, row, text};
use cosmic::{Application, Element};

pub fn main() -> cosmic::Result {
    cosmic::app::run::<BasicApplet>(())
}

#[derive(Default)]
pub struct BasicApplet {
    core: Core,
    button_presses: u32,
}

#[derive(Clone, Copy, Debug)]
pub enum Message {
    Increment,
    Decrement,
}

impl Application for BasicApplet {
    type Executor = cosmic::executor::Default;
    type Message = Message;
    type Theme = cosmic::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                core: Core::default(),
                button_presses: 0,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Basic Applet")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Increment => self.button_presses += 1,
            Message::Decrement => self.button_presses = self.button_presses.saturating_sub(1),
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let content = column![
            text(format!("Presses: {}", self.button_presses)),
            row![
                button("−").on_press(Message::Decrement),
                button("+").on_press(Message::Increment),
            ]
        ];

        container(content).into()
    }
}
