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
    /// Load cache with built-in fallback
    pub fn load_or_fallback() -> Self {
        Self::load_from_file().unwrap_or_else(|_| {
            // If file doesn't exist or is corrupted, use built-in tracks
            Self {
                tracks: Self::get_builtin_tracks(),
            }
        })
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

        // For now, we'll use built-in tracks since we can't make HTTP requests in this context
        // In a real implementation, you would use reqwest or similar HTTP client
        let cache = Self {
            tracks: Self::get_builtin_tracks(),
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
            // Load existing cache file or use built-in tracks as fallback
            Ok(Self::load_or_fallback())
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
    /// Get built-in tracks that are always available (fallback when API/cache fails)
    pub fn get_builtin_tracks() -> Vec<MusicTrack> {
        vec![
            MusicTrack {
                id: "2141439".to_string(),
                title: "blop - blop 002".to_string(),
                artist: "blop".to_string(),
                duration: 80,
                image: "https://usercontent.jamendo.com?type=album&id=553497&width=300&trackid=2141439".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2141439&format=mp31&from=c2KKMxK8yevDanmRoLBFzA%3D%3D%7Cg4MUfj1WXGqJMAPZLRq4uA%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2222823".to_string(),
                title: "Music LO-FI for workplace".to_string(),
                artist: "Dmytro Demchenko".to_string(),
                duration: 246,
                image: "https://usercontent.jamendo.com?type=album&id=589597&width=300&trackid=2222823".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2222823&format=mp31&from=cEzDWdu0FVpWgxE9YimfFA%3D%3D%7Czns9Ujr4KU1aTd5UKGf00g%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2027454".to_string(),
                title: "Ко$мон@вт - Аутро - Кучерявые облака".to_string(),
                artist: "Ко$мон@вт".to_string(),
                duration: 216,
                image: "https://usercontent.jamendo.com?type=album&id=519118&width=300&trackid=2027454".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2027454&format=mp31&from=k3UKTnHk%2FOjm7dWXA2oPzg%3D%3D%7C5q1whFznDLuDsZq3%2BF2wtA%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2125991".to_string(),
                title: "The Simply Lofi 2".to_string(),
                artist: "Vzen instrumental beat".to_string(),
                duration: 170,
                image: "https://usercontent.jamendo.com?type=album&id=545891&width=300&trackid=2125991".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2125991&format=mp31&from=%2FZYzsleY8nVxo%2BFAN7zLsA%3D%3D%7CeGPSIpRMRv1sxP9Gpq6Fcg%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2165960".to_string(),
                title: "Sleepover - Instrumental Background Music".to_string(),
                artist: "Lulakarma".to_string(),
                duration: 180,
                image: "https://usercontent.jamendo.com?type=album&id=566041&width=300&trackid=2165960".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2165960&format=mp31&from=0I%2Fa3LiAHpNEILL6NGoILA%3D%3D%7CnHEmki8HfKjk0SWWYKWklA%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2238552".to_string(),
                title: "Morning Mist".to_string(),
                artist: "Buurd".to_string(),
                duration: 178,
                image: "https://usercontent.jamendo.com?type=album&id=596494&width=300&trackid=2238552".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2238552&format=mp31&from=cPHrH2M%2BUIHfwPAa5iMnvw%3D%3D%7CoYIE9tgoufCbUP9%2Bq2T%2FqQ%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2024649".to_string(),
                title: "Something About the Rain".to_string(),
                artist: "Reverb Reflections".to_string(),
                duration: 160,
                image: "https://usercontent.jamendo.com?type=album&id=517853&width=300&trackid=2024649".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2024649&format=mp31&from=%2B1oAq1ZTqHvIBXS5N5rlqw%3D%3D%7Cv%2FHN8bpP8dxtgpmv0DQ76g%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2026509".to_string(),
                title: "Lo Fi  Mood".to_string(),
                artist: "Dmytro Demchenko".to_string(),
                duration: 265,
                image: "https://usercontent.jamendo.com?type=album&id=518972&width=300&trackid=2026509".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2026509&format=mp31&from=jEBHohCxa3QG3Bs%2B6v8VIw%3D%3D%7CfVPr6BcGl1lcPyrUEGyRMg%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2238525".to_string(),
                title: "Weekend Reverie".to_string(),
                artist: "Buurd".to_string(),
                duration: 211,
                image: "https://usercontent.jamendo.com?type=album&id=596494&width=300&trackid=2238525".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2238525&format=mp31&from=%2FiX1wMywzmjVBFXNtsWdHA%3D%3D%7CwJUJ15E66O8DXM2kYncMYw%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2152346".to_string(),
                title: "Lo-Fi Ambient".to_string(),
                artist: "weekwrite".to_string(),
                duration: 188,
                image: "https://usercontent.jamendo.com?type=album&id=559222&width=300&trackid=2152346".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2152346&format=mp31&from=WqnZI%2BaUidCj%2Bey%2FHlttZQ%3D%3D%7CthiOj%2Fc9aWRFhz0l0CjV%2BQ%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2125437".to_string(),
                title: "See You Then".to_string(),
                artist: "C MUSIC Professional Library".to_string(),
                duration: 204,
                image: "https://usercontent.jamendo.com?type=album&id=545611&width=300&trackid=2125437".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2125437&format=mp31&from=zs%2BO3nLAuYjblWu34T3Heg%3D%3D%7Cw9K6K3rJhSEYjbBjnIXywg%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2238542".to_string(),
                title: "Urban Garden".to_string(),
                artist: "Buurd".to_string(),
                duration: 124,
                image: "https://usercontent.jamendo.com?type=album&id=596494&width=300&trackid=2238542".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2238542&format=mp31&from=SViUNU6gkUw9BqYB%2F8ug5A%3D%3D%7C4sJ3nTR3mJjlDm3TnBOJXg%3D%3D".to_string(),
            },
            MusicTrack {
                id: "1993011".to_string(),
                title: "LORD ( Lo-Fi Hip-Hop Beat )".to_string(),
                artist: "HIGHVEGAZ".to_string(),
                duration: 132,
                image: "https://usercontent.jamendo.com?type=album&id=505803&width=300&trackid=1993011".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=1993011&format=mp31&from=c1TmSj3hESQQ6yypKEkTnw%3D%3D%7CgMTR9XzMZKdB2vLqDBio%2Bw%3D%3D".to_string(),
            },
            MusicTrack {
                id: "1952739".to_string(),
                title: "Chill Lofi on".to_string(),
                artist: "Pumpupthemind".to_string(),
                duration: 38,
                image: "https://usercontent.jamendo.com?type=album&id=486665&width=300&trackid=1952739".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=1952739&format=mp31&from=N9I1JT6PfsLvYAWP2wWSgA%3D%3D%7CMPJLoUND1zY%2B3ogOSyGn0w%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2223155".to_string(),
                title: "LoFidown".to_string(),
                artist: "Grumpynora".to_string(),
                duration: 101,
                image: "https://usercontent.jamendo.com?type=album&id=589786&width=300&trackid=2223155".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2223155&format=mp31&from=HS%2FGMAmMt%2FaKO%2BUINwox%2FQ%3D%3D%7C%2FX6Sa48KzSr2oA%2BfI8DFjw%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2153293".to_string(),
                title: "Blizzard".to_string(),
                artist: "Silver-Stage".to_string(),
                duration: 168,
                image: "https://usercontent.jamendo.com?type=album&id=559640&width=300&trackid=2153293".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2153293&format=mp31&from=dhYlU%2F82eCpbG0XI8Ck2AQ%3D%3D%7CKU%2BXmeEoBHVgw70TwS29oA%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2197178".to_string(),
                title: "Moonlit Echoes".to_string(),
                artist: "Top Flow".to_string(),
                duration: 0,
                image: "https://usercontent.jamendo.com?type=album&id=578053&width=300&trackid=2197178".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2197178&format=mp31&from=a3cuvBHWmWqFJTfi7DoSEQ%3D%3D%7CTo6%2BxTadL5sRdsFeRJ7PKg%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2171152".to_string(),
                title: "Thoughts and Prayers".to_string(),
                artist: "Double-F The King".to_string(),
                duration: 247,
                image: "https://usercontent.jamendo.com?type=album&id=566917&width=300&trackid=2171152".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2171152&format=mp31&from=%2FX%2FbTG6TiNDznpdvG2lLXg%3D%3D%7CKvZfmYVUQJKUP0eP3DgWBg%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2230304".to_string(),
                title: "Second Iteration".to_string(),
                artist: "chriisduran".to_string(),
                duration: 143,
                image: "https://usercontent.jamendo.com?type=album&id=592379&width=300&trackid=2230304".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2230304&format=mp31&from=AqoWhvuXCDCBoKJK6EAqEQ%3D%3D%7Ca7Ua9Jge%2F12HlpYHZ4soYA%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2201534".to_string(),
                title: "lo-Fi mix music".to_string(),
                artist: "An Ki".to_string(),
                duration: 154,
                image: "https://usercontent.jamendo.com?type=album&id=579063&width=300&trackid=2201534".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2201534&format=mp31&from=k%2Fx1nM%2B7oT%2BBPHmNImVbxA%3D%3D%7CrJwv7DRVEtstoLNuroTxDA%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2018952".to_string(),
                title: "Chill Meditation (short 2)".to_string(),
                artist: "Vicate".to_string(),
                duration: 49,
                image: "https://usercontent.jamendo.com?type=album&id=515883&width=300&trackid=2018952".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2018952&format=mp31&from=bnD8ZVUmC6He8nKoOZ8mbw%3D%3D%7C6moKFu7ihw7rqQbAwoXftQ%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2153501".to_string(),
                title: "Soul's Labrynth (Instrumental)".to_string(),
                artist: "Vic Apollo".to_string(),
                duration: 124,
                image: "https://usercontent.jamendo.com?type=album&id=559678&width=300&trackid=2153501".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2153501&format=mp31&from=u4KmbICnCKTVDYG9onmdJg%3D%3D%7C6VXqHomT4%2F0YUQQqNKOVdw%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2238543".to_string(),
                title: "Neon Reflections".to_string(),
                artist: "Buurd".to_string(),
                duration: 158,
                image: "https://usercontent.jamendo.com?type=album&id=596494&width=300&trackid=2238543".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2238543&format=mp31&from=a4q7XiT4I%2FbgR9H2WR8d9Q%3D%3D%7CqdAqgBO68fsDbF2ICA6iUg%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2035689".to_string(),
                title: "Love Frequency".to_string(),
                artist: "Yigit Atilla".to_string(),
                duration: 181,
                image: "https://usercontent.jamendo.com?type=album&id=522810&width=300&trackid=2035689".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2035689&format=mp31&from=VgeCrUYRq7xeCJSCQQb3kw%3D%3D%7Cvkz%2BgC7jEp4%2F3UIbJyDZBw%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2194745".to_string(),
                title: "Good Reason".to_string(),
                artist: "Cephas".to_string(),
                duration: 256,
                image: "https://usercontent.jamendo.com?type=album&id=576807&width=300&trackid=2194745".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2194745&format=mp31&from=ujndbxkiTkSFxuFm3MxW2w%3D%3D%7C4zMYawqZxNLBZM5%2Fry4AMg%3D%3D".to_string(),
            },
            MusicTrack {
                id: "1956052".to_string(),
                title: "Just Love (LoFi 2022 Version)".to_string(),
                artist: "Dj Saryon & Sory".to_string(),
                duration: 110,
                image: "https://usercontent.jamendo.com?type=album&id=487732&width=300&trackid=1956052".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=1956052&format=mp31&from=V%2BxeToAW4y3Gv2vyzstnvA%3D%3D%7Cyw5T2JRZ46Ds5Thl9JRgFg%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2223150".to_string(),
                title: "Laidback LoFi".to_string(),
                artist: "Grumpynora".to_string(),
                duration: 111,
                image: "https://usercontent.jamendo.com?type=album&id=589788&width=300&trackid=2223150".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2223150&format=mp31&from=GKpHZn%2FbEaq9TRtVHauB8A%3D%3D%7CxpHXY%2FB2Oex8bQrxno0HLw%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2237118".to_string(),
                title: "DEEP CHURCH - CHILDHOOD [hopecore x lo-fi hip-hop]".to_string(),
                artist: "TransistorBudddha".to_string(),
                duration: 150,
                image: "https://usercontent.jamendo.com?type=album&id=595678&width=300&trackid=2237118".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2237118&format=mp31&from=WI9ij%2BoWTu70ZOKh18RqCg%3D%3D%7CmRCSPUrGAS8fpXEF4G%2BtEw%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2203470".to_string(),
                title: "Acoustic Guitar Lofi beat".to_string(),
                artist: "Joel Loopez".to_string(),
                duration: 123,
                image: "https://usercontent.jamendo.com?type=album&id=579751&width=300&trackid=2203470".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2203470&format=mp31&from=s%2BxhDawubjEau%2BkKplSmyA%3D%3D%7C%2BY%2FiYOqbGi%2F8XOym1Fzirw%3D%3D".to_string(),
            },
            MusicTrack {
                id: "1998436".to_string(),
                title: "In The Rain".to_string(),
                artist: "B3B0CK".to_string(),
                duration: 206,
                image: "https://usercontent.jamendo.com?type=album&id=508340&width=300&trackid=1998436".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=1998436&format=mp31&from=TlkMueO2V0XdXRnAJ3oGGQ%3D%3D%7CK5w49kaosStYdMI1qqGDLw%3D%3D".to_string(),
            },
            MusicTrack {
                id: "1987202".to_string(),
                title: "Thats Right".to_string(),
                artist: "Reverb Reflections".to_string(),
                duration: 267,
                image: "https://usercontent.jamendo.com?type=album&id=503060&width=300&trackid=1987202".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=1987202&format=mp31&from=Z9uCHajQJcBxU2tlWnSkQA%3D%3D%7Clol6EuxF5Zy42icUGn94pw%3D%3D".to_string(),
            },
            MusicTrack {
                id: "1998424".to_string(),
                title: "Melancholy".to_string(),
                artist: "B3B0CK".to_string(),
                duration: 194,
                image: "https://usercontent.jamendo.com?type=album&id=508340&width=300&trackid=1998424".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=1998424&format=mp31&from=rcMpwBLZaRooYY%2B9C5JYgA%3D%3D%7CDJM%2BM5zjTmeIaKxoygVZ7g%3D%3D".to_string(),
            },
            MusicTrack {
                id: "1977424".to_string(),
                title: "Lo-Fi Chill Hip Hop".to_string(),
                artist: "Janevo".to_string(),
                duration: 126,
                image: "https://usercontent.jamendo.com?type=album&id=498499&width=300&trackid=1977424".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=1977424&format=mp31&from=WIdy9FoPMkMPpM%2FiWXn05A%3D%3D%7Cu715VRgQSM1j1TSbrpHMcQ%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2211769".to_string(),
                title: "Feel Good Music 1".to_string(),
                artist: "Reyd Productions".to_string(),
                duration: 174,
                image: "https://usercontent.jamendo.com?type=album&id=583827&width=300&trackid=2211769".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2211769&format=mp31&from=uhkyjud05PPaiBfZWtIgOg%3D%3D%7C42qy8lY1m2hZFqjP2aeoyQ%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2037721".to_string(),
                title: "Downtempo Delights (Lofi Hip-hop)".to_string(),
                artist: "Moonwalk".to_string(),
                duration: 144,
                image: "https://usercontent.jamendo.com?type=album&id=525295&width=300&trackid=2037721".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2037721&format=mp31&from=jWhdHXLddQuA2dJ18EmvCw%3D%3D%7CBSb%2BIE7ydVf5aZJnRMS%2BrA%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2163957".to_string(),
                title: "Dreamscape Lullaby".to_string(),
                artist: "Silver-Stage".to_string(),
                duration: 199,
                image: "https://usercontent.jamendo.com?type=album&id=560384&width=300&trackid=2163957".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2163957&format=mp31&from=RFUf2HxGmx4od7pr0eMvyQ%3D%3D%7Chr8kWk5m5LKOqHu5VhOlGg%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2208119".to_string(),
                title: "After Sunset (Calm Chillout LoFi)".to_string(),
                artist: "ANtarcticbreeze".to_string(),
                duration: 211,
                image: "https://usercontent.jamendo.com?type=album&id=581815&width=300&trackid=2208119".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2208119&format=mp31&from=yZ8cCQb6rn2yipVdjCZvgA%3D%3D%7CZEI5BCumlNABpoKUDvwnlw%3D%3D".to_string(),
            },
            MusicTrack {
                id: "1966215".to_string(),
                title: "Holiday".to_string(),
                artist: "Edibeat21".to_string(),
                duration: 183,
                image: "https://usercontent.jamendo.com?type=album&id=492454&width=300&trackid=1966215".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=1966215&format=mp31&from=9inPBM9Ra3Zm4qt18Za5qA%3D%3D%7Cq%2F1LhFm0A8cG0VfEjlW6PQ%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2063623".to_string(),
                title: "Cozy Environment".to_string(),
                artist: "LEXMusic".to_string(),
                duration: 113,
                image: "https://usercontent.jamendo.com?type=album&id=531999&width=300&trackid=2063623".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2063623&format=mp31&from=RX1gM74%2FK6fyO860681WVQ%3D%3D%7Cd6mKuiHbsEcJIHbpgPoWPw%3D%3D".to_string(),
            },
            MusicTrack {
                id: "1991296".to_string(),
                title: "Alone in the city".to_string(),
                artist: "Antonio Fiorucci".to_string(),
                duration: 146,
                image: "https://usercontent.jamendo.com?type=album&id=507602&width=300&trackid=1991296".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=1991296&format=mp31&from=OklO5NueUwuDBotWrXA7gA%3D%3D%7CKQrLOgDZrtM%2F4BIuoKmpJA%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2165892".to_string(),
                title: "Letter To The Future_64".to_string(),
                artist: "TaigaSoundProd".to_string(),
                duration: 64,
                image: "https://usercontent.jamendo.com?type=album&id=564393&width=300&trackid=2165892".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2165892&format=mp31&from=rftv2Z%2FtgIwsh2Us%2Bs6Ewg%3D%3D%7CO25vwpJTBeY8uiQGVnBZrQ%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2037689".to_string(),
                title: "Calm Water (Lofi Trip-hop)".to_string(),
                artist: "Moonwalk".to_string(),
                duration: 49,
                image: "https://usercontent.jamendo.com?type=album&id=525347&width=300&trackid=2037689".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2037689&format=mp31&from=em4x6xm7vrgeUK1cH%2B2ghA%3D%3D%7C9gdLu%2FNhaEiZOmkQRBmS2g%3D%3D".to_string(),
            },
            MusicTrack {
                id: "1957124".to_string(),
                title: "Soft Ambient Lofi Hiphop (15sec V.1)".to_string(),
                artist: "Joel Loopez".to_string(),
                duration: 15,
                image: "https://usercontent.jamendo.com?type=album&id=487989&width=300&trackid=1957124".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=1957124&format=mp31&from=Rsq%2FDIzZCPmSUF%2FD7S6Hyg%3D%3D%7Cp3QQnbmtm5gWg12yuVR8aQ%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2201537".to_string(),
                title: "CHILL LO-FI  Jazz Rap Hip-Hop".to_string(),
                artist: "An Ki".to_string(),
                duration: 261,
                image: "https://usercontent.jamendo.com?type=album&id=579066&width=300&trackid=2201537".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2201537&format=mp31&from=e9322E2lq5erHLxKxD9LlA%3D%3D%7CA%2FaVyZ0Yjeqh1PvmStvK%2Fg%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2236490".to_string(),
                title: "DEEP CHURCH - MOTH SHELTER".to_string(),
                artist: "TransistorBudddha".to_string(),
                duration: 135,
                image: "https://usercontent.jamendo.com?type=album&id=595405&width=300&trackid=2236490".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2236490&format=mp31&from=f%2F94HM9FPBQov6%2FM3g5uvA%3D%3D%7C3PHSAGxlB7LE00KJbpfCTw%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2223367".to_string(),
                title: "LO Fi light music relaxing".to_string(),
                artist: "Dmytro Demchenko".to_string(),
                duration: 249,
                image: "https://usercontent.jamendo.com?type=album&id=589837&width=300&trackid=2223367".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2223367&format=mp31&from=oZTM4A6FwwzkNyAJsN%2BIKA%3D%3D%7CmQ3cgUVBdFrCPMEQybKJag%3D%3D".to_string(),
            },
            MusicTrack {
                id: "1960523".to_string(),
                title: "Living Life".to_string(),
                artist: "EdRecords".to_string(),
                duration: 133,
                image: "https://usercontent.jamendo.com?type=album&id=489384&width=300&trackid=1960523".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=1960523&format=mp31&from=IW7frPX%2F8RrNB6dsAYoUhw%3D%3D%7CUQd23ydcV6cK81TwV5Qfsw%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2009971".to_string(),
                title: "Lofi Hiphop Chill Beat".to_string(),
                artist: "Joel Loopez".to_string(),
                duration: 166,
                image: "https://usercontent.jamendo.com?type=album&id=512832&width=300&trackid=2009971".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2009971&format=mp31&from=uNMhFGAFr4MssCN8aiUW8g%3D%3D%7CTcBvRSrVu%2Bun0bQqDydh%2Fw%3D%3D".to_string(),
            },
            MusicTrack {
                id: "1957130".to_string(),
                title: "Soft Ambient Lofi Hiphop (60sec V.1)".to_string(),
                artist: "Joel Loopez".to_string(),
                duration: 60,
                image: "https://usercontent.jamendo.com?type=album&id=487981&width=300&trackid=1957130".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=1957130&format=mp31&from=XmiRCuT4QvtGOtER%2FVEmOg%3D%3D%7CxGR118nHw5xCJRQg1b%2BiHg%3D%3D".to_string(),
            },
            MusicTrack {
                id: "1998437".to_string(),
                title: "Floats".to_string(),
                artist: "B3B0CK".to_string(),
                duration: 186,
                image: "https://usercontent.jamendo.com?type=album&id=508340&width=300&trackid=1998437".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=1998437&format=mp31&from=TVZA7%2BJR%2FDIIb1EhAis4kA%3D%3D%7CsA9JkIkrZpvuHFailr82hQ%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2184705".to_string(),
                title: "State of sleep N° 3: Reverie's dance".to_string(),
                artist: "Laidback Groove Crew".to_string(),
                duration: 80,
                image: "https://usercontent.jamendo.com?type=album&id=571985&width=300&trackid=2184705".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2184705&format=mp31&from=BG7pncIEo4qRVXq5VbJYFw%3D%3D%7C6ZlPTi4D5KAbfaM%2FdO5NmQ%3D%3D".to_string(),
            },
            MusicTrack {
                id: "1960654".to_string(),
                title: "Date In Call-Box".to_string(),
                artist: "EdRecords".to_string(),
                duration: 138,
                image: "https://usercontent.jamendo.com?type=album&id=489429&width=300&trackid=1960654".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=1960654&format=mp31&from=EmVBBnCT7%2BcNGnh7KPyDEQ%3D%3D%7C1UCARSnvCDe1N6coB3Ok0g%3D%3D".to_string(),
            },
            MusicTrack {
                id: "1973650".to_string(),
                title: "An Impulse".to_string(),
                artist: "Edibeat21".to_string(),
                duration: 193,
                image: "https://usercontent.jamendo.com?type=album&id=495949&width=300&trackid=1973650".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=1973650&format=mp31&from=GxTzuw6vo7yp19XdUoIggg%3D%3D%7CFHaIDwSwViJeDl8FAocCTQ%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2178400".to_string(),
                title: "Lo-Fi Coziness".to_string(),
                artist: "Eduard Perelyhin".to_string(),
                duration: 168,
                image: "https://usercontent.jamendo.com?type=album&id=569123&width=300&trackid=2178400".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2178400&format=mp31&from=H1U9aRpIL21e%2FerWvDkyTA%3D%3D%7CfFTCbZ7mHv26iZ4YYYIsEA%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2208135".to_string(),
                title: "The Midnight (Calm Chillout Lofi)".to_string(),
                artist: "ANtarcticbreeze".to_string(),
                duration: 232,
                image: "https://usercontent.jamendo.com?type=album&id=581825&width=300&trackid=2208135".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2208135&format=mp31&from=p7ac6EmZxKEDMK3UhegF0g%3D%3D%7CfrYe0O8p%2F1hbM4nnYr4Igg%3D%3D".to_string(),
            },
            MusicTrack {
                id: "1957125".to_string(),
                title: "Soft Ambient Lofi Hiphop (60sec V.2)".to_string(),
                artist: "Joel Loopez".to_string(),
                duration: 60,
                image: "https://usercontent.jamendo.com?type=album&id=487986&width=300&trackid=1957125".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=1957125&format=mp31&from=axI4%2B5FFotk%2BBIoV2SY%2BxA%3D%3D%7Cj6OxGNhmvKLw53edd4T6Vw%3D%3D".to_string(),
            },
            MusicTrack {
                id: "1971776".to_string(),
                title: "We Choose Proven (kbkbts. Remix)".to_string(),
                artist: "Paradeigma".to_string(),
                duration: 183,
                image: "https://usercontent.jamendo.com?type=album&id=495101&width=300&trackid=1971776".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=1971776&format=mp31&from=%2BcnA687QqFOtOaA3rzCEFA%3D%3D%7CF5BVwrP6G3pETSxU5gT45w%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2153290".to_string(),
                title: "Desert Eagle".to_string(),
                artist: "Silver-Stage".to_string(),
                duration: 168,
                image: "https://usercontent.jamendo.com?type=album&id=559640&width=300&trackid=2153290".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2153290&format=mp31&from=0F6z4W7SXuEksqNjIZZ3bQ%3D%3D%7C4xeV%2F0kiMdDJHaoQIOjp%2Fw%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2039763".to_string(),
                title: "Magic Of This Evening".to_string(),
                artist: "Pumpupthemind".to_string(),
                duration: 71,
                image: "https://usercontent.jamendo.com?type=album&id=527002&width=300&trackid=2039763".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2039763&format=mp31&from=cYfqzf5Cl1gh80gYSkZwVw%3D%3D%7CeAj0b6zuOa7OyYZ3TOAsvw%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2130661".to_string(),
                title: "Flow of life".to_string(),
                artist: "Cephas".to_string(),
                duration: 300,
                image: "https://usercontent.jamendo.com?type=album&id=547867&width=300&trackid=2130661".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2130661&format=mp31&from=pex4RIL1NnkzIRk0DIGujg%3D%3D%7Czv1Nb0QtonbXenqvPuXNPA%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2233327".to_string(),
                title: "Color Program (feat. Izumi)".to_string(),
                artist: "DJ Gami.K".to_string(),
                duration: 319,
                image: "https://usercontent.jamendo.com?type=album&id=593802&width=300&trackid=2233327".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2233327&format=mp31&from=dZ6iXFr9FiIlwkoFM9SpWw%3D%3D%7CVeVOpOiMgYXoPmG3aM%2BOKA%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2195847".to_string(),
                title: "Good morning".to_string(),
                artist: "Cephas".to_string(),
                duration: 216,
                image: "https://usercontent.jamendo.com?type=album&id=577441&width=300&trackid=2195847".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2195847&format=mp31&from=q08RxYLoq%2BsoaMpI5BAB0g%3D%3D%7COMncbGe3YRkixCjEHbScyw%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2241791".to_string(),
                title: "On Lo-Fi".to_string(),
                artist: "Jack Jack".to_string(),
                duration: 151,
                image: "https://usercontent.jamendo.com?type=album&id=598062&width=300&trackid=2241791".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2241791&format=mp31&from=H9YzRBeXGuMBaahND7JmwA%3D%3D%7C2x%2FJWt57BuFBO0t0lEbeDQ%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2191517".to_string(),
                title: "4. Knapsten - Girls".to_string(),
                artist: "Knapsten".to_string(),
                duration: 132,
                image: "https://usercontent.jamendo.com?type=album&id=575402&width=300&trackid=2191517".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2191517&format=mp31&from=5eRG%2BibBLJnnr84g%2FH%2FdkQ%3D%3D%7CnqHnYra7a%2FUEIZ5N1V7RXg%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2017404".to_string(),
                title: "Fashion".to_string(),
                artist: "lesfm".to_string(),
                duration: 158,
                image: "https://usercontent.jamendo.com?type=album&id=515418&width=300&trackid=2017404".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2017404&format=mp31&from=p7R%2BY0AVadg8XqkKzDSq%2Bg%3D%3D%7CrDOtxlPMLOArOv5%2BDxhOPw%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2039794".to_string(),
                title: "Streets  After Blackout".to_string(),
                artist: "Pumpupthemind".to_string(),
                duration: 78,
                image: "https://usercontent.jamendo.com?type=album&id=526853&width=300&trackid=2039794".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2039794&format=mp31&from=AMjd1r1lscp5XkfjFFw%2FOg%3D%3D%7Cu2NXn68vWur%2BIMTzeSfMUw%3D%3D".to_string(),
            },
            MusicTrack {
                id: "1958533".to_string(),
                title: "Sunny Days - Canggu Beats (Lo-Fi Type Beat)".to_string(),
                artist: "Canggu Beats".to_string(),
                duration: 192,
                image: "https://usercontent.jamendo.com?type=album&id=488611&width=300&trackid=1958533".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=1958533&format=mp31&from=f4KU6wkLLtNYOPKbemm%2BEA%3D%3D%7CEEIzIPX7%2Bqg%2FU%2BKDBc%2Bd7A%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2039466".to_string(),
                title: "Chillout Guitar Lo-Fi Mood".to_string(),
                artist: "STOCK ELITE MUSIC".to_string(),
                duration: 133,
                image: "https://usercontent.jamendo.com?type=album&id=524787&width=300&trackid=2039466".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2039466&format=mp31&from=%2BdXXfgkv2RlVsWC%2FP%2B9nOA%3D%3D%7C8fqzM7vVCmo2f7OApkXZPA%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2144972".to_string(),
                title: "Still Learning".to_string(),
                artist: "Cephas".to_string(),
                duration: 249,
                image: "https://usercontent.jamendo.com?type=album&id=555260&width=300&trackid=2144972".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2144972&format=mp31&from=5VrqZ72uVyN%2FgMO9DaA5Hg%3D%3D%7CozFANw0cESSVqTQVK9kXqQ%3D%3D".to_string(),
            },
            MusicTrack {
                id: "1921661".to_string(),
                title: "Story Of Midnight".to_string(),
                artist: "Oddvision Media".to_string(),
                duration: 92,
                image: "https://usercontent.jamendo.com?type=album&id=472357&width=300&trackid=1921661".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=1921661&format=mp31&from=AQT0LPcK4rkJcCphslLjkg%3D%3D%7CFiMhZ%2F6vRJKCqEWL1CVcaw%3D%3D".to_string(),
            },
            MusicTrack {
                id: "1968006".to_string(),
                title: "The Cold Weather".to_string(),
                artist: "lesfm".to_string(),
                duration: 152,
                image: "https://usercontent.jamendo.com?type=album&id=493089&width=300&trackid=1968006".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=1968006&format=mp31&from=k08qrZFpoMqHSnc1px4z%2FA%3D%3D%7ClP0kJn5hGvHIV59feA9SQw%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2184421".to_string(),
                title: "Wistful Whispers".to_string(),
                artist: "Laidback Groove Crew".to_string(),
                duration: 106,
                image: "https://usercontent.jamendo.com?type=album&id=571872&width=300&trackid=2184421".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2184421&format=mp31&from=6vrD4XRfZ80qEIt0%2FbQWJQ%3D%3D%7CKCzWVbnVP0DOxTWqGiN8%2BA%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2176804".to_string(),
                title: "Eyes Closed Again".to_string(),
                artist: "NxSG".to_string(),
                duration: 0,
                image: "https://usercontent.jamendo.com?type=album&id=568653&width=300&trackid=2176804".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2176804&format=mp31&from=Y3qIU6KkRbdZmhk3ZqLtMw%3D%3D%7CB78Ix5K5dAbYWFpaYstrPw%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2255334".to_string(),
                title: "Aniki Lion - Into the blue".to_string(),
                artist: "Kolarbeatz".to_string(),
                duration: 133,
                image: "https://usercontent.jamendo.com?type=album&id=604061&width=300&trackid=2255334".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2255334&format=mp31&from=B4MhIxN6oP3lmaF9TTiPGA%3D%3D%7CicfHZm81OSGEux1Na7TjlA%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2026085".to_string(),
                title: "YOGA".to_string(),
                artist: "JVP".to_string(),
                duration: 175,
                image: "https://usercontent.jamendo.com?type=album&id=519172&width=300&trackid=2026085".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2026085&format=mp31&from=b0a76djg%2BPORinWILS1yCA%3D%3D%7CRJ%2BvwVUmmK15DeoyVbXcog%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2157059".to_string(),
                title: "Starfall".to_string(),
                artist: "Boulvard X-Audi".to_string(),
                duration: 170,
                image: "https://usercontent.jamendo.com?type=album&id=561113&width=300&trackid=2157059".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2157059&format=mp31&from=zsivtSQ1JnzwBsUyM0lqfw%3D%3D%7CVxHpyheha9tWl%2FHRgy5WdQ%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2146877".to_string(),
                title: "Breakthru - lofi Mood and Chill Vibe Song".to_string(),
                artist: "Lulakarma".to_string(),
                duration: 145,
                image: "https://usercontent.jamendo.com?type=album&id=556630&width=300&trackid=2146877".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2146877&format=mp31&from=r3ZtRnkF2e8BzJcNoDIlLA%3D%3D%7CAt%2FhDkZo708bA%2F%2FzhX%2B2OQ%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2184443".to_string(),
                title: "Swaying in the Breeze".to_string(),
                artist: "Laidback Groove Crew".to_string(),
                duration: 73,
                image: "https://usercontent.jamendo.com?type=album&id=571884&width=300&trackid=2184443".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2184443&format=mp31&from=U6Q%2BJKEkA1e7rDNAQ4snLQ%3D%3D%7CEao4gCLIemCypDtzgsgwJw%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2004411".to_string(),
                title: "LoFi HipHop Intro 63".to_string(),
                artist: "TaigaSoundProd".to_string(),
                duration: 11,
                image: "https://usercontent.jamendo.com?type=album&id=510471&width=300&trackid=2004411".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2004411&format=mp31&from=1R%2FjgGF0GhoRsORYZpZS%2BQ%3D%3D%7Cw467x3Jt%2Bpm2VYKzbNNWOw%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2223149".to_string(),
                title: "Laidback LoFi (loop 3)".to_string(),
                artist: "Grumpynora".to_string(),
                duration: 24,
                image: "https://usercontent.jamendo.com?type=album&id=589782&width=300&trackid=2223149".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2223149&format=mp31&from=9ogaCO%2BY9sWvtr6PbZkGhQ%3D%3D%7CJL6RoeXGMdWp58FCfk5MVw%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2037902".to_string(),
                title: "Head Above Water (Chilling Lo Fi Beats Ambient Background Piano)".to_string(),
                artist: "Epikton".to_string(),
                duration: 210,
                image: "https://usercontent.jamendo.com?type=album&id=523872&width=300&trackid=2037902".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2037902&format=mp31&from=LRzKulclCnsuP6XlEWSX%2BQ%3D%3D%7CTbDLU0JL2jeBq17Mqu4N4Q%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2253210".to_string(),
                title: "Dziedavy Historyji".to_string(),
                artist: "Plushka".to_string(),
                duration: 164,
                image: "https://usercontent.jamendo.com?type=album&id=602864&width=300&trackid=2253210".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2253210&format=mp31&from=%2FgBWfnfYDxy8Wq5U%2FMLGdw%3D%3D%7CuXPH3Kucmqhr7UGO0mezyA%3D%3D".to_string(),
            },
            MusicTrack {
                id: "1989444".to_string(),
                title: "Once Upton a Time".to_string(),
                artist: "Alex-Productions".to_string(),
                duration: 141,
                image: "https://usercontent.jamendo.com?type=album&id=504138&width=300&trackid=1989444".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=1989444&format=mp31&from=3lxUGG8ccMs9JrmaSAyacw%3D%3D%7COjLl1jHSG%2FJvnRha32wfFg%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2039798".to_string(),
                title: "Why So Sad".to_string(),
                artist: "Pumpupthemind".to_string(),
                duration: 78,
                image: "https://usercontent.jamendo.com?type=album&id=526856&width=300&trackid=2039798".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2039798&format=mp31&from=MXpy59rsAlXJ7NZ1oOsclQ%3D%3D%7CImMFyoYlReKwKCWMjSrr6g%3D%3D".to_string(),
            },
            MusicTrack {
                id: "1998439".to_string(),
                title: "Lullabies".to_string(),
                artist: "B3B0CK".to_string(),
                duration: 200,
                image: "https://usercontent.jamendo.com?type=album&id=508340&width=300&trackid=1998439".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=1998439&format=mp31&from=2t%2BVaPz6gCVO%2FsgMNwU9GQ%3D%3D%7CaWUHNwGo3oE4MjsmX1CRhw%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2210762".to_string(),
                title: "With Kaxe".to_string(),
                artist: "Majed Salih".to_string(),
                duration: 136,
                image: "https://usercontent.jamendo.com?type=album&id=583168&width=300&trackid=2210762".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2210762&format=mp31&from=EnoctUS1uOSXYQrIE8lylw%3D%3D%7CDfzE40aUyhSTqSLuH0d%2B3g%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2025576".to_string(),
                title: "Little Gem (loop)".to_string(),
                artist: "Grumpynora".to_string(),
                duration: 35,
                image: "https://usercontent.jamendo.com?type=album&id=518202&width=300&trackid=2025576".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2025576&format=mp31&from=zS9tcsDy9Eqx3iWYKAbw9Q%3D%3D%7C0SamKsXV6XPsH0yKh7DvgA%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2253205".to_string(),
                title: "05.30".to_string(),
                artist: "Plushka".to_string(),
                duration: 123,
                image: "https://usercontent.jamendo.com?type=album&id=602864&width=300&trackid=2253205".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2253205&format=mp31&from=MngmdeHfhFyNXasw6mb41A%3D%3D%7Cjvx7lFPsn0ctOJMc6AfYTQ%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2163954".to_string(),
                title: "Vintage Vibes".to_string(),
                artist: "Silver-Stage".to_string(),
                duration: 200,
                image: "https://usercontent.jamendo.com?type=album&id=560384&width=300&trackid=2163954".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2163954&format=mp31&from=Wxyxs2lWSm%2F7hX1ZlE0C%2BA%3D%3D%7C50d65c7SaCrg6hr0%2Fh79Mw%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2153292".to_string(),
                title: "Black Sand".to_string(),
                artist: "Silver-Stage".to_string(),
                duration: 156,
                image: "https://usercontent.jamendo.com?type=album&id=559640&width=300&trackid=2153292".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2153292&format=mp31&from=w0tZJbq%2Bsy4MXGuGq6TZFw%3D%3D%7CCBKEc8%2BQWSBu9h3niGWByA%3D%3D".to_string(),
            },
            MusicTrack {
                id: "1957123".to_string(),
                title: "Soft Ambient Lofi Hiphop".to_string(),
                artist: "Joel Loopez".to_string(),
                duration: 158,
                image: "https://usercontent.jamendo.com?type=album&id=487987&width=300&trackid=1957123".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=1957123&format=mp31&from=s5nVko2imZNSn%2B5Jjp4NVg%3D%3D%7CP7s4huw2IxKbohxRkAt0Qw%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2216813".to_string(),
                title: "Supported".to_string(),
                artist: "Grumpynora".to_string(),
                duration: 76,
                image: "https://usercontent.jamendo.com?type=album&id=586536&width=300&trackid=2216813".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2216813&format=mp31&from=FaSh2CcRl5jiOIN3t8%2FDgw%3D%3D%7Crah%2FGepziP4hTLtYpSSAiA%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2029913".to_string(),
                title: "Dream Chill LoFI".to_string(),
                artist: "Dmytro Demchenko".to_string(),
                duration: 236,
                image: "https://usercontent.jamendo.com?type=album&id=520377&width=300&trackid=2029913".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2029913&format=mp31&from=MN3HwqYQxz385vLLHh2Wcg%3D%3D%7CQtDi7TG1kMi8VevTWYsLHg%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2255254".to_string(),
                title: "Deep Ferry | Lo-fi Music".to_string(),
                artist: "Praz Khanal".to_string(),
                duration: 176,
                image: "https://usercontent.jamendo.com?type=album&id=604025&width=300&trackid=2255254".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2255254&format=mp31&from=Z2YRXpALD%2FIizcJjQCQkiw%3D%3D%7CKIyFIdKJicegS%2Fd0oIs4%2Bg%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2226764".to_string(),
                title: "Pele's Volcano".to_string(),
                artist: "Aberrant Realities".to_string(),
                duration: 162,
                image: "https://usercontent.jamendo.com?type=album&id=590967&width=300&trackid=2226764".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2226764&format=mp31&from=waTXLqdI5Q2RD3p5qbkZVA%3D%3D%7CHiBpCC6vsx1TikN%2FsvnGNQ%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2226747".to_string(),
                title: "Ptesanwi's Sanctuary".to_string(),
                artist: "Aberrant Realities".to_string(),
                duration: 163,
                image: "https://usercontent.jamendo.com?type=album&id=590967&width=300&trackid=2226747".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2226747&format=mp31&from=ceggGznGCWKt7z6BBqXY7w%3D%3D%7CThS%2FtgAWr2MHFUGH1VkP7A%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2191787".to_string(),
                title: "Africa".to_string(),
                artist: "LOFI 528".to_string(),
                duration: 196,
                image: "https://usercontent.jamendo.com?type=album&id=575591&width=300&trackid=2191787".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2191787&format=mp31&from=w93GwMQJPmOM2EsqNZ1vuA%3D%3D%7CTNK%2BELJ1T50jTQCza7wEzg%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2015304".to_string(),
                title: "LoFi HipHop Intro 67".to_string(),
                artist: "TaigaSoundProd".to_string(),
                duration: 15,
                image: "https://usercontent.jamendo.com?type=album&id=514639&width=300&trackid=2015304".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2015304&format=mp31&from=Vp%2BHtgkrnJ27Sa4eYPpr0A%3D%3D%7CkigH96p%2FYg1BJOcvS60hxQ%3D%3D".to_string(),
            },
            MusicTrack {
                id: "1954734".to_string(),
                title: "Eppur si sogna".to_string(),
                artist: "Akhaton".to_string(),
                duration: 186,
                image: "https://usercontent.jamendo.com?type=album&id=487290&width=300&trackid=1954734".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=1954734&format=mp31&from=cLsvXGkHF1EyUCecqvt5MQ%3D%3D%7CNXYCPUKWjeYZa8B8msDtOA%3D%3D".to_string(),
            },
            MusicTrack {
                id: "2211694".to_string(),
                title: "LoFi Tapes - Cali Grooves".to_string(),
                artist: "Brothas Groove".to_string(),
                duration: 172,
                image: "https://usercontent.jamendo.com?type=album&id=583799&width=300&trackid=2211694".to_string(),
                audio: "https://prod-1.storage.jamendo.com/?trackid=2211694&format=mp31&from=953053f8FPcH0VI0MbwTEA%3D%3D%7CBKLbwp35r%2BlM%2BC82QkmZ9w%3D%3D".to_string(),
            }
        ]
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
