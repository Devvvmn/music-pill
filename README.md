# Music Pill — applet-плеер для Cosmic

Компактный «пилюля»-плеер в стиле macOS Music Pill: маленькая полупрозрачная
плашка в dock, которая показывает текущий трек и управляет воспроизведением
через **MPRIS** (D-Bus `org.mpris.MediaPlayer2`).

## Возможности

- Компактная пилюля 180×60 px: обложка трека, название, исполнитель и
  тонкий прогресс-бар.
- При наведении курсора пилюля «расширяется»: появляются кнопки
  «Назад / Пуск-Пауза / Вперёд» и слайдер громкости.
- Клик по пилюле открывает всплывающее окно (popup): большая обложка,
  название/исполнитель, слайдер перемотки с временными метками, транспорт
  и громкость.
- Полупрозрачный «стеклянный» фон (glassmorphism) и скруглённые углы.
- MPRIS: отслеживание `PropertiesChanged`/`NameOwnerChanged`, выбор
  активного плеера, управление Play/Pause/Next/Previous/Seek/Volume,
  загрузка обложки (`mpris:artUrl`: `file://`, `data:` и `http(s)://`).

## Структура

```
src/main.rs            — точка входа: cosmic::applet::run::<MusicPill>(())
src/ui/pill.rs         — модель приложения (MVU), popup, подписка, MPRIS-мост
src/ui/controls.rs     — стили (glass) и виджеты: компактный ряд, hover-ряд, popup
src/mpris/mod.rs       — типы: Status, PlayerInfo
src/mpris/player.rs    — обнаружение плееров и D-Bus-вызовы управления
src/mpris/monitor.rs   — наблюдение за состоянием плеера, загрузка обложки
res/icons/music-pill.svg — иконка
res/music-pill.desktop — десктоп-запись
```

## Сборка

```sh
cargo build --release
```

Зависимости берутся из git: `libcosmic` (pop-os/libcosmic) и его форк
`iced` (pop-os/iced). Требуется Wayland-сессия (libcosmic собран с
фичами `applet`, `applet-token`, `multi-window`, `tokio`, `wayland`).

```sh
# запустить
./target/release/music-pill
```

## Установка в dock (Cosmic)

```sh
cargo install --path .
cp res/music-pill.desktop ~/.local/share/applications/
```

Иконка: скопируйте `res/icons/music-pill.svg` в
`~/.local/share/icons/hicolor/scalable/actions/` как `music-pill.svg`
(или в системную папку иконок).

## Примечания

- Приложение работает в Wayland-сессии Pop!_OS/Cosmic; для обычного
  X11-окна applet-режим libcosmic не предназначен.
- Обложки, пришедшие по `http(s)`, декодируются через `image`-crate и
  отображаются через `iced::widget::image`.
- Тикер позиции 1 Гц подстрахует плееры, которые перестают шлать
  сигнал `Position`.
