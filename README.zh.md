<div align="center">

[English version](./README.md)

# ♫ Opal Player

一个现代、极简、功能完整的终端音乐播放器

[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![AUR](https://img.shields.io/badge/AUR-opal--player-blue)](https://aur.archlinux.org/packages/opal-player)

</div>

---

## ✨ 功能

| 功能 |
|------|
| 🎵 浏览、播放本地音乐文件 |
| 🎛️ 播放控制：播放/暂停/停止/上下曲/快进/音量/静音 |
| 📋 播放列表管理（创建、保存/加载 M3U） |
| 🔍 实时搜索/过滤（按标题、艺术家、专辑） |
| 🎤 同步歌词显示（.lrc 文件） |
| ⚙️ 持久化配置（音量、队列、主题、播放位置） |
| ⌨️ 全键盘操作，`?` 弹出帮助 |
| 🖱️ 鼠标支持 — 点击曲目播放，点击进度条跳转 |
| 🔌 MPRIS D-Bus 集成 — 多媒体键、桌面环境控制 |

## 🖼️ 截图

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

## ⌨️ 快捷键

| 按键 | 功能 |
|------|------|
| `q` / `Esc` / `Ctrl+C` | 退出 |
| `Space` | 播放 / 暂停 |
| `Enter` | 播放选中曲目 |
| `n` / `p` | 下一曲 / 上一曲 |
| `←` / `→` | 快退 / 快进 5 秒 |
| `↑` / `↓` | 列表导航 |
| `+` / `-` | 音量增减 |
| `m` | 切换静音 |
| `s` | 切换随机播放 |
| `r` | 切换循环模式 |
| `a` | 添加到队列 |
| `d` | 从队列移除 |
| `x` | 清空队列 |
| `A` | 添加到播放列表 |
| `N` | 新建播放列表 |
| `/` | 聚焦搜索 |
| `Tab` | 切换面板焦点 |
| `1`–`4` | 切换标签页 |
| `?` | 切换帮助 |

## 🖱️ 鼠标操作

| 操作 |
|------|
| 点击库中的曲目 — 选中并播放曲目 |
| 点击队列中的条目 — 选中并播放队列中的曲目 |
| 点击进度条 — 跳转到对应播放位置 |
| 在库/队列中滚动滚轮 — 上下移动光标 |

## 🔌 MPRIS D-Bus 桌面集成

Opal TUI 在会话总线上注册为 `org.mpris.MediaPlayer2.opal`，支持：

| 功能 |
|------|
| 多媒体键控制（播放/暂停/上下曲/停止） |
| 桌面环境播放状态小部件 |
| 音量 / 进度跳转（桌面环境集成） |
| 显示曲目元数据（标题、艺术家、专辑） |

## 🚀 快速开始

### 前置条件

- **Rust** stable (1.70+)
- **ALSA** 开发库（仅 Linux）

```bash
# Debian/Ubuntu
sudo apt install libasound2-dev

# Arch
sudo pacman -S alsa-lib

# Fedora
sudo dnf install alsa-lib-devel
```

### 安装

**Arch Linux (AUR):**

```bash
paru -S opal-player
# or: yay -S opal-player
```

**从源码构建：**

```bash
git clone https://github.com/stevenx65/opal-player.git
cd opal-player
cargo build --release
./target/release/opal-player
```

### 配置

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

## 🏗️ 架构

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

## 🛠️ 技术栈

| 组件 | Crate |
|------|-------|
| 终端界面 | `ratatui` + `crossterm` |
| 异步 | `tokio` |
| 音频输出 | `rodio` |
| 音频解码 | `symphonia`（所有格式） |
| 元数据 | `lofty` |
| 配置 | `serde` + `toml` |
| 错误处理 | `thiserror` + `anyhow` |

## 📄 许可协议

MIT © 2026 [stevenx65](https://github.com/stevenx65)
