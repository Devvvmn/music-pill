# ICE / libcosmic Framework Documentation

## What is libcosmic?

libcosmic is a GUI toolkit for creating COSMIC-themed applets and applications built on the **iced** library. It provides components and styling consistent with the COSMIC Desktop Environment.

**Official Links:**
- GitHub: https://github.com/pop-os/libcosmic
- API Docs: https://pop-os.github.io/libcosmic/cosmic/
- Book/Guide: https://pop-os.github.io/libcosmic-book/
- App Template: https://github.com/pop-os/cosmic-app-template
- Applet Template: https://github.com/pop-os/cosmic-applet-template

## Key Concepts

### 1. Architecture (MVU Pattern)
- **Model**: Application state
- **View**: Render UI based on state
- **Update**: Handle messages and update state

### 2. Main Components
- `cosmic::Application` - Main app trait
- `cosmic::Element` - UI element wrapper
- `cosmic::Command` - Async actions
- `cosmic::Message` - Event messages

### 3. Built-in Widgets
- `cosmic::widget::container` - Container with styling
- `cosmic::widget::row` / `cosmic::widget::column` - Layouts
- `cosmic::widget::button` - Styled button
- `cosmic::widget::text` - Text display
- `cosmic::widget::slider` - Value slider
- `cosmic::widget::progress_bar` - Progress indicator

### 4. Styling System
- Theme-aware colors
- Built-in spacing/padding
- Glassmorphism support
- Custom stylesheets via `cosmic::iced::theme`

## Panel Applets (Cosmic Applets)

### Structure
```
cosmic-applet-name/
├── Cargo.toml
├── src/
│   └── main.rs
├── res/
│   └── icons/
└── i18n/
```

### Basic Template
```rust
use cosmic::app::{Command, Core};
use cosmic::{Application, Element};

pub fn main() -> cosmic::Result {
    cosmic::app::run::<MusicPill>(())
}

pub struct MusicPill {
    core: Core,
}

#[derive(Clone, Debug)]
pub enum Message {
    Play,
    Pause,
    Next,
    Previous,
}

impl Application for MusicPill {
    type Executor = cosmic::executor::Default;
    type Message = Message;
    type Theme = cosmic::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        let mut core = Core::default();
        (Self { core }, Command::none())
    }

    fn title(&self) -> String {
        String::from("Music Pill")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        // Return UI
        cosmic::widget::container("Music").into()
    }
}
```

## Resources
- libcosmic examples: https://github.com/pop-os/libcosmic/tree/main/examples
- iced book: https://book.iced.rs/
- Cosmic Design: https://github.com/pop-os/cosmic-design
