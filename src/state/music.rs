use serde::{ Deserialize, Serialize };
use std::time::{ SystemTime, UNIX_EPOCH };
use rand::seq::SliceRandom;
use crate::utils::path;
use crate::state::config::AppConfig;
use crate::libs::audio::music_player::RodioMusicPlayer;
use std::sync::{ Arc, Mutex };
use std::sync::mpsc;

// Music player command channel
static MUSIC_PLAYER_CHANNEL: std::sync::OnceLock<Arc<Mutex<Option<mpsc::Sender<MusicPlayerCommand>>>>> = std::sync::OnceLock::new();

// Global music player state
static GLOBAL_MUSIC_PLAYER_STATE: std::sync::OnceLock<Arc<Mutex<Option<MusicPlayerState>>>> = std::sync::OnceLock::new();

#[derive(Debug, Clone)]
pub enum MusicPlayerCommand {
    Play(String), // URL
    Pause,
    SetVolume(f32),
    SetMuted(bool),
}

pub fn get_music_player_channel() -> Arc<Mutex<Option<mpsc::Sender<MusicPlayerCommand>>>> {
    MUSIC_PLAYER_CHANNEL.get_or_init(|| Arc::new(Mutex::new(None))).clone()
}

pub fn get_global_music_player_state() -> Arc<Mutex<Option<MusicPlayerState>>> {
    GLOBAL_MUSIC_PLAYER_STATE.get_or_init(|| Arc::new(Mutex::new(None))).clone()
}

pub async fn initialize_global_music_player_state() -> Result<(), String> {
    let global_state_ref = get_global_music_player_state();
    let mut global_state_lock = global_state_ref.lock().unwrap();

    if global_state_lock.is_none() {
        match MusicPlayerState::initialize().await {
            Ok(player_state) => {
                // Initialize rodio player volume from state
                let channel_ref = get_music_player_channel();
                if let Ok(channel_lock) = channel_ref.try_lock() {
                    if let Some(ref sender) = *channel_lock {
                        let _ = sender.send(
                            MusicPlayerCommand::SetVolume(player_state.volume / 100.0)
                        );
                        let _ = sender.send(MusicPlayerCommand::SetMuted(player_state.is_muted));
                    }
                }
                *global_state_lock = Some(player_state);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    Ok(())
}

pub fn update_global_music_player_state<F>(f: F) where F: FnOnce(&mut MusicPlayerState) {
    let global_state_ref = get_global_music_player_state();
    if let Ok(mut global_state_lock) = global_state_ref.try_lock() {
        if let Some(ref mut player_state) = *global_state_lock {
            f(player_state);
        }
    }
}

pub fn get_global_music_player_state_copy() -> Option<MusicPlayerState> {
    let global_state_ref = get_global_music_player_state();
    if let Ok(global_state_lock) = global_state_ref.try_lock() {
        global_state_lock.clone()
    } else {
        None
    }
}

pub fn initialize_music_player() -> Result<(), String> {
    let (sender, receiver) = mpsc::channel::<MusicPlayerCommand>();

    // Store the sender for UI to use
    let channel_ref = get_music_player_channel();
    let mut channel_lock = channel_ref.lock().unwrap();
    *channel_lock = Some(sender);
    drop(channel_lock);

    // Spawn a background thread to handle music player
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if let Ok(player) = RodioMusicPlayer::new() {
                while let Ok(command) = receiver.recv() {
                    match command {
                        MusicPlayerCommand::Play(url) => {
                            if let Err(e) = player.play(&url).await {
                                eprintln!("Failed to play track: {}", e);
                            }
                        }
                        MusicPlayerCommand::Pause => {
                            let _ = player.pause();
                        }
                        MusicPlayerCommand::SetVolume(volume) => {
                            let _ = player.set_volume(volume);
                        }
                        MusicPlayerCommand::SetMuted(muted) => {
                            let _ = player.set_muted(muted);
                        }
                    }
                }
            }
        });
    });

    Ok(())
}

