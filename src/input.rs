use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    Quit,
    PlayPause,
    PlaySelected,
    Stop,
    NextTrack,
    PrevTrack,
    SeekForward,
    SeekBackward,
    VolumeUp,
    VolumeDown,
    Mute,
    FocusSearch,
    ToggleShuffle,
    ToggleRepeat,
    AddToQueue,
    RemoveFromQueue,
    AddToPlaylist,
    CreatePlaylist,
    SavePlaylist,
    LoadPlaylist,
    ClearQueue,
    SwitchPanel,
    ToggleHelp,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    PageUp,
    PageDown,
    Home,
    End,
    ExpandDir,
    CollapseDir,
    RefreshLibrary,
    SelectTab1,
    SelectTab2,
    SelectTab3,
    SelectTab4,
}

impl Action {
    pub fn description(self) -> &'static str {
        match self {
            Action::Quit => "Quit",
            Action::PlayPause => "Play/Pause",
            Action::PlaySelected => "Play selected track",
            Action::Stop => "Stop playback",
            Action::NextTrack => "Next track",
            Action::PrevTrack => "Previous track",
            Action::SeekForward => "Seek forward 5s",
            Action::SeekBackward => "Seek backward 5s",
            Action::VolumeUp => "Volume up",
            Action::VolumeDown => "Volume down",
            Action::Mute => "Toggle mute",
            Action::FocusSearch => "Focus search",
            Action::ToggleShuffle => "Toggle shuffle",
            Action::ToggleRepeat => "Toggle repeat mode",
            Action::AddToQueue => "Add to queue",
            Action::RemoveFromQueue => "Remove from queue",
            Action::AddToPlaylist => "Add to playlist",
            Action::CreatePlaylist => "Create new playlist",
            Action::SavePlaylist => "Save playlist",
            Action::LoadPlaylist => "Load playlist",
            Action::ClearQueue => "Clear queue",
            Action::SwitchPanel => "Switch focus",
            Action::ToggleHelp => "Toggle help",
            Action::MoveUp => "Move up",
            Action::MoveDown => "Move down",
            Action::MoveLeft => "Move left",
            Action::MoveRight => "Move right",
            Action::PageUp => "Page up",
            Action::PageDown => "Page down",
            Action::Home => "Go to start",
            Action::End => "Go to end",
            Action::ExpandDir => "Expand directory",
            Action::CollapseDir => "Collapse directory",
            Action::RefreshLibrary => "Refresh library",
            Action::SelectTab1 => "Library tab",
            Action::SelectTab2 => "Playlists tab",
            Action::SelectTab3 => "Queue tab",
            Action::SelectTab4 => "Now Playing tab",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Keybindings {
    pub bindings: HashMap<KeyPattern, Action>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct KeyPattern {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl Keybindings {
    pub fn default() -> Self {
        let mut bindings = HashMap::new();

        macro_rules! bind {
            ($code:expr, $action:expr) => {
                bindings.insert(
                    KeyPattern {
                        code: $code,
                        modifiers: KeyModifiers::NONE,
                    },
                    $action,
                );
            };
            ($code:expr, $mod:expr, $action:expr) => {
                bindings.insert(
                    KeyPattern {
                        code: $code,
                        modifiers: $mod,
                    },
                    $action,
                );
            };
        }

        // Navigation
        bind!(KeyCode::Up, Action::MoveUp);
        bind!(KeyCode::Down, Action::MoveDown);
        bind!(KeyCode::Left, Action::MoveLeft);
        bind!(KeyCode::Right, Action::MoveRight);
        bind!(KeyCode::PageUp, Action::PageUp);
        bind!(KeyCode::PageDown, Action::PageDown);
        bind!(KeyCode::Home, Action::Home);
        bind!(KeyCode::End, Action::End);
        bind!(KeyCode::Tab, Action::SwitchPanel);
        bind!(KeyCode::BackTab, KeyModifiers::SHIFT, Action::SwitchPanel);

        // Playback
        bind!(KeyCode::Char(' '), Action::PlayPause);
        bind!(KeyCode::Enter, Action::PlaySelected);
        bind!(KeyCode::Char('n'), Action::NextTrack);
        bind!(KeyCode::Char('p'), Action::PrevTrack);
        bind!(KeyCode::Char('s'), Action::ToggleShuffle);
        bind!(KeyCode::Char('r'), Action::ToggleRepeat);
        bind!(KeyCode::Char('m'), Action::Mute);
        bind!(KeyCode::Char('+'), Action::VolumeUp);
        bind!(KeyCode::Char('-'), Action::VolumeDown);
        bind!(KeyCode::Char('='), Action::VolumeUp);

        // Seek
        bind!(KeyCode::Left, Action::SeekBackward);
        bind!(KeyCode::Right, Action::SeekForward);

        // Queue / Playlist
        bind!(KeyCode::Char('a'), Action::AddToQueue);
        bind!(KeyCode::Char('d'), Action::RemoveFromQueue);
        bind!(KeyCode::Char('x'), Action::ClearQueue);

        // Playlist management
        bind!(KeyCode::Char('A'), Action::AddToPlaylist);
        bind!(KeyCode::Char('N'), Action::CreatePlaylist);

        // Misc
        bind!(KeyCode::Char('/'), Action::FocusSearch);
        bind!(KeyCode::Char('?'), Action::ToggleHelp);
        bind!(KeyCode::Char('q'), Action::Quit);
        bind!(KeyCode::Esc, Action::Quit);

        // Tabs
        bind!(KeyCode::Char('1'), Action::SelectTab1);
        bind!(KeyCode::Char('2'), Action::SelectTab2);
        bind!(KeyCode::Char('3'), Action::SelectTab3);
        bind!(KeyCode::Char('4'), Action::SelectTab4);

        Self { bindings }
    }

    pub fn resolve(&self, event: &KeyEvent) -> Option<Action> {
        // Ctrl+C is always quit
        if event.code == KeyCode::Char('c') && event.modifiers == KeyModifiers::CONTROL {
            return Some(Action::Quit);
        }
        if event.code == KeyCode::Char('C') && event.modifiers == KeyModifiers::CONTROL {
            return Some(Action::Quit);
        }

        let pattern = KeyPattern {
            code: event.code,
            modifiers: event.modifiers,
        };
        self.bindings.get(&pattern).copied()
    }

    pub fn key_for_action(&self, action: Action) -> String {
        for (pattern, act) in &self.bindings {
            if *act == action {
                return pattern.to_string();
            }
        }
        "?".to_string()
    }

    pub fn help_entries(&self) -> Vec<(String, &'static str)> {
        let mut seen = std::collections::HashSet::new();
        let mut entries = Vec::new();
        for (pattern, action) in &self.bindings {
            if seen.insert(*action) {
                entries.push((pattern.to_string(), action.description()));
            }
        }
        // Also show Ctrl+C
        entries.push(("Ctrl+C".to_string(), "Quit"));
        entries.sort_by(|a, b| a.1.cmp(b.1));
        entries
    }
}

impl std::fmt::Display for KeyPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.modifiers.contains(KeyModifiers::CONTROL) {
            write!(f, "Ctrl+")?;
        }
        if self.modifiers.contains(KeyModifiers::ALT) {
            write!(f, "Alt+")?;
        }
        if self.modifiers.contains(KeyModifiers::SHIFT) {
            write!(f, "Shift+")?;
        }

        match self.code {
            KeyCode::Char(c) => write!(f, "{}", c),
            KeyCode::Enter => write!(f, "Enter"),
            KeyCode::Tab => write!(f, "Tab"),
            KeyCode::BackTab => write!(f, "Shift+Tab"),
            KeyCode::Esc => write!(f, "Esc"),
            KeyCode::Up => write!(f, "↑"),
            KeyCode::Down => write!(f, "↓"),
            KeyCode::Left => write!(f, "←"),
            KeyCode::Right => write!(f, "→"),
            KeyCode::Home => write!(f, "Home"),
            KeyCode::End => write!(f, "End"),
            KeyCode::PageUp => write!(f, "PgUp"),
            KeyCode::PageDown => write!(f, "PgDn"),
            KeyCode::F(n) => write!(f, "F{}", n),
            KeyCode::Null => write!(f, "Null"),
            _ => write!(f, "??"),
        }
    }
}
