use std::sync::Arc;
use std::time::Duration;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use crate::config::Config;
use crate::error::Result;
use crate::input::{Action, Keybindings};
use crate::library::{MusicLibrary, TrackInfo};
use crate::lyrics::Lyrics;
use crate::mpris::{MprisAction, MprisMetadata, MprisShared};
use crate::player::{Player, RepeatMode};
use crate::playlist::PlaylistManager;
use crate::theme::OpalineTheme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPanel {
    Library,
    NowPlaying,
    Queue,
    Search,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabView {
    Library,
    Playlists,
    Queue,
    NowPlaying,
}

/// Areas of clickable UI elements, populated by the render pass.
#[derive(Debug, Clone, Copy, Default)]
pub struct UiLayout {
    pub library_list: Rect,
    pub queue_list: Rect,
    pub progress_bar: Rect,
}

fn scroll_offset_for(total: usize, selected: usize, visible_rows: usize) -> usize {
    if total <= visible_rows {
        0
    } else {
        let half = visible_rows / 2;
        if selected > half {
            (selected - half).min(total - visible_rows)
        } else {
            0
        }
    }
}

pub struct App {
    pub player: Player,
    pub library: MusicLibrary,
    pub playlist_manager: PlaylistManager,
    pub config: Config,
    pub theme: OpalineTheme,
    pub keybindings: Keybindings,
    pub focused_panel: FocusedPanel,
    pub active_tab: TabView,
    pub search_active: bool,
    pub search_cursor_visible: bool,
    pub show_help: bool,
    pub running: bool,
    pub lyrics: Option<Lyrics>,
    pub status_msg: String,
    pub status_timer: i32,
    pub cursor_timer: u32,
    pub mpris_shared: Arc<MprisShared>,
}

impl App {
    pub fn new(mpris_shared: Arc<MprisShared>) -> Result<Self> {
        let config = Config::load().unwrap_or_default();
        let theme = config.theme();
        let mut player = Player::new()?;
        let vol = config.volume;
        player.set_volume(vol);

        Ok(Self {
            player,
            library: MusicLibrary::new(),
            playlist_manager: PlaylistManager::new(),
            config,
            theme,
            keybindings: Keybindings::default(),
            focused_panel: FocusedPanel::Library,
            active_tab: TabView::Library,
            search_active: false,
            search_cursor_visible: true,
            show_help: false,
            running: true,
            lyrics: None,
            status_msg: String::new(),
            status_timer: 0,
            cursor_timer: 0,
            mpris_shared,
        })
    }

    pub fn handle_event(&mut self, event: &KeyEvent) -> Result<()> {
        // Only respond to initial press — repeat events are ignored.
        // The Player::seek guard prevents overlapping seeks, so buffered
        // events are harmless no-ops.
        if event.kind != KeyEventKind::Press {
            return Ok(());
        }

        // Arrow keys handle seek directly (not via keybindings) for zero-latency response.
        match event.code {
            KeyCode::Right => return self.player.seek(5.0).map_err(Into::into),
            KeyCode::Left => return self.player.seek(-5.0).map_err(Into::into),
            _ => {}
        }

        if self.search_active {
            return self.handle_search_input(event);
        }

        let Some(action) = self.keybindings.resolve(event) else {
            return Ok(());
        };

        self.execute_action(action)
    }

    pub fn handle_mouse(&mut self, event: MouseEvent, layout: &UiLayout) -> Result<()> {
        if event.kind != MouseEventKind::Down(MouseButton::Left) {
            return Ok(());
        }

        let (col, row) = (event.column, event.row);

        // ── Progress bar click → seek ──
        let pb = layout.progress_bar;
        if row == pb.y && col >= pb.x && col < pb.x + pb.width {
            let ratio = (col - pb.x) as f64 / pb.width.saturating_sub(1) as f64;
            let ratio = ratio.clamp(0.0, 1.0);
            if let Some(total) = self.player.total_duration {
                let target = total.as_secs_f64() * ratio;
                let _ = self.player.seek_to(target);
                self.set_status(&format!("Seeked to {:.0}s", target));
            }
            return Ok(());
        }

        // ── Library list click → select + play ──
        let lib = layout.library_list;
        if col >= lib.x && col < lib.x + lib.width && row >= lib.y && row < lib.y + lib.height {
            let rel_row = (row - lib.y) as usize;
            let total = self.library.filtered_indices.len();
            let visible_rows = (lib.height as usize).saturating_sub(2);
            if visible_rows == 0 || total == 0 {
                return Ok(());
            }
            let scroll_offset = scroll_offset_for(total, self.library.selected_index, visible_rows);
            let target_idx = scroll_offset + rel_row;
            if target_idx < total {
                self.library.selected_index = target_idx;
                self.focused_panel = FocusedPanel::Library;
                // Play on click
                if let Some(track) = self.library.selected_track() {
                    self.play_track(&track)?;
                }
            }
            return Ok(());
        }

        // ── Queue list click → select + play ──
        let q = layout.queue_list;
        if col >= q.x && col < q.x + q.width && row >= q.y && row < q.y + q.height {
            let rel_row = (row - q.y) as usize;
            let total = self.playlist_manager.queue.len();
            let visible_rows = (q.height as usize).saturating_sub(2);
            if visible_rows == 0 || total == 0 {
                return Ok(());
            }
            let scroll_offset = scroll_offset_for(total, self.playlist_manager.selected_queue_index, visible_rows);
            let target_idx = scroll_offset + rel_row;
            if target_idx < total {
                self.playlist_manager.selected_queue_index = target_idx;
                self.focused_panel = FocusedPanel::Queue;
                let entry = self.playlist_manager.queue.remove(target_idx);
                self.playlist_manager.selected_queue_index = self
                    .playlist_manager
                    .selected_queue_index
                    .min(self.playlist_manager.queue.len().saturating_sub(1));
                let path = entry.path.clone();
                let info = MusicLibrary::read_metadata_file(&path).unwrap_or_else(|_| TrackInfo {
                    path,
                    title: entry.title,
                    artist: entry.artist,
                    album: String::new(),
                    duration: entry.duration_secs.map(Duration::from_secs_f64),
                    track_number: None,
                    genre: None,
                });
                self.play_track(&info)?;
            }
            return Ok(());
        }

        Ok(())
    }

    fn handle_search_input(&mut self, event: &KeyEvent) -> Result<()> {
        match event.code {
            KeyCode::Esc => {
                self.search_active = false;
                self.focused_panel = FocusedPanel::Library;
            }
            KeyCode::Enter => {
                self.search_active = false;
                self.focused_panel = FocusedPanel::Library;
            }
            KeyCode::Backspace => {
                self.library.search_query.pop();
                self.library.update_filter();
            }
            KeyCode::Char(c) => {
                self.library.search_query.push(c);
                self.library.update_filter();
            }
            _ => {}
        }
        Ok(())
    }

    pub fn execute_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Quit => {
                self.save_state();
                self.running = false;
            }

            Action::PlayPause => self.player.play_pause(),

            Action::PlaySelected => match self.focused_panel {
                FocusedPanel::Queue => {
                    let idx = self.playlist_manager.selected_queue_index;
                    if let Some(entry) = self.playlist_manager.queue.get(idx).cloned() {
                        self.playlist_manager.queue.remove(idx);
                        self.playlist_manager.selected_queue_index =
                            idx.min(self.playlist_manager.queue.len().saturating_sub(1));
                        let path = entry.path.clone();
                        let info = MusicLibrary::read_metadata_file(&path)
                            .unwrap_or_else(|_| TrackInfo {
                                path,
                                title: entry.title,
                                artist: entry.artist,
                                album: String::new(),
                                duration: entry.duration_secs.map(Duration::from_secs_f64),
                                track_number: None,
                                genre: None,
                            });
                        self.play_track(&info)?;
                    }
                }
                _ => {
                    if let Some(track) = self.library.selected_track() {
                        self.play_track(&track)?;
                    }
                }
            },

            Action::Stop => {
                self.player.stop();
                self.lyrics = None;
            }

            Action::NextTrack => {
                self.play_next()?;
            }

            Action::PrevTrack => {
                self.play_previous()?;
            }

            Action::VolumeUp => self.player.change_volume(0.05),
            Action::VolumeDown => self.player.change_volume(-0.05),
            Action::Mute => self.player.toggle_mute(),

            Action::FocusSearch => {
                self.search_active = true;
            }

            Action::ToggleShuffle => self.player.shuffle = !self.player.shuffle,

            Action::ToggleRepeat => {
                self.player.repeat_mode = match self.player.repeat_mode {
                    RepeatMode::Off => RepeatMode::Playlist,
                    RepeatMode::Playlist => RepeatMode::Track,
                    RepeatMode::Track => RepeatMode::Off,
                };
            }

            Action::AddToQueue => {
                if let Some(track) = self.library.selected_track() {
                    self.playlist_manager.add_to_queue(&track);
                    self.set_status(&format!("Added: {}", track.display_name()));
                }
            }

            Action::RemoveFromQueue => {
                let idx = self.playlist_manager.selected_queue_index;
                if self.playlist_manager.remove_from_queue(idx).is_some() {
                    self.set_status("Removed from queue");
                }
            }

            Action::ClearQueue => {
                self.playlist_manager.clear_queue();
                self.set_status("Queue cleared");
            }

            Action::AddToPlaylist => {
                if let Some(track) = self.library.selected_track() {
                    if let Some(pl) = self.playlist_manager.current_playlist_mut() {
                        pl.add_track(&track);
                        self.set_status(&format!("Added to playlist: {}", track.display_name()));
                    } else {
                        self.set_status("No playlist selected. Press 'N' to create one.");
                    }
                }
            }

            Action::CreatePlaylist => {
                self.playlist_manager.create_playlist("New Playlist");
                self.set_status("New playlist created");
            }

            Action::SavePlaylist => {
                if let Err(e) = self.playlist_manager.save_current_playlist() {
                    self.set_status(&format!("Save failed: {}", e));
                } else {
                    self.set_status("Playlist saved");
                }
            }

            Action::LoadPlaylist => {
                self.set_status("Use .m3u file to load");
            }

            Action::SwitchPanel => {
                self.focused_panel = match self.focused_panel {
                    FocusedPanel::Library => FocusedPanel::NowPlaying,
                    FocusedPanel::NowPlaying => FocusedPanel::Queue,
                    FocusedPanel::Queue => FocusedPanel::Library,
                    FocusedPanel::Search => FocusedPanel::Library,
                };
            }

            Action::ToggleHelp => self.show_help = !self.show_help,

            Action::MoveUp => match self.focused_panel {
                FocusedPanel::Library => self.library.select_prev(),
                FocusedPanel::Queue => self.playlist_manager.select_queue_prev(),
                _ => {}
            },

            Action::MoveDown => match self.focused_panel {
                FocusedPanel::Library => self.library.select_next(),
                FocusedPanel::Queue => self.playlist_manager.select_queue_next(),
                _ => {}
            },

            Action::PageUp => {
                if self.focused_panel == FocusedPanel::Library {
                    let skip = 10.min(self.library.selected_index);
                    self.library.selected_index -= skip;
                }
            }

            Action::PageDown => {
                if self.focused_panel == FocusedPanel::Library {
                    let max = self.library.filtered_indices.len().saturating_sub(1);
                    self.library.selected_index = (self.library.selected_index + 10).min(max);
                }
            }

            Action::Home => match self.focused_panel {
                FocusedPanel::Library => self.library.selected_index = 0,
                FocusedPanel::Queue => self.playlist_manager.selected_queue_index = 0,
                _ => {}
            },

            Action::End => match self.focused_panel {
                FocusedPanel::Library => {
                    let max = self.library.filtered_indices.len().saturating_sub(1);
                    self.library.selected_index = max;
                }
                FocusedPanel::Queue => {
                    let max = self.playlist_manager.queue.len().saturating_sub(1);
                    self.playlist_manager.selected_queue_index = max;
                }
                _ => {}
            },

            Action::SelectTab1 => self.active_tab = TabView::Library,
            Action::SelectTab2 => self.active_tab = TabView::Playlists,
            Action::SelectTab3 => self.active_tab = TabView::Queue,
            Action::SelectTab4 => self.active_tab = TabView::NowPlaying,

            _ => {}
        }
        Ok(())
    }

    fn play_track(&mut self, track: &TrackInfo) -> Result<()> {
        // Remove this track from the queue if present — it's being played now,
        // so play_next should not replay it later.
        if let Some(pos) = self
            .playlist_manager
            .queue
            .iter()
            .position(|e| e.path == track.path)
        {
            self.playlist_manager.queue.remove(pos);
            self.playlist_manager.selected_queue_index = self
                .playlist_manager
                .selected_queue_index
                .min(self.playlist_manager.queue.len().saturating_sub(1));
        }

        self.lyrics = Lyrics::find_for_audio(&track.path);
        self.player.play_file(track)?;
        self.set_status(&format!("Now playing: {}", track.display_name()));
        Ok(())
    }

    fn play_next(&mut self) -> Result<()> {
        // Try queue first
        if !self.playlist_manager.queue.is_empty() {
            let entry = self.playlist_manager.queue.remove(0);
            self.playlist_manager.selected_queue_index = self
                .playlist_manager
                .selected_queue_index
                .min(self.playlist_manager.queue.len().saturating_sub(1));
            let path = entry.path.clone();
            if let Ok(info) = MusicLibrary::read_metadata_file(&path) {
                return self.play_track(&info);
            }
            let info = TrackInfo {
                path,
                title: entry.title,
                artist: entry.artist,
                album: String::new(),
                duration: entry.duration_secs.map(Duration::from_secs_f64),
                track_number: None,
                genre: None,
            };
            return self.play_track(&info);
        }

        // Otherwise, next in filtered library
        let current_idx = self
            .player
            .current_track
            .as_ref()
            .and_then(|t| {
                self.library
                    .filtered_indices
                    .iter()
                    .position(|&i| self.library.tracks[i].path == t.path)
            })
            .unwrap_or(self.library.selected_index);

        let next_idx = if self.library.filtered_indices.is_empty() {
            return Ok(());
        } else if self.player.shuffle {
            use std::collections::hash_map::RandomState;
            use std::hash::{BuildHasher, Hasher};
            let len = self.library.filtered_indices.len();
            if len > 1 {
                let mut rng = RandomState::new().build_hasher();
                rng.write_usize(current_idx);
                let mut rand = rng.finish() as usize % len;
                if rand == current_idx {
                    rand = (rand + 1) % len;
                }
                rand
            } else {
                0
            }
        } else if current_idx + 1 < self.library.filtered_indices.len() {
            current_idx + 1
        } else {
            match self.player.repeat_mode {
                RepeatMode::Off => {
                    self.player.stop();
                    return Ok(());
                }
                RepeatMode::Playlist => 0,
                RepeatMode::Track => current_idx,
            }
        };

        if let Some(&track_idx) = self.library.filtered_indices.get(next_idx) {
            let track = self.library.tracks[track_idx].clone();
            self.library.selected_index = next_idx;
            self.play_track(&track)?;
        }

        Ok(())
    }

    fn play_previous(&mut self) -> Result<()> {
        let current_idx = self
            .player
            .current_track
            .as_ref()
            .and_then(|t| {
                self.library
                    .filtered_indices
                    .iter()
                    .position(|&i| self.library.tracks[i].path == t.path)
            })
            .unwrap_or(self.library.selected_index);

        if current_idx > 0 {
            let prev_idx = current_idx - 1;
            if let Some(&track_idx) = self.library.filtered_indices.get(prev_idx) {
                let track = self.library.tracks[track_idx].clone();
                self.library.selected_index = prev_idx;
                self.play_track(&track)?;
            }
        }

        Ok(())
    }

    fn set_status(&mut self, msg: &str) {
        self.status_msg = msg.to_string();
        self.status_timer = 120; // ~2 seconds at 60fps
    }

    pub fn tick(&mut self) {
        // Decrement status timer
        if self.status_timer > 0 {
            self.status_timer -= 1;
            if self.status_timer == 0 {
                self.status_msg.clear();
            }
        }

        // Blink search cursor
        self.cursor_timer = (self.cursor_timer + 1) % 30;
        if self.cursor_timer == 0 {
            self.search_cursor_visible = !self.search_cursor_visible;
        }

        // Check if current track finished
        if self.player.is_finished() {
            let _ = self.play_next();
        }

        // Sync player state → MPRIS
        self.update_mpris_state();

        // Process MPRIS commands from external sources (media keys, desktop)
        self.poll_mpris_commands();
    }

    fn update_mpris_state(&self) {
        let mut state = self.mpris_shared.state.lock().unwrap();

        state.playback_status = match self.player.state {
            crate::player::PlayState::Playing => "Playing".into(),
            crate::player::PlayState::Paused => "Paused".into(),
            crate::player::PlayState::Stopped => "Stopped".into(),
        };

        state.volume = self.player.volume as f64;
        state.position = self.player.get_elapsed().as_micros() as i64;
        state.shuffle = self.player.shuffle;

        state.loop_status = match self.player.repeat_mode {
            RepeatMode::Off => "None".into(),
            RepeatMode::Playlist => "Playlist".into(),
            RepeatMode::Track => "Track".into(),
        };

        if let Some(track) = &self.player.current_track {
            state.metadata.track_id =
                crate::mpris::track_id_from_path(&track.path);
            state.metadata.title = track.title.clone();
            state.metadata.artist = if track.artist.is_empty() {
                vec![]
            } else {
                vec![track.artist.clone()]
            };
            state.metadata.album = track.album.clone();
            state.metadata.length = track
                .duration
                .or(self.player.total_duration)
                .map(|d| d.as_micros() as i64)
                .unwrap_or(0);
        } else {
            state.metadata = MprisMetadata::default();
        }
    }

    fn poll_mpris_commands(&mut self) {
        // Collect commands in a scoped block so the Mutex lock is released
        // before we call &mut self methods below.
        let cmds: Vec<MprisAction> = {
            let mut guard = self.mpris_shared.commands.lock().unwrap();
            if guard.is_empty() {
                return;
            }
            guard.drain(..).collect()
        };

        for cmd in cmds {
            match cmd {
                MprisAction::PlayPause => self.player.play_pause(),
                MprisAction::Next => {
                    let _ = self.play_next();
                }
                MprisAction::Previous => {
                    let _ = self.play_previous();
                }
                MprisAction::Stop => {
                    self.player.stop();
                    self.lyrics = None;
                }
                MprisAction::Seek(offset) => {
                    let delta = offset as f64 / 1_000_000.0;
                    let _ = self.player.seek(delta);
                }
                MprisAction::SetPosition(pos) => {
                    let target = pos as f64 / 1_000_000.0;
                    let _ = self.player.seek_to(target);
                }
                MprisAction::SetVolume(v) => {
                    self.player.set_volume(v as f32);
                }
                MprisAction::Quit => {
                    self.save_state();
                    self.running = false;
                }
            }
        }
    }

    pub fn is_library_focused(&self) -> bool {
        matches!(
            (self.active_tab, self.focused_panel),
            (TabView::Library, FocusedPanel::Library)
                | (TabView::Queue, FocusedPanel::Library)
                | (TabView::NowPlaying, FocusedPanel::Library)
        )
    }

    pub fn save_state(&mut self) {
        self.config.volume = self.player.volume;
        self.config.muted = self.player.muted;
        self.config.shuffle = self.player.shuffle;
        self.config.repeat_mode = match self.player.repeat_mode {
            RepeatMode::Off => crate::config::RepeatModeConfig::Off,
            RepeatMode::Playlist => crate::config::RepeatModeConfig::Playlist,
            RepeatMode::Track => crate::config::RepeatModeConfig::Track,
        };

        // Save recent files
        if let Some(track) = &self.player.current_track {
            let path_str = track.path.to_string_lossy().to_string();
            if !self.config.recent_files.contains(&path_str) {
                self.config.recent_files.push(path_str);
                if self.config.recent_files.len() > 50 {
                    self.config.recent_files.remove(0);
                }
            }
        }

        if let Err(e) = self.config.save() {
            eprintln!("Failed to save config: {}", e);
        }
    }
}