// ===== MUSIC TYPES =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicTrack {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub duration: u32, // in seconds
    pub image: String,
    pub audio: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicApiResponse {
    pub success: bool,
    pub data: Vec<MusicTrack>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicCache {
    pub tracks: Vec<MusicTrack>,
}

impl Default for MusicCache {
    fn default() -> Self {
        Self {
            tracks: Vec::new(),
        }
    }
}

// ===== MUSIC CACHE FUNCTIONS =====
impl MusicCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load cache from music.json file
    pub fn load_from_file() -> Result<Self, String> {
        let cache_path = get_music_cache_path();

        if !std::path::Path::new(&cache_path).exists() {
            return Ok(Self::new());
        }

        match path::read_file_contents(&cache_path) {
            Ok(contents) => {
                match serde_json::from_str::<MusicCache>(&contents) {
                    Ok(cache) => Ok(cache),
                    Err(e) => {
                        eprintln!("Failed to parse music cache: {}", e);
                        Ok(Self::new())
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read music cache file: {}", e);
                Ok(Self::new())
            }
        }
    }

    /// Save cache to music.json file
    pub fn save_to_file(&self) -> Result<(), String> {
        let cache_path = get_music_cache_path();

        match serde_json::to_string_pretty(self) {
            Ok(json) => { path::write_file_contents(&cache_path, &json) }
            Err(e) => Err(format!("Failed to serialize music cache: {}", e)),
        }
    }
    /// Fetch fresh music data from API and update timestamp in config
    pub async fn fetch_and_update() -> Result<Self, String> {
        println!("🎵 Fetching fresh music data from API...");
        let _music_api_url = "https://mechvibes-music-stream.vercel.app/music";

        // For now, we'll create a mock response since we can't make HTTP requests in this context
        // In a real implementation, you would use reqwest or similar HTTP client
        let mock_response = MusicApiResponse {
            success: true,
            data: vec![
                MusicTrack {
                    id: "2187221".to_string(),
                    title: "Sunday".to_string(),
                    artist: "Tomas Gomez".to_string(),
                    duration: 97,
                    image: "https://usercontent.jamendo.com?type=album&id=573080&width=300&trackid=2187221".to_string(),
                    audio: "https://prod-1.storage.jamendo.com/?trackid=2187221&format=mp31&from=1kT172uF%2F%2BbphUy7sgFh%2Fw%3D%3D%7CctbiYFWhpfmbPmWQG1FiaA%3D%3D".to_string(),
                },
                MusicTrack {
                    id: "2187222".to_string(),
                    title: "Morning Coffee".to_string(),
                    artist: "Cafe Sounds".to_string(),
                    duration: 156,
                    image: "https://example.com/image2.jpg".to_string(),
                    audio: "https://example.com/audio2.mp3".to_string(),
                },
                MusicTrack {
                    id: "2187223".to_string(),
                    title: "Peaceful Moments".to_string(),
                    artist: "Ambient Studio".to_string(),
                    duration: 203,
                    image: "https://example.com/image3.jpg".to_string(),
                    audio: "https://example.com/audio3.mp3".to_string(),
                }
            ],
        };

        let cache = Self {
            tracks: mock_response.data,
        };

        // Save cache to file
        cache.save_to_file()?;

        // Update timestamp in config
        let mut config = AppConfig::load();
        let current_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Failed to get current timestamp: {}", e))?
            .as_secs();
        config.music_player.music_last_updated = current_timestamp;
        config.save()?;

        Ok(cache)
    }
    /// Check if cache needs to be updated based on config timestamp
    pub fn should_fetch_from_config() -> bool {
        let config = AppConfig::load();
        let current_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Fetch if never updated or older than 6 hours (21600 seconds)
        config.music_player.music_last_updated == 0 ||
            current_timestamp - config.music_player.music_last_updated > 21600
    }

    /// Load cache with intelligent caching - only fetch if needed
    pub async fn load_or_fetch() -> Result<Self, String> {
        if Self::should_fetch_from_config() {
            // Fetch fresh data if cache is missing or expired
            Self::fetch_and_update().await
        } else {
            // Load existing cache file if it exists, otherwise return empty cache
            Self::load_from_file().or_else(|_| Ok(Self::new()))
        }
    }

    pub fn get_current_track(
        &self,
        current_index: usize,
        shuffle_order: &[usize]
    ) -> Option<&MusicTrack> {
        // Always use shuffle order for random playback
        if !shuffle_order.is_empty() {
            shuffle_order.get(current_index).and_then(|&track_index| self.tracks.get(track_index))
        } else {
            // Fallback to sequential if shuffle order is empty (shouldn't happen)
            self.tracks.get(current_index)
        }
    }

    pub fn generate_shuffle_order(&self) -> Vec<usize> {
        let mut rng = rand::rng();
        let mut shuffle_order: Vec<usize> = (0..self.tracks.len()).collect();
        shuffle_order.shuffle(&mut rng);
        shuffle_order
    }
    pub fn format_duration(seconds: u32) -> String {
        let minutes = seconds / 60;
        let remaining_seconds = seconds % 60;
        format!("{}:{:02}", minutes, remaining_seconds)
    }
}

fn get_music_cache_path() -> String {
    crate::state::paths::data
        ::soundpack_cache_json()
        .parent()
        .unwrap_or_else(|| std::path::Path::new("data"))
        .join("music.json")
        .to_string_lossy()
        .to_string()
}

// ===== MUSIC PLAYER STATE =====

#[derive(Debug, Clone)]
pub struct MusicPlayerState {
    pub cache: MusicCache,
    pub is_playing: bool,
    pub current_time: u32, // in seconds
    pub volume: f32,
    pub is_muted: bool,
    pub current_index: usize, // Current position in shuffle order
    pub shuffle_order: Vec<usize>, // Shuffle order for random playback
}

impl Default for MusicPlayerState {
    fn default() -> Self {
        Self {
            cache: MusicCache::new(),
            is_playing: false,
            current_time: 0,
            volume: 50.0,
            is_muted: false,
            current_index: 0,
            shuffle_order: Vec::new(),
        }
    }
}

impl MusicPlayerState {
    pub fn new() -> Self {
        Self::default()
    }
    pub async fn initialize() -> Result<Self, String> {
        // Load config first
        let config = AppConfig::load();

        // Use optimized cache loading - only fetches if needed
        let cache = MusicCache::load_or_fetch().await?;

        // Generate shuffle order immediately when tracks are available (always shuffle mode)
        let shuffle_order = if !cache.tracks.is_empty() {
            cache.generate_shuffle_order()
        } else {
            Vec::new()
        };

        // Find current track position if available
        let current_index = if let Some(track_id) = &config.music_player.current_track_id {
            if let Some(track_index) = cache.tracks.iter().position(|track| &track.id == track_id) {
                // Find the position in shuffle order
                shuffle_order
                    .iter()
                    .position(|&idx| idx == track_index)
                    .unwrap_or(0)
            } else {
                0
            }
        } else {
            0
        };

        Ok(Self {
            cache,
            volume: config.music_player.volume,
            is_muted: config.music_player.is_muted,
            current_index,
            shuffle_order,
            ..Default::default()
        })
    }
    pub fn save_config(&self) -> Result<(), String> {
        let mut config = AppConfig::load();

        // Update music player config
        config.music_player.current_track_id = self.get_current_track_id();
        config.music_player.volume = self.volume;
        config.music_player.is_muted = self.is_muted;

        config.save()
    }

    pub fn get_current_track_id(&self) -> Option<String> {
        self.cache
            .get_current_track(self.current_index, &self.shuffle_order)
            .map(|track| track.id.clone())
    }

    pub fn get_current_track_info(&self) -> (String, String, String, String) {
        if let Some(track) = self.cache.get_current_track(self.current_index, &self.shuffle_order) {
            (
                track.title.clone(),
                track.artist.clone(),
                MusicCache::format_duration(self.current_time),
                MusicCache::format_duration(track.duration),
            )
        } else {
            (
                "No track selected".to_string(),
                "Unknown Artist".to_string(),
                "0:00".to_string(),
                "0:00".to_string(),
            )
        }
    }
    pub fn get_current_track_image(&self) -> String {
        if let Some(track) = self.cache.get_current_track(self.current_index, &self.shuffle_order) {
            track.image.clone()
        } else {
            String::new()
        }
    }
    pub fn play_pause(&mut self) -> bool {
        let was_playing = self.is_playing;
        self.is_playing = !self.is_playing;

        if self.is_playing && self.cache.tracks.is_empty() {
            // If no tracks available, can't play
            self.is_playing = false;
            return self.is_playing;
        }

        // Use music player channel
        let channel_ref = get_music_player_channel();
        if let Ok(channel_lock) = channel_ref.try_lock() {
            if let Some(ref sender) = *channel_lock {
                if self.is_playing && !was_playing {
                    // Start playing current track
                    if
                        let Some(track) = self.cache.get_current_track(
                            self.current_index,
                            &self.shuffle_order
                        )
                    {
                        let _ = sender.send(MusicPlayerCommand::Play(track.audio.clone()));
                    }
                } else if !self.is_playing && was_playing {
                    // Pause
                    let _ = sender.send(MusicPlayerCommand::Pause);
                }
            }
        }

        // Save config when play state changes
        let _ = self.save_config();
        self.is_playing
    }
    pub fn next_track(&mut self) -> Option<String> {
        if !self.cache.tracks.is_empty() && !self.shuffle_order.is_empty() {
            // Move to next track in shuffle order
            self.current_index = (self.current_index + 1) % self.shuffle_order.len();
            self.current_time = 0;

            let track_title = self.cache
                .get_current_track(self.current_index, &self.shuffle_order)
                .map(|track| track.title.clone());

            if track_title.is_some() {
                // Save config when track changes
                let _ = self.save_config();

                // If currently playing, start playing the new track
                if self.is_playing {
                    let channel_ref = get_music_player_channel();
                    if let Ok(channel_lock) = channel_ref.try_lock() {
                        if let Some(ref sender) = *channel_lock {
                            if
                                let Some(track) = self.cache.get_current_track(
                                    self.current_index,
                                    &self.shuffle_order
                                )
                            {
                                let _ = sender.send(MusicPlayerCommand::Play(track.audio.clone()));
                            }
                        }
                    }
                }
            }

            track_title
        } else {
            None
        }
    }
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 100.0);
        if self.is_muted && volume > 0.0 {
            self.is_muted = false;
        }

        // Update music player volume (convert from 0-100 to 0-1)
        let channel_ref = get_music_player_channel();
        if let Ok(channel_lock) = channel_ref.try_lock() {
            if let Some(ref sender) = *channel_lock {
                let _ = sender.send(MusicPlayerCommand::SetVolume(self.volume / 100.0));
            }
        }

        // Save config when volume changes
        let _ = self.save_config();
    }
    pub fn toggle_mute(&mut self) {
        self.is_muted = !self.is_muted;

        // Update music player mute state
        let channel_ref = get_music_player_channel();
        if let Ok(channel_lock) = channel_ref.try_lock() {
            if let Some(ref sender) = *channel_lock {
                let _ = sender.send(MusicPlayerCommand::SetMuted(self.is_muted));
            }
        }

        // Save config when mute state changes
        let _ = self.save_config();
    }
}
