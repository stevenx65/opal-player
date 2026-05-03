use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use zbus::zvariant::{ObjectPath, Value};

/// Commands sent from the MPRIS D-Bus thread to the main application.
#[derive(Debug)]
pub enum MprisAction {
    PlayPause,
    Next,
    Previous,
    Stop,
    Seek(i64),
    SetPosition(i64),
    SetVolume(f64),
    Quit,
}

/// Metadata snapshot shared between the main thread and MPRIS thread.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MprisMetadata {
    pub track_id: String,
    pub title: String,
    pub artist: Vec<String>,
    pub album: String,
    pub length: i64,
}

/// Player state snapshot, updated by the main thread and read by MPRIS.
#[derive(Debug, Clone, PartialEq)]
pub struct MprisState {
    pub playback_status: String,
    pub volume: f64,
    pub position: i64,
    pub metadata: MprisMetadata,
    pub shuffle: bool,
    pub loop_status: String,
}

/// Container for shared state between main and MPRIS threads.
pub struct MprisShared {
    pub state: Mutex<MprisState>,
    pub commands: Mutex<Vec<MprisAction>>,
    pub state_version: AtomicU64,
    connection: Mutex<Option<zbus::Connection>>,
}

impl MprisShared {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(MprisState {
                playback_status: "Stopped".into(),
                volume: 0.8,
                position: 0,
                metadata: MprisMetadata::default(),
                shuffle: false,
                loop_status: "None".into(),
            }),
            commands: Mutex::new(Vec::new()),
            state_version: AtomicU64::new(0),
            connection: Mutex::new(None),
        }
    }

    fn push_command(&self, action: MprisAction) {
        self.commands.lock().unwrap().push(action);
    }

    /// Called by the main thread after updating the state.
    pub fn notify_state_changed(&self) {
        self.state_version.fetch_add(1, Ordering::Release);
    }
}

/// Generate a valid D-Bus object path segment from an arbitrary file path.
pub fn track_id_from_path(path: &std::path::Path) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    path.hash(&mut hasher);
    format!("/org/mpris/MediaPlayer2/opal/{}", hasher.finish())
}

// ── org.mpris.MediaPlayer2 (root) ──────────────────────────────────────

pub struct OpalMprisRoot {
    shared: Arc<MprisShared>,
}

#[zbus::dbus_interface(name = "org.mpris.MediaPlayer2")]
impl OpalMprisRoot {
    #[dbus_interface(property)]
    fn can_quit(&self) -> bool {
        true
    }

    #[dbus_interface(property)]
    fn can_raise(&self) -> bool {
        false
    }

    #[dbus_interface(property)]
    fn identity(&self) -> &str {
        "Opal Player"
    }

    #[dbus_interface(property)]
    fn desktop_entry(&self) -> &str {
        "opal-player"
    }

    #[dbus_interface(property)]
    fn supported_uri_schemes(&self) -> Vec<String> {
        vec!["file".into()]
    }

    #[dbus_interface(property)]
    fn supported_mime_types(&self) -> Vec<String> {
        vec![
            "audio/mpeg".into(),
            "audio/flac".into(),
            "audio/x-wav".into(),
            "audio/ogg".into(),
            "audio/aac".into(),
            "audio/x-m4a".into(),
            "audio/opus".into(),
        ]
    }

    fn raise(&self) {}

    fn quit(&self) {
        self.shared.push_command(MprisAction::Quit);
    }
}

// ── org.mpris.MediaPlayer2.Player ──────────────────────────────────────

pub struct OpalMprisPlayer {
    shared: Arc<MprisShared>,
}

#[zbus::dbus_interface(name = "org.mpris.MediaPlayer2.Player")]
impl OpalMprisPlayer {
    fn next(&self) {
        self.shared.push_command(MprisAction::Next);
    }

    fn previous(&self) {
        self.shared.push_command(MprisAction::Previous);
    }

    fn pause(&self) {
        let state = self.shared.state.lock().unwrap();
        if state.playback_status == "Playing" {
            drop(state);
            self.shared.push_command(MprisAction::PlayPause);
        }
    }

    fn play_pause(&self) {
        self.shared.push_command(MprisAction::PlayPause);
    }

    fn stop(&self) {
        self.shared.push_command(MprisAction::Stop);
    }

    fn play(&self) {
        let state = self.shared.state.lock().unwrap();
        if state.playback_status != "Playing" {
            drop(state);
            self.shared.push_command(MprisAction::PlayPause);
        }
    }

    fn seek(&self, offset: i64) {
        self.shared.push_command(MprisAction::Seek(offset));
    }

    fn set_position(&self, _track_id: ObjectPath<'_>, position: i64) {
        self.shared.push_command(MprisAction::SetPosition(position));
    }

    fn open_uri(&self, _uri: String) {}

    // ── Properties ──

    #[dbus_interface(property)]
    fn playback_status(&self) -> String {
        self.shared.state.lock().unwrap().playback_status.clone()
    }

    #[dbus_interface(property)]
    fn loop_status(&self) -> String {
        self.shared.state.lock().unwrap().loop_status.clone()
    }

    #[dbus_interface(property)]
    fn set_loop_status(&self, _value: String) {}

    #[dbus_interface(property)]
    fn rate(&self) -> f64 {
        1.0
    }

    #[dbus_interface(property)]
    fn set_rate(&self, _value: f64) {}

    #[dbus_interface(property)]
    fn shuffle(&self) -> bool {
        self.shared.state.lock().unwrap().shuffle
    }

    #[dbus_interface(property)]
    fn set_shuffle(&self, _value: bool) {}

