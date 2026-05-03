use std::path::{Path, PathBuf};
use std::sync::Arc;
use lofty::prelude::*;
use lofty::probe::Probe;

use crate::error::Result;

pub const SUPPORTED_EXTENSIONS: &[&str] = &["mp3", "flac", "wav", "ogg", "aac", "m4a", "opus"];

#[derive(Debug, Clone)]
pub struct TrackInfo {
    pub path: PathBuf,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: Option<std::time::Duration>,
    pub track_number: Option<u32>,
    pub genre: Option<String>,
}

impl TrackInfo {
    pub fn display_name(&self) -> String {
        if self.title.is_empty() {
            self.path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown".to_string())
        } else {
            format!("{} - {}", self.artist, self.title)
        }
    }

    pub fn search_text(&self) -> String {
        format!(
            "{} {} {} {}",
            self.title,
            self.artist,
            self.album,
            self.path.to_string_lossy()
        )
        .to_lowercase()
    }
}

#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub expanded: bool,
    pub depth: usize,
    pub track_count: usize,
}

#[derive(Debug, Clone)]
pub struct MusicLibrary {
    pub tracks: Vec<Arc<TrackInfo>>,
    pub directories: Vec<DirectoryEntry>,
    pub current_dir: PathBuf,
    pub filtered_indices: Vec<usize>,
    pub search_query: String,
    pub selected_index: usize,
    pub is_loading: bool,
}

impl MusicLibrary {
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
            directories: Vec::new(),
            current_dir: dirs::audio_dir().unwrap_or_else(|| PathBuf::from(".")),
            filtered_indices: Vec::new(),
            search_query: String::new(),
            selected_index: 0,
            is_loading: false,
        }
    }

    pub async fn scan_directory(&mut self, dir: &Path) -> Result<()> {
        self.is_loading = true;
        self.current_dir = dir.to_path_buf();
        self.tracks.clear();
        self.directories.clear();

        self.scan_recursive(dir, 0).await?;
        self.update_filter();

        self.is_loading = false;
        Ok(())
    }

    async fn scan_recursive(&mut self, dir: &Path, depth: usize) -> Result<()> {
        let mut entries: Vec<std::fs::DirEntry> = match std::fs::read_dir(dir) {
            Ok(read) => read.filter_map(|e| e.ok()).collect(),
            Err(_) => return Ok(()),
        };

        entries.sort_by(|a, b| {
            let a_is_dir = a.file_type().map(|t| t.is_dir()).unwrap_or(false);
            let b_is_dir = b.file_type().map(|t| t.is_dir()).unwrap_or(false);
            b_is_dir
                .cmp(&a_is_dir)
                .then_with(|| a.file_name().cmp(&b.file_name()))
        });

        for entry in &entries {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if name.starts_with('.') {
                continue;
            }

            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                let track_count = self.count_tracks_in_dir(&path);
                self.directories.push(DirectoryEntry {
                    name,
                    path: path.clone(),
                    is_dir: true,
                    expanded: false,
                    depth,
                    track_count,
                });
            } else if file_type.is_file() {
                if let Some(ext) = path.extension() {
                    let ext = ext.to_string_lossy().to_lowercase();
                    if SUPPORTED_EXTENSIONS.contains(&ext.as_str()) {
                        if let Ok(info) = self.read_metadata(&path) {
                            self.tracks.push(Arc::new(info));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn count_tracks_in_dir(&self, dir: &Path) -> usize {
        let mut count = 0;
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.file_name().map(|n| n.to_string_lossy().starts_with('.')).unwrap_or(true) {
                    continue;
                }
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    count += self.count_tracks_in_dir(&path);
                } else if let Some(ext) = path.extension() {
                    if SUPPORTED_EXTENSIONS.contains(&ext.to_string_lossy().to_lowercase().as_str()) {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    pub fn read_metadata_file(path: &Path) -> Result<TrackInfo> {
        Self::read_metadata_static(path)
    }

    fn read_metadata(&self, path: &Path) -> Result<TrackInfo> {
        Self::read_metadata_static(path)
    }

    fn read_metadata_static(path: &Path) -> Result<TrackInfo> {
        let tagged_file = Probe::open(path)
            .map_err(|e| crate::error::AppError::Decode(e.to_string()))?
            .read()
            .ok();

        let tag = tagged_file.as_ref().and_then(|f| {
            f.primary_tag().or_else(|| f.first_tag())
        });

        let properties = tagged_file.as_ref().map(|f| f.properties());

        let title = tag
            .and_then(|t| t.title().map(|s| s.to_string()))
            .unwrap_or_default();
        let artist = tag
            .and_then(|t| t.artist().map(|s| s.to_string()))
            .unwrap_or_default();
        let album = tag
            .and_then(|t| t.album().map(|s| s.to_string()))
            .unwrap_or_default();
        let track_number = tag.and_then(|t| t.track());
        let genre = tag
            .and_then(|t| t.genre().map(|s| s.to_string()));

        let duration = properties.map(|p| p.duration());

        Ok(TrackInfo {
            path: path.to_path_buf(),
            title,
            artist,
            album,
            duration,
            track_number,
            genre,
        })
    }

    pub fn update_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_indices = (0..self.tracks.len()).collect();
        } else {
            let query = self.search_query.to_lowercase();
            self.filtered_indices = self
                .tracks
                .iter()
                .enumerate()
                .filter(|(_, t)| t.search_text().contains(&query))
                .map(|(i, _)| i)
                .collect();
        }
        if self.selected_index >= self.filtered_indices.len() {
            self.selected_index = self.filtered_indices.len().saturating_sub(1);
        }
    }

    pub fn search(&mut self, query: &str) {
        self.search_query = query.to_string();
        self.update_filter();
    }

    pub fn selected_track(&self) -> Option<Arc<TrackInfo>> {
        self.filtered_indices
            .get(self.selected_index)
            .and_then(|&i| self.tracks.get(i))
            .cloned()
    }

    pub fn select_next(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.selected_index = (self.selected_index + 1).min(self.filtered_indices.len() - 1);
        }
    }

    pub fn select_prev(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
    }

    pub fn select_index(&mut self, idx: usize) {
        self.selected_index = idx.min(self.filtered_indices.len().saturating_sub(1));
    }
}
