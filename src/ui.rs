use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Gauge, List, ListItem, Paragraph, Tabs,
    },
    Frame,
};

use crate::app::{App, FocusedPanel, TabView};
use crate::player::{PlayState, RepeatMode};
use crate::theme::OpalineTheme;

pub fn render(f: &mut Frame, app: &App) {
    let theme = &app.theme;

    let area = f.area();

    // Main layout: title, content, status, search
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // title bar
            Constraint::Min(0),     // content area
            Constraint::Length(1),  // status bar
            Constraint::Length(1),  // search bar (if active)
        ])
        .split(area);

    render_title_bar(f, app, main_layout[0], theme);

    // Content area: three panels or tabs
    if app.show_help {
        render_help(f, main_layout[1], theme);
    } else {
        render_content(f, app, main_layout[1], theme);
    }

    render_status_bar(f, app, main_layout[2], theme);

    if app.search_active {
        render_search_bar(f, app, main_layout[3], theme);
    } else {
        render_hint_bar(f, app, main_layout[3], theme);
    }
}

fn render_title_bar(f: &mut Frame, app: &App, area: Rect, theme: &OpalineTheme) {
    let title_block = Block::default()
        .style(Style::default().bg(theme.surface).fg(theme.text_bright));

    let inner = title_block.inner(area);
    f.render_widget(title_block, area);

    let title_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    // Row 1: App name + music dir
    let row1 = Line::from(vec![
        Span::styled("♫ Opal Player", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("  │  ", Style::default().fg(theme.text_dim)),
        Span::styled(
            format!("📁 {}", app.library.current_dir.display()),
            Style::default().fg(theme.text_dim),
        ),
    ]);
    let center = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)])
        .split(title_layout[0]);
    f.render_widget(Paragraph::new(row1).alignment(Alignment::Left), center[0]);

    // Row 2: Tabs
    let tabs = vec!["1:Library", "2:Playlists", "3:Queue", "4:Now Playing"];
    let tab_idx = match app.active_tab {
        TabView::Library => 0,
        TabView::Playlists => 1,
        TabView::Queue => 2,
        TabView::NowPlaying => 3,
    };
    let tab_widget = Tabs::new(tabs)
        .block(Block::default())
        .select(tab_idx)
        .style(Style::default().fg(theme.text))
        .highlight_style(
            Style::default()
                .fg(theme.highlight)
                .add_modifier(Modifier::BOLD),
        )
        .divider("│");
    f.render_widget(tab_widget, title_layout[1]);

    // Row 3: Track count + play state indicator
    let state_text = match app.player.state {
        PlayState::Playing => Span::styled("▶ PLAYING", Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
        PlayState::Paused => Span::styled("⏸ PAUSED", Style::default().fg(theme.warning)),
        PlayState::Stopped => Span::styled("⏹ STOPPED", Style::default().fg(theme.text_dim)),
    };

    let extra = if let Some(track) = &app.player.current_track {
        format!("{} - {}", track.artist, track.title)
    } else {
        "No track loaded".to_string()
    };

    let row3 = Line::from(vec![
        Span::styled(
            format!("{} tracks", app.library.filtered_indices.len()),
            Style::default().fg(theme.text_dim),
        ),
        Span::styled("  │  ", Style::default().fg(theme.border)),
        state_text,
        Span::styled("  │  ", Style::default().fg(theme.border)),
        Span::styled(extra, Style::default().fg(theme.text)),
    ]);
    f.render_widget(Paragraph::new(row3).alignment(Alignment::Left), title_layout[2]);
}

fn render_content(f: &mut Frame, app: &App, area: Rect, theme: &OpalineTheme) {
    match app.active_tab {
        TabView::Library | TabView::NowPlaying => {
            // Three-panel layout
            let panels = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Percentage(40),
                    Constraint::Percentage(30),
                ])
                .split(area);

            render_library_panel(f, app, panels[0], theme);
            render_now_playing(f, app, panels[1], theme);
            render_queue_panel(f, app, panels[2], theme);
        }
        TabView::Playlists => {
            let panels = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ])
                .split(area);
            render_playlists_panel(f, app, panels[0], theme);
            render_playlist_tracks(f, app, panels[1], theme);
        }
        TabView::Queue => {
            let panels = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ])
                .split(area);
            render_library_panel(f, app, panels[0], theme);
            render_queue_panel(f, app, panels[1], theme);
        }
    }
}