    #[dbus_interface(property)]
    fn metadata(&self) -> HashMap<String, Value<'static>> {
        let state = self.shared.state.lock().unwrap();
        build_metadata_dict(&state.metadata)
    }

    #[dbus_interface(property)]
    fn volume(&self) -> f64 {
        self.shared.state.lock().unwrap().volume
    }

    #[dbus_interface(property)]
    fn set_volume(&self, value: f64) {
        self.shared
            .push_command(MprisAction::SetVolume(value.clamp(0.0, 1.5)));
    }

    #[dbus_interface(property)]
    fn position(&self) -> i64 {
        self.shared.state.lock().unwrap().position
    }

    #[dbus_interface(property)]
    fn can_go_next(&self) -> bool {
        true
    }

    #[dbus_interface(property)]
    fn can_go_previous(&self) -> bool {
        true
    }

    #[dbus_interface(property)]
    fn can_play(&self) -> bool {
        true
    }

    #[dbus_interface(property)]
    fn can_pause(&self) -> bool {
        true
    }

    #[dbus_interface(property)]
    fn can_seek(&self) -> bool {
        true
    }

    #[dbus_interface(property)]
    fn can_control(&self) -> bool {
        true
    }
}

// ── Metadata helper ────────────────────────────────────────────────────

fn build_metadata_dict(m: &MprisMetadata) -> HashMap<String, Value<'static>> {
    let mut map: HashMap<String, Value<'static>> = HashMap::new();

    if !m.track_id.is_empty() {
        // Cloning into a String to get an owned ObjectPath<'static>
        let owned = m.track_id.clone();
        let path = ObjectPath::from_str_unchecked(&owned).into_owned();
        map.insert("mpris:trackid".into(), Value::ObjectPath(path));
    }

    if !m.title.is_empty() {
        map.insert("xesam:title".into(), Value::new(m.title.clone()));
    }

    if !m.artist.is_empty() {
        let artists: Vec<Value<'static>> =
            m.artist.iter().map(|a| Value::new(a.clone())).collect();
        map.insert("xesam:artist".into(), Value::new(artists));
    }

    if !m.album.is_empty() {
        map.insert("xesam:album".into(), Value::new(m.album.clone()));
    }

    if m.length > 0 {
        map.insert("mpris:length".into(), Value::new(m.length));
        map.insert("mpris:artUrl".into(), Value::new(""));
    }

    map
}

// ── Spawn entry ────────────────────────────────────────────────────────

/// Starts the MPRIS D-Bus server in a background thread.
/// Emits PropertiesChanged signals when state changes so desktop
/// widgets (e.g. quickshell) can display track info in real time.
pub fn start_mpris(shared: Arc<MprisShared>) {
    std::thread::Builder::new()
        .name("mpris".into())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("build mpris tokio runtime");

            rt.block_on(async move {
                // Build connection
                let conn = match zbus::ConnectionBuilder::session() {
                    Ok(builder) => match builder.name("org.mpris.MediaPlayer2.opal") {
                        Ok(builder) => match builder.build().await {
                            Ok(conn) => conn,
                            Err(e) => {
                                log_mpris(&format!("build: {e}"));
                                return;
                            }
                        },
                        Err(e) => {
                            log_mpris(&format!("name: {e}"));
                            return;
                        }
                    },
                    Err(e) => {
                        log_mpris(&format!("session: {e}"));
                        return;
                    }
                };

                let root = OpalMprisRoot {
                    shared: shared.clone(),
                };
                let player = OpalMprisPlayer {
                    shared: shared.clone(),
                };

                let obj_path = "/org/mpris/MediaPlayer2";
                if let Err(e) = conn.object_server().at(obj_path, root).await {
                    log_mpris(&format!("root interface: {e}"));
                    return;
                }
                if let Err(e) = conn.object_server().at(obj_path, player).await {
                    log_mpris(&format!("player interface: {e}"));
                    return;
                }

                // Store connection so we can emit signals from the polling loop
                *shared.connection.lock().unwrap() = Some(conn.clone());

                log_mpris("MPRIS server started");

                // Polling loop: watch for state changes and emit PropertiesChanged
                let mut last_version: u64 = 0;
                let iface = "org.mpris.MediaPlayer2.Player";
                loop {
                    tokio::time::sleep(std::time::Duration::from_millis(250)).await;

                    let current_version = shared.state_version.load(Ordering::Acquire);
                    if current_version == last_version {
                        continue;
                    }
                    last_version = current_version;

                    // Read current state snapshot
                    let state = shared.state.lock().unwrap().clone();

                    // Build changed properties dict
                    let mut changed: HashMap<String, Value<'static>> = HashMap::new();
                    changed.insert("PlaybackStatus".into(), Value::new(state.playback_status));
                    changed.insert("LoopStatus".into(), Value::new(state.loop_status));
                    changed.insert("Shuffle".into(), Value::new(state.shuffle));
                    changed.insert("Volume".into(), Value::new(state.volume));
                    changed.insert("Position".into(), Value::new(state.position));
                    changed.insert("Metadata".into(), Value::new(build_metadata_dict(&state.metadata)));

                    let invalidated: Vec<String> = vec![];

                    let body = (iface.to_string(), changed, invalidated);
                    if let Err(e) = conn
                        .emit_signal(
                            None::<&str>,
                            obj_path,
                            "org.freedesktop.DBus.Properties",
                            "PropertiesChanged",
                            &body,
                        )
                        .await
                    {
                        log_mpris(&format!("emit PropertiesChanged: {e}"));
                    }
                }
            });
        })
        .expect("spawn mpris thread");
}

fn log_mpris(msg: &str) {
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/opal-mpris.log")
        .map(|mut f| {
            use std::io::Write;
            let _ = writeln!(f, "[MPRIS] {msg}");
        });
}
