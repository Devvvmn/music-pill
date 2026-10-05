# Music Pill - Cosmic DE Applet
## References & Resources

### Official Documentation
- **libcosmic GitHub**: https://github.com/pop-os/libcosmic
- **API Documentation**: https://pop-os.github.io/libcosmic/cosmic/
- **libcosmic Book**: https://pop-os.github.io/libcosmic-book/
- **Iced Book**: https://book.iced.rs/

### Templates & Examples
- **Cosmic Applet Template**: https://github.com/pop-os/cosmic-applet-template
- **Cosmic App Template**: https://github.com/pop-os/cosmic-app-template
- **libcosmic Examples**: https://github.com/pop-os/libcosmic/tree/main/examples

### D-Bus / MPRIS
- **MPRIS Spec**: https://specifications.freedesktop.org/mpris-spec/
- **zbus crate**: https://docs.rs/zbus/
- **D-Bus Intro**: https://dbus.freedesktop.org/

### Design & Style
- **Cosmic Design**: https://github.com/pop-os/cosmic-design
- **macOS Music Widget Inspiration**

### Key Crates for Music Pill
```toml
[dependencies]
cosmic = "0.1"
iced = "0.12"
tokio = { version = "1", features = ["full"] }
zbus = "4"
```

---

## Project Structure

```
music_pill/
├── Cargo.toml              # Rust project config
├── src/
│   ├── main.rs             # Entry point
│   ├── ui/
│   │   ├── pill.rs         # Main UI component
│   │   └── controls.rs     # Play/pause/volume controls
│   └── mpris/
│       ├── player.rs       # D-Bus MPRIS interface
│       └── monitor.rs      # Listen to player changes
├── examples/
│   └── basic_applet.rs     # Basic libcosmic example
├── docs/
│   ├── ICE_FRAMEWORK.md    # libcosmic documentation
│   └── MPRIS_INTEGRATION.md # D-Bus/MPRIS guide
└── PROMPT.md               # Development brief
```

---

## Quick Start

1. **Clone template**:
   ```bash
   git clone https://github.com/pop-os/cosmic-applet-template music-pill
   cd music-pill
   ```

2. **Add dependencies**:
   ```toml
   zbus = "4"
   serde = { version = "1", features = ["derive"] }
   ```

3. **Implement UI** (glassmorphic pill with track info)

4. **Connect to MPRIS** for real player data

5. **Test in COSMIC panel**

---

## macOS Music Pill Style Guide

### Visual
- **Shape**: Rounded pill (radius ≈ 32px for 60px height)
- **Background**: `rgba(255, 255, 255, 0.1)` with `blur(20px)`
- **Size**: 180×60px (compact), expands on hover
- **Animation**: Smooth spring easing

### Layout
```
┌─────────────────────────────┐
│ 🎵 Track Title              │ ← Album art (optional)
│    Artist Name      ▶ ⏭ 🔊  │
└─────────────────────────────┘
  └────────────── Progress ──┘
```

### Interaction
- Click to expand → full player
- Hover → show all controls
- Progress drag to seek
- Volume slider

---

## Testing Checklist

- [ ] Applet loads in COSMIC panel
- [ ] Connects to active music player
- [ ] Play/pause buttons work
- [ ] Next/previous buttons work
- [ ] Shows current track info
- [ ] Updates on player changes
- [ ] Volume control works
- [ ] Glassmorphic background renders
- [ ] Responsive sizing