fn render_library_panel(f: &mut Frame, app: &App, area: Rect, theme: &OpalineTheme) {
    let is_focused = app.is_library_focused();
    let border_style = if is_focused {
        Style::default().fg(theme.primary)
    } else {
        Style::default().fg(theme.border)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title_bottom(Span::styled("Library", Style::default().fg(theme.text_dim)))
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.library.is_loading {
        let loading = Paragraph::new("Scanning...")
            .style(Style::default().fg(theme.text_dim))
            .alignment(Alignment::Center);
        f.render_widget(loading, inner);
        return;
    }

    if app.library.filtered_indices.is_empty() {
        let empty = Paragraph::new("No tracks found\n\nPress / to search\nUse ←↓↑→ to navigate")
            .style(Style::default().fg(theme.text_dim))
            .alignment(Alignment::Center);
        f.render_widget(empty, inner);
        return;
    }

    // Build list items with scrolling
    let items: Vec<ListItem> = app
        .library
        .filtered_indices
        .iter()
        .enumerate()
        .map(|(i, &track_idx)| {
            let track = &app.library.tracks[track_idx];
            let is_selected = i == app.library.selected_index;

            let prefix = if is_selected && is_focused {
                "▶ "
            } else {
                "  "
            };

            let style = if is_selected && is_focused {
                Style::default()
                    .fg(theme.highlight)
                    .bg(theme.highlight_bg)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default()
                    .fg(theme.text)
                    .bg(theme.surface)
            } else {
                Style::default().fg(theme.text)
            };

            let track_text = if track.title.is_empty() {
                track
                    .path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown".to_string())
            } else {
                format!("{}", track.title)
            };

            let artist_text = if track.artist.is_empty() {
                String::new()
            } else {
                format!("  [{}]", track.artist)
            };

            Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(track_text, style),
                Span::styled(artist_text, Style::default().fg(theme.text_dim)),
            ])
            .into()
        })
        .collect();

    let list = List::new(items)
        .block(Block::default())
        .style(Style::default().bg(theme.bg));

    f.render_widget(list, inner);
}

fn render_now_playing(f: &mut Frame, app: &App, area: Rect, theme: &OpalineTheme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(" Now Playing ")
        .title_style(Style::default().fg(theme.primary))
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let now_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // cover placeholder + info
            Constraint::Length(3),  // progress bar + time
            Constraint::Min(0),     // lyrics (if available)
        ])
        .split(inner);

    if let Some(track) = &app.player.current_track {
        // Cover art placeholder
        let cover_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(12), Constraint::Min(0)])
            .split(now_layout[0]);

        let cover_text = vec![
            Line::from("  ┌──────┐  "),
            Line::from("  │ ♫  ♫ │  "),
            Line::from("  │  ♫   │  "),
            Line::from("  │ ♫  ♫ │  "),
            Line::from("  └──────┘  "),
        ];
        let cover_para = Paragraph::new(cover_text)
            .style(Style::default().fg(theme.primary))
            .alignment(Alignment::Center);
        f.render_widget(cover_para, cover_area[0]);

        // Track info
        let info_lines = vec![
            Line::from(vec![
                Span::styled("Title:  ", Style::default().fg(theme.text_dim)),
                Span::styled(
                    &track.title,
                    Style::default()
                        .fg(theme.text_bright)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Artist: ", Style::default().fg(theme.text_dim)),
                Span::styled(&track.artist, Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("Album:  ", Style::default().fg(theme.text_dim)),
                Span::styled(&track.album, Style::default().fg(theme.text)),
            ]),
        ];
        let info = Paragraph::new(info_lines).style(Style::default().fg(theme.text));
        f.render_widget(info, cover_area[1]);

        // Progress bar + time
        let elapsed = app.player.get_elapsed();
        let total = app.player.total_duration.unwrap_or(std::time::Duration::ZERO);

        let elapsed_str = format_duration(elapsed);
        let total_str = format_duration(total);

        let progress = if total > std::time::Duration::ZERO {
            (elapsed.as_secs_f64() / total.as_secs_f64()).min(1.0)
        } else {
            0.0
        };

        let gauge = Gauge::default()
            .block(Block::default())
            .gauge_style(Style::default().fg(theme.progress).bg(theme.progress_bg))
            .ratio(progress)
            .label(format!(" {} / {} ", elapsed_str, total_str));

        f.render_widget(gauge, now_layout[1]);

        // Lyrics
        if let Some(lyrics) = &app.lyrics {
            let pos_secs = elapsed.as_secs_f64();
            let window = lyrics.window(pos_secs, 5);
            let current_idx = lyrics.current_line_index(pos_secs);

            let lyric_lines: Vec<Line> = window
                .iter()
                .enumerate()
                .map(|(i, line)| {
                    let global_idx = current_idx.map_or(0, |c| {
                        c.saturating_sub(5) + i
                    });
                    let is_current = Some(global_idx) == current_idx;

                    let style = if is_current {
                        Style::default()
                            .fg(theme.highlight)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.text_dim)
                    };

                    Line::from(Span::styled(&line.text, style))
                })
                .collect();

            let lyrics_block = Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(theme.border))
                .title(" Lyrics ")
                .title_style(Style::default().fg(theme.accent));

            let lyrics_inner = lyrics_block.inner(now_layout[2]);
            f.render_widget(lyrics_block, now_layout[2]);
            f.render_widget(
                Paragraph::new(lyric_lines).alignment(Alignment::Center),
                lyrics_inner,
            );
        } else {
            let nothing = Paragraph::new("No lyrics available")
                .style(Style::default().fg(theme.text_dim))
                .alignment(Alignment::Center);
            f.render_widget(nothing, now_layout[2]);
        }
    } else {
        // No track loaded
        let placeholder = vec![
            Line::from(""),
            Line::from("  ♫  No track loaded  ♫"),
            Line::from(""),
            Line::from("  Browse your library"),
            Line::from("  and press Enter to play"),
        ];
        let para = Paragraph::new(placeholder)
            .style(Style::default().fg(theme.text_dim))
            .alignment(Alignment::Center);
        f.render_widget(para, inner);
    }
}

