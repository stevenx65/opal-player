<div align="center">

[Chinese version](./README.zh.md)

# ♫ Opal Player

*A modern, minimal, fully-featured TUI music player*

[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![AUR](https://img.shields.io/badge/AUR-opal--player-blue)](https://aur.archlinux.org/packages/opal-player)

</div>

---

## ✨ Features

| Feature |
|---------|
| 🎵 Browse & play local music (MP3, FLAC, WAV, OGG, AAC, M4A, Opus) |
| 🎛️ Play/Pause, Stop, Next/Prev, Seek, Volume, Mute |
| 📋 Playlist management (create, save/load M3U) |
| 🔍 Real-time search/filter by title, artist, album |
| 🎤 Synchronized lyrics (.lrc) display |
| ⚙️ Persistent config (volume, queue, theme, position) |
| ⌨️ Full keyboard control with help popup (`?`) |
| 🖱️ Mouse support — click tracks to play, drag progress bar |
| 🔌 MPRIS D-Bus integration — media keys, desktop environments |

## 🖼️ Screenshot

```
┌──────────────────────────────────────────────────────────────┐
│  ♫ Opal Player  │  📁 ~/Music  │  ▶ PLAYING               │
│  ── 1:Library ─│── 2:Playlists ─│── 3:Queue ─│── 4:Now ── │
│  42 tracks  │  ▶ PLAYING  │  Song - Artist                  │
├──────────────┼────────────────────────┬─────────────────────┤
│  Library     │  Now Playing          │  Queue               │
│              │  ┌──────────┐         │                      │
│  ▶ Song 1    │  │  ♫  ♫   │         │  ▶ Track 1           │
│    Song 2    │  │   ♫     │         │    Track 2            │
│    Song 3    │  │  ♫  ♫   │         │    Track 3            │
│    Song 4    │  └──────────┘         │                      │
│              │  Title:  Song Name    │                      │
│              │  Artist: Artist Name  │                      │
│              │  Album:  Album Name   │                      │
│              │  ═══════════════      │                      │
│              │  01:23 / 04:56       │                      │
│              │  ── Lyrics ──        │                      │
│              │  previous line       │                      │
│              │  → current line ←    │                      │
│              │  next line           │                      │
├──────────────┴────────────────────────┴─────────────────────┤
│  🔊 80% │ ↻ Off │ 🔀 Off │ Press ? for help               │
│  ?:help │ Space:pause │ n/p:next/prev │ ←/→:seek │ /:search│
└──────────────────────────────────────────────────────────────┘
```

## ⌨️ Keybindings

| Key | Action |
|-----|--------|
| `q` / `Esc` / `Ctrl+C` | Quit |
| `Space` | Play / Pause |
| `Enter` | Play selected track |
| `n` / `p` | Next / Previous track |
| `←` / `→` | Seek backward / forward 5s |
| `↑` / `↓` | Navigate lists |
| `+` / `-` | Volume up / down |
| `m` | Toggle mute |
| `s` | Toggle shuffle |
| `r` | Toggle repeat (Off → Playlist → Track) |
| `a` | Add to queue |
| `d` | Remove from queue |
| `x` | Clear queue |
| `A` | Add to playlist |
| `N` | Create new playlist |
| `/` | Focus search |
| `Tab` | Switch panel focus |
| `1`–`4` | Switch tab view |
| `?` | Toggle help |

## 🖱️ Mouse Controls

| Action |
|--------|
| Click track in Library — Select and play the track |
| Click entry in Queue — Select and play the queued track |
| Click progress bar — Seek to that position |
| Scroll wheel in Library / Queue — Move cursor up / down |

## 🔌 MPRIS D-Bus

Opal TUI registers as `org.mpris.MediaPlayer2.opal` on the session bus, enabling:

| Feature |
|---------|
| Media keys (Play/Pause, Next, Prev, Stop) |
| Desktop environment now-playing widgets (KDE, GNOME) |
| Volume control / Seek via DE integrations |
| Show track metadata (title, artist, album) |

## 🚀 Quick Start

### Prerequisites

- **Rust** stable (1.70+)
- **ALSA** development libraries (Linux only)

```bash
# Debian/Ubuntu
sudo apt install libasound2-dev

# Arch
sudo pacman -S alsa-lib

# Fedora
sudo dnf install alsa-lib-devel
```

### Install

**Arch Linux (AUR):**

```bash
paru -S opal-player
# or: yay -S opal-player
```

**From source:**

```bash
git clone https://github.com/stevenx65/opal-player.git
cd opal-player
cargo build --release
./target/release/opal-player
```

### Configuration

Config is auto-created at `~/.config/opal-player/config.toml` on first run.

```toml
volume = 0.8
shuffle = false
repeat_mode = "off"
music_dirs = ["~/Music"]
theme_preset = "opaline"  # opaline | catppuccin | nord | dracula | one_dark

[theme_overrides]
primary = "f0c0c0"
secondary = "b4f9f8"
# ... customize any color from the preset palette
```

## 🏗️ Architecture

```
src/
├── main.rs      Entry point, terminal setup, event loop
├── app.rs       Application state & event dispatch
├── player.rs    Audio engine (rodio + symphonia, precise seek)
├── library.rs   Music scanner + lofty metadata
├── playlist.rs  M3U playlist management
├── lyrics.rs    LRC lyrics parser
├── ui.rs        ratatui rendering (panels, progress, lyrics, help)
├── theme.rs     Opaline theme engine (configurable colors)
├── config.rs    TOML config persistence
├── input.rs     Keybinding system
└── error.rs     Error types
```

## 🛠️ Tech Stack

| Component | Crate |
|-----------|-------|
| TUI | `ratatui` + `crossterm` |
| Async | `tokio` |
| Audio output | `rodio` |
| Audio decoding | `symphonia` (all formats) |
| Metadata | `lofty` |
| Config | `serde` + `toml` |
| Error handling | `thiserror` + `anyhow` |

## 📄 License

MIT © 2026 [stevenx65](https://github.com/stevenx65)
