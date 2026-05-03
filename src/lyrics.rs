use std::fs;
use std::path::Path;

/// A single timestamped lyric line.
#[derive(Debug, Clone, PartialEq)]
pub struct LyricLine {
    pub timestamp: f64,  // seconds
    pub text: String,
}

/// Parsed LRC lyrics file.
#[derive(Debug, Clone)]
pub struct Lyrics {
    pub lines: Vec<LyricLine>,
    pub title: Option<String>,
    pub artist: Option<String>,
}

impl Lyrics {
    /// Try to find and parse an .lrc file next to the given audio file.
    pub fn find_for_audio(audio_path: &Path) -> Option<Self> {
        let lrc_path = audio_path.with_extension("lrc");
        if lrc_path.exists() {
            return Self::load(&lrc_path);
        }
        // Some files use .txt extension for LRC
        let txt_path = audio_path.with_extension("txt");
        if txt_path.exists() {
            if let Some(lyrics) = Self::load(&txt_path) {
                if !lyrics.lines.is_empty() {
                    return Some(lyrics);
                }
            }
        }
        None
    }

    pub fn load(path: &Path) -> Option<Self> {
        let content = fs::read_to_string(path).ok()?;
        Some(Self::parse(&content))
    }

    pub fn parse(content: &str) -> Self {
        let mut lines = Vec::new();
        let mut title = None;
        let mut artist = None;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Parse ID tags: [ti:Title] [ar:Artist]
            if line.starts_with("[ti:") {
                title = Some(line[4..line.len() - 1].to_string());
                continue;
            }
            if line.starts_with("[ar:") {
                artist = Some(line[4..line.len() - 1].to_string());
                continue;
            }
            // Skip other ID tags
            if line.starts_with('[') && line.contains(":") && !line.contains(']') {
                continue;
            }

            // Parse timestamp lines: [mm:ss.xx]text or [mm:ss]text
            let mut timestamps = Vec::new();
            let mut rest_start = 0;

            while rest_start < line.len() {
                if line[rest_start..].starts_with('[') {
                    if let Some(end) = line[rest_start..].find(']') {
                        let tag = &line[rest_start + 1..rest_start + end];
                        // Check if it's a timestamp
                        if let Some(ts) = parse_timestamp(tag) {
                            timestamps.push(ts);
                            rest_start += end + 1;
                            continue;
                        }
                    }
                }
                break;
            }

            if !timestamps.is_empty() {
                let text = line[rest_start..].trim().to_string();
                for ts in timestamps {
                    lines.push(LyricLine {
                        timestamp: ts,
                        text: text.clone(),
                    });
                }
            }
        }

        // Sort by timestamp
        lines.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap_or(std::cmp::Ordering::Equal));

        Lyrics {
            lines,
            title,
            artist,
        }
    }

    /// Get the current lyric line index for the given playback position in seconds.
    pub fn current_line_index(&self, position_secs: f64) -> Option<usize> {
        if self.lines.is_empty() {
            return None;
        }
        let mut idx = 0;
        for (i, line) in self.lines.iter().enumerate() {
            if line.timestamp <= position_secs {
                idx = i;
            } else {
                break;
            }
        }
        Some(idx)
    }

    /// Get a window of lyric lines around the current position.
    pub fn window(&self, position_secs: f64, context: usize) -> &[LyricLine] {
        if self.lines.is_empty() {
            return &[];
        }
        let current = self.current_line_index(position_secs).unwrap_or(0);
        let start = current.saturating_sub(context);
        let end = (current + context + 1).min(self.lines.len());
        &self.lines[start..end]
    }
}

fn parse_timestamp(s: &str) -> Option<f64> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    let minutes: f64 = parts[0].parse().ok()?;
    let seconds: f64 = parts[1].parse().ok()?;
    Some(minutes * 60.0 + seconds)
}