fn render_queue_panel(f: &mut Frame, app: &App, area: Rect, theme: &OpalineTheme) {
    let is_focused = app.focused_panel == FocusedPanel::Queue;
    let border_style = if is_focused {
        Style::default().fg(theme.primary)
    } else {
        Style::default().fg(theme.border)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(" Queue ")
        .title_style(Style::default().fg(theme.secondary))
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.playlist_manager.queue.is_empty() {
        let empty = Paragraph::new("Queue is empty\n\n'a' to add tracks\n'd' to remove")
            .style(Style::default().fg(theme.text_dim))
            .alignment(Alignment::Center);
        f.render_widget(empty, inner);
        return;
    }

    let items: Vec<ListItem> = app
        .playlist_manager
        .queue
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_selected = i == app.playlist_manager.selected_queue_index && is_focused;

            let style = if is_selected {
                Style::default()
                    .fg(theme.highlight)
                    .bg(theme.highlight_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };

            let prefix = if is_selected && is_focused { "▶ " } else { "  " };
            Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(&entry.title, style),
                Span::styled(
                    format!("  [{}]", entry.artist),
                    Style::default().fg(theme.text_dim),
                ),
            ])
            .into()
        })
        .collect();

    let list = List::new(items)
        .block(Block::default())
        .style(Style::default().bg(theme.bg));

    f.render_widget(list, inner);
}

fn render_playlists_panel(f: &mut Frame, app: &App, area: Rect, theme: &OpalineTheme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(" Playlists ")
        .title_style(Style::default().fg(theme.secondary))
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let playlists = &app.playlist_manager.playlists;

    if playlists.is_empty() {
        let empty = Paragraph::new("No playlists\n\n'N' to create\nDrag .m3u to load")
            .style(Style::default().fg(theme.text_dim))
            .alignment(Alignment::Center);
        f.render_widget(empty, inner);
        return;
    }

    let items: Vec<ListItem> = playlists
        .iter()
        .enumerate()
        .map(|(i, pl)| {
            let is_selected = i == app.playlist_manager.selected_playlist_index;
            let style = if is_selected {
                Style::default()
                    .fg(theme.highlight)
                    .bg(theme.highlight_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };
            let prefix = if is_selected { "▶ " } else { "  " };
            Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(&pl.name, style),
                Span::styled(
                    format!("  ({} tracks)", pl.tracks.len()),
                    Style::default().fg(theme.text_dim),
                ),
            ])
            .into()
        })
        .collect();

    let list = List::new(items).block(Block::default());
    f.render_widget(list, inner);
}

fn render_playlist_tracks(f: &mut Frame, app: &App, area: Rect, theme: &OpalineTheme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(" Tracks ")
        .title_style(Style::default().fg(theme.text_dim))
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let playlist = match app.playlist_manager.current_playlist() {
        Some(p) => p,
        None => {
            let empty = Paragraph::new("Select a playlist")
                .style(Style::default().fg(theme.text_dim))
                .alignment(Alignment::Center);
            f.render_widget(empty, inner);
            return;
        }
    };

    let items: Vec<ListItem> = playlist
        .tracks
        .iter()
        .enumerate()
        .map(|(_i, entry)| {
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(&entry.title, Style::default().fg(theme.text)),
                Span::styled(
                    format!("  [{}]", entry.artist),
                    Style::default().fg(theme.text_dim),
                ),
            ])
            .into()
        })
        .collect();

    let list = List::new(items).block(Block::default());
    f.render_widget(list, inner);
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect, theme: &OpalineTheme) {
    let vol_text = if app.player.muted {
        "🔇 MUTED".to_string()
    } else {
        format!("🔊 {}%", (app.player.volume * 100.0) as u32)
    };

    let repeat_text = match app.player.repeat_mode {
        RepeatMode::Off => "↻ Off",
        RepeatMode::Playlist => "↻ Playlist",
        RepeatMode::Track => "↻ Track",
    };

    let shuffle_text = if app.player.shuffle {
        "🔀 On"
    } else {
        "🔀 Off"
    };

    let status = Line::from(vec![
        Span::styled(
            format!(" {} │ ", vol_text),
            Style::default().fg(theme.text),
        ),
        Span::styled(
            format!("{} │ ", repeat_text),
            Style::default().fg(if app.player.repeat_mode != RepeatMode::Off {
                theme.success
            } else {
                theme.text_dim
            }),
        ),
        Span::styled(
            format!("{} │ ", shuffle_text),
            Style::default().fg(if app.player.shuffle {
                theme.success
            } else {
                theme.text_dim
            }),
        ),
        Span::styled(
            if app.search_active {
                "Search: "
            } else {
                ""
            },
            Style::default().fg(theme.primary),
        ),
        Span::styled(
            if app.search_active {
                &app.library.search_query
            } else {
                "Press ? for help"
            },
            Style::default().fg(theme.text_dim),
        ),
    ]);

    let status_block = Block::default()
        .style(Style::default().bg(theme.surface));

    f.render_widget(&status_block, area);
    f.render_widget(
        Paragraph::new(status).alignment(Alignment::Left),
        status_block.inner(area),
    );
}

