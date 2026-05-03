use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::library::TrackInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub name: String,
    pub tracks: Vec<PlaylistEntry>,
    pub file_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistEntry {
    pub path: PathBuf,
    pub title: String,
    pub artist: String,
    pub duration_secs: Option<f64>,
}

impl Playlist {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tracks: Vec::new(),
            file_path: None,
        }
    }

    pub fn load_m3u(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read playlist: {}", path.display()))?;

        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string());

        let mut playlist = Self::new(&name);
        playlist.file_path = Some(path.to_path_buf());

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let track_path = PathBuf::from(line);
            playlist.tracks.push(PlaylistEntry {
                path: track_path.clone(),
                title: track_path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default(),
                artist: String::new(),
                duration_secs: None,
            });
        }

        Ok(playlist)
    }

    pub fn save_m3u(&self, path: Option<&Path>) -> Result<()> {
        let save_path = match path.or(self.file_path.as_deref()) {
            Some(p) => p.to_path_buf(),
            None => return Err(crate::error::AppError::Playlist("no save path".into())),
        };

        let mut content = String::from("#EXTM3U\n");
        content.push_str(&format!("#PLAYLIST:{}\n", self.name));

        for entry in &self.tracks {
            if let Some(dur) = entry.duration_secs {
                content.push_str(&format!("#EXTINF:{:.0},{}\n", dur, entry.title));
            }
            content.push_str(&entry.path.to_string_lossy());
            content.push('\n');
        }

        fs::write(&save_path, content)?;
        Ok(())
    }

    pub fn add_track(&mut self, track: &TrackInfo) {
        self.tracks.push(PlaylistEntry {
            path: track.path.clone(),
            title: track.title.clone(),
            artist: track.artist.clone(),
            duration_secs: track.duration.map(|d| d.as_secs_f64()),
        });
    }

    pub fn remove_track(&mut self, index: usize) -> Option<PlaylistEntry> {
        if index < self.tracks.len() {
            Some(self.tracks.remove(index))
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.tracks.clear();
    }
}

#[derive(Debug, Clone)]
pub struct PlaylistManager {
    pub playlists: Vec<Playlist>,
    pub queue: Vec<PlaylistEntry>,
    pub current_playlist_index: usize,
    pub selected_playlist_index: usize,
    pub selected_queue_index: usize,
}

impl PlaylistManager {
    pub fn new() -> Self {
        Self {
            playlists: Vec::new(),
            queue: Vec::new(),
            current_playlist_index: 0,
            selected_playlist_index: 0,
            selected_queue_index: 0,
        }
    }

    pub fn create_playlist(&mut self, name: &str) {
        self.playlists.push(Playlist::new(name));
        self.selected_playlist_index = self.playlists.len().saturating_sub(1);
    }

    pub fn load_playlist(&mut self, path: &Path) -> Result<()> {
        let playlist = Playlist::load_m3u(path)?;
        self.playlists.push(playlist);
        self.selected_playlist_index = self.playlists.len().saturating_sub(1);
        Ok(())
    }

    pub fn save_current_playlist(&self) -> Result<()> {
        if let Some(playlist) = self.playlists.get(self.selected_playlist_index) {
            playlist.save_m3u(None)?;
        }
        Ok(())
    }

    pub fn current_playlist(&self) -> Option<&Playlist> {
        self.playlists.get(self.current_playlist_index)
    }

    pub fn current_playlist_mut(&mut self) -> Option<&mut Playlist> {
        self.playlists.get_mut(self.current_playlist_index)
    }

    pub fn add_to_queue(&mut self, track: &TrackInfo) {
        self.queue.push(PlaylistEntry {
            path: track.path.clone(),
            title: track.title.clone(),
            artist: track.artist.clone(),
            duration_secs: track.duration.map(|d| d.as_secs_f64()),
        });
    }

    pub fn remove_from_queue(&mut self, index: usize) -> Option<PlaylistEntry> {
        if index < self.queue.len() {
            let removed = self.queue.remove(index);
            if self.selected_queue_index >= self.queue.len() {
                self.selected_queue_index = self.queue.len().saturating_sub(1);
            }
            Some(removed)
        } else {
            None
        }
    }

    pub fn clear_queue(&mut self) {
        self.queue.clear();
        self.selected_queue_index = 0;
    }

    pub fn pop_next_queue(&mut self) -> Option<PlaylistEntry> {
        if self.queue.is_empty() {
            None
        } else {
            Some(self.queue.remove(0))
        }
    }

    pub fn select_queue_next(&mut self) {
        if !self.queue.is_empty() {
            self.selected_queue_index =
                (self.selected_queue_index + 1).min(self.queue.len() - 1);
        }
    }

    pub fn select_queue_prev(&mut self) {
        self.selected_queue_index = self.selected_queue_index.saturating_sub(1);
    }

    pub fn select_playlist_next(&mut self) {
        if !self.playlists.is_empty() {
            self.selected_playlist_index =
                (self.selected_playlist_index + 1).min(self.playlists.len() - 1);
        }
    }

    pub fn select_playlist_prev(&mut self) {
        self.selected_playlist_index = self.selected_playlist_index.saturating_sub(1);
    }
}
