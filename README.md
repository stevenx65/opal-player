<div align="center">

# ♫ Opal Player

*A modern, minimal, fully-featured TUI music player*

一个现代、极简、功能完整的终端音乐播放器

[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![AUR](https://img.shields.io/badge/AUR-opal--player-blue)](https://aur.archlinux.org/packages/opal-player)

</div>

---

## ✨ Features | 功能

| Feature | 功能 |
|---------|------|
| 🎵 Browse & play local music (MP3, FLAC, WAV, OGG, AAC, M4A, Opus) | 浏览、播放本地音乐文件 |
| 🎛️ Play/Pause, Stop, Next/Prev, Seek, Volume, Mute | 播放控制：播放/暂停/停止/上下曲/快进/音量/静音 |
| 📋 Playlist management (create, save/load M3U) | 播放列表管理（创建、保存/加载 M3U） |
| 🔍 Real-time search/filter by title, artist, album | 实时搜索/过滤（按标题、艺术家、专辑） |
| 🎤 Synchronized lyrics (.lrc) display | 同步歌词显示（.lrc 文件） |
| ⚙️ Persistent config (volume, queue, theme, position) | 持久化配置（音量、队列、主题、播放位置） |
| ⌨️ Full keyboard control with help popup (`?`) | 全键盘操作，`?` 弹出帮助 |
| 🖱️ Mouse support — click tracks to play, drag progress bar | 鼠标支持 — 点击曲目播放，点击进度条跳转 |
| 🔌 MPRIS D-Bus integration — media keys, desktop environments | MPRIS D-Bus 集成 — 多媒体键、桌面环境控制 |

## 🖼️ Screenshot | 截图

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

## ⌨️ Keybindings | 快捷键

| Key | Action | 功能 |
|-----|--------|------|
| `q` / `Esc` / `Ctrl+C` | Quit | 退出 |
| `Space` | Play / Pause | 播放 / 暂停 |
| `Enter` | Play selected track | 播放选中曲目 |
| `n` / `p` | Next / Previous track | 下一曲 / 上一曲 |
| `←` / `→` | Seek backward / forward 5s | 快退 / 快进 5 秒 |
| `↑` / `↓` | Navigate lists | 列表导航 |
| `+` / `-` | Volume up / down | 音量增减 |
| `m` | Toggle mute | 切换静音 |
| `s` | Toggle shuffle | 切换随机播放 |
| `r` | Toggle repeat (Off → Playlist → Track) | 切换循环模式 |
| `a` | Add to queue | 添加到队列 |
| `d` | Remove from queue | 从队列移除 |
| `x` | Clear queue | 清空队列 |
| `A` | Add to playlist | 添加到播放列表 |
| `N` | Create new playlist | 新建播放列表 |
| `/` | Focus search | 聚焦搜索 |
| `Tab` | Switch panel focus | 切换面板焦点 |
| `1`–`4` | Switch tab view | 切换标签页 |
| `?` | Toggle help | 切换帮助 |

## 🖱️ Mouse Controls | 鼠标操作

| Action | 操作 |
|--------|------|
| Click track in Library | Select and play the track |
| 点击库中的曲目 | 选中并播放曲目 |
| Click entry in Queue | Select and play the queued track |
| 点击队列中的条目 | 选中并播放队列中的曲目 |
| Click progress bar | Seek to that position |
| 点击进度条 | 跳转到对应播放位置 |
| Scroll wheel in Library / Queue | Move cursor up / down |
| 在库/队列中滚动滚轮 | 上下移动光标 |

## 🔌 MPRIS D-Bus | 桌面集成

Opal TUI registers as `org.mpris.MediaPlayer2.opal` on the session bus, enabling:

Opal TUI 在会话总线上注册为 `org.mpris.MediaPlayer2.opal`，支持：

| Feature | 功能 |
|---------|------|
| Media keys (Play/Pause, Next, Prev, Stop) | 多媒体键控制（播放/暂停/上下曲/停止） |
| Desktop environment now-playing widgets (KDE, GNOME) | 桌面环境播放状态小部件 |
| Volume control / Seek via DE integrations | 音量 / 进度跳转（桌面环境集成） |
| Show track metadata (title, artist, album) | 显示曲目元数据（标题、艺术家、专辑） |

## 🚀 Quick Start | 快速开始

### Prerequisites | 前置条件

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

### Install | 安装

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

### Configuration | 配置

Config is auto-created at `~/.config/opal-player/config.toml` on first run.

首次运行后配置自动生成于 `~/.config/opal-player/config.toml`。

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

## 🏗️ Architecture | 架构

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

## 🛠️ Tech Stack | 技术栈

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