fn render_search_bar(f: &mut Frame, app: &App, area: Rect, theme: &OpalineTheme) {
    let block = Block::default()
        .style(Style::default().bg(theme.surface_alt));

    f.render_widget(&block, area);

    let cursor = if app.search_cursor_visible { "▎" } else { " " };
    let text = format!("/ {}", app.library.search_query);
    let line = Line::from(vec![
        Span::styled(text, Style::default().fg(theme.text_bright)),
        Span::styled(cursor, Style::default().fg(theme.primary)),
    ]);

    f.render_widget(
        Paragraph::new(line).alignment(Alignment::Left),
        block.inner(area),
    );
}

fn render_hint_bar(f: &mut Frame, _app: &App, area: Rect, theme: &OpalineTheme) {
    let hints = vec![
        "?:help", "Space:pause", "n/p:next/prev", "←/→:seek",
        "+/-:vol", "a:add", "d:remove", "/:search", "Tab:switch",
    ];
    let hint_text = hints.join(" │ ");

    let line = Line::from(Span::styled(
        hint_text,
        Style::default().fg(theme.text_dim),
    ));

    let hint_block = Block::default()
        .style(Style::default().bg(theme.surface));

    f.render_widget(&hint_block, area);
    f.render_widget(
        Paragraph::new(line).alignment(Alignment::Center),
        hint_block.inner(area),
    );
}

fn render_help(f: &mut Frame, area: Rect, theme: &OpalineTheme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary))
        .title(" Help ")
        .title_style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(area);
    f.render_widget(block, area);
    f.render_widget(Clear, area);

    let help_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner);

    f.render_widget(
        Paragraph::new("Keyboard Shortcuts")
            .style(Style::default().fg(theme.text_bright).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center),
        help_layout[0],
    );

    let shortcuts = vec![
        ("q / Esc / Ctrl+C", "Quit"),
        ("Space", "Play / Pause"),
        ("Enter", "Play selected track"),
        ("n / p", "Next / Previous track"),
        ("← / →", "Seek backward / forward 5s"),
        ("↑ / ↓", "Navigate lists"),
        ("+ / -", "Volume up / down"),
        ("m", "Toggle mute"),
        ("s", "Toggle shuffle"),
        ("r", "Toggle repeat (Off / Playlist / Track)"),
        ("a", "Add selected to queue"),
        ("d", "Remove from queue"),
        ("x", "Clear queue"),
        ("A", "Add to current playlist"),
        ("N", "Create new playlist"),
        ("/", "Focus search / Filter library"),
        ("Tab", "Switch panel focus"),
        ("1-4", "Switch tab view"),
        ("?", "Toggle this help"),
    ];

    let help_lines: Vec<Line> = shortcuts
        .iter()
        .map(|(key, desc)| {
            Line::from(vec![
                Span::styled(
                    format!("  {:20}", key),
                    Style::default()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(*desc, Style::default().fg(theme.text)),
            ])
        })
        .collect();

    let help_text = Paragraph::new(help_lines).style(Style::default().bg(theme.bg));
    f.render_widget(help_text, help_layout[1]);
}

fn format_duration(d: std::time::Duration) -> String {
    let total_secs = d.as_secs();
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;
    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, mins, secs)
    } else {
        format!("{:02}:{:02}", mins, secs)
    }
}

// Clear widget for overlays
struct Clear;

impl ratatui::widgets::Widget for Clear {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        for x in area.left()..area.right() {
            for y in area.top()..area.bottom() {
                buf[(x, y)].set_char(' ');
            }
        }
    }
}
