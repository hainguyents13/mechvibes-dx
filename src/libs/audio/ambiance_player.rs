use std::sync::{ Arc, Mutex };
use std::collections::HashMap;
use rodio::{ Decoder, OutputStream, OutputStreamHandle, Sink, Source };
use std::io::Cursor;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum AmbianceCommand {
    PlaySound {
        sound_id: String,
        audio_url: String,
        volume: f32,
        should_loop: bool,
    },
    StopSound(String),
    StopAll,
    PauseAll,
    ResumeAll,
    SetSoundVolume {
        sound_id: String,
        volume: f32,
    },
    SetGlobalVolume(f32),
    SetGlobalMute(bool),
}

#[derive(Debug, Clone)]
pub struct AmbianceStatus {
    pub playing_sounds: Vec<String>,
    pub global_volume: f32,
    pub is_muted: bool,
    pub is_playing: bool,
}

impl Default for AmbianceStatus {
    fn default() -> Self {
        Self {
            playing_sounds: Vec::new(),
            global_volume: 0.5,
            is_muted: false,
            is_playing: false,
        }
    }
}

pub struct RodioAmbiancePlayer {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sinks: Arc<Mutex<HashMap<String, Sink>>>, // sound_id -> Sink
    command_sender: mpsc::UnboundedSender<AmbianceCommand>,
    status: Arc<Mutex<AmbianceStatus>>,
}

impl RodioAmbiancePlayer {
    pub fn new() -> Result<Self, String> {
        let (_stream, stream_handle) = rodio::OutputStream
            ::try_default()
            .map_err(|e| format!("Failed to create audio output stream: {}", e))?;

        let sinks = Arc::new(Mutex::new(HashMap::new()));
        let status = Arc::new(Mutex::new(AmbianceStatus::default()));

        let (command_sender, mut command_receiver) = mpsc::unbounded_channel();

        // Background task to handle commands
        {
            let sinks_clone = sinks.clone();
            let stream_handle_clone = stream_handle.clone();
            let status_clone = status.clone();

            tokio::spawn(async move {
                while let Some(command) = command_receiver.recv().await {
                    match command {
                        AmbianceCommand::PlaySound { sound_id, audio_url, volume, should_loop } => {
                            if
                                let Err(e) = Self::handle_play_sound_command(
                                    &sinks_clone,
                                    &stream_handle_clone,
                                    &status_clone,
                                    &sound_id,
                                    &audio_url,
                                    volume,
                                    should_loop
                                ).await
                            {
                                eprintln!("❌ Failed to play ambiance sound {}: {}", sound_id, e);
                            }
                        }
                        AmbianceCommand::StopSound(sound_id) => {
                            Self::handle_stop_sound_command(&sinks_clone, &status_clone, &sound_id);
                        }
                        AmbianceCommand::StopAll => {
                            Self::handle_stop_all_command(&sinks_clone, &status_clone);
                        }
                        AmbianceCommand::PauseAll => {
                            Self::handle_pause_all_command(&sinks_clone, &status_clone);
                        }
                        AmbianceCommand::ResumeAll => {
                            Self::handle_resume_all_command(&sinks_clone, &status_clone);
                        }
                        AmbianceCommand::SetSoundVolume { sound_id, volume } => {
                            Self::handle_set_sound_volume_command(&sinks_clone, &sound_id, volume);
                        }
                        AmbianceCommand::SetGlobalVolume(volume) => {
                            Self::handle_set_global_volume_command(
                                &sinks_clone,
                                &status_clone,
                                volume
                            );
                        }
                        AmbianceCommand::SetGlobalMute(muted) => {
                            Self::handle_set_global_mute_command(
                                &sinks_clone,
                                &status_clone,
                                muted
                            );
                        }
                    }
                }
            });
        }

        Ok(Self {
            _stream,
            stream_handle,
            sinks,
            command_sender,
            status,
        })
    }

    async fn handle_play_sound_command(
        sinks: &Arc<Mutex<HashMap<String, Sink>>>,
        stream_handle: &OutputStreamHandle,
        status: &Arc<Mutex<AmbianceStatus>>,
        sound_id: &str,
        audio_url: &str,
        volume: f32,
        should_loop: bool
    ) -> Result<(), String> {
        // Stop existing sound if playing
        Self::handle_stop_sound_command(sinks, status, sound_id);

        // Load audio file from local path
        let audio_path = audio_url.replace("assets/", "");
        let full_path = format!("assets/{}", audio_path);

        // Read the audio file
        let audio_data = std::fs
            ::read(&full_path)
            .map_err(|e| format!("Failed to read audio file {}: {}", full_path, e))?;

        // Create a cursor from the audio data
        let cursor = Cursor::new(audio_data);

        // Create decoder
        let decoder = Decoder::new(cursor).map_err(|e| format!("Failed to decode audio: {}", e))?;

        // Create new sink
        let sink = Sink::try_new(stream_handle).map_err(|e|
            format!("Failed to create audio sink: {}", e)
        )?;

        // Set volume
        let status_lock = status.lock().unwrap();
        let effective_volume = if status_lock.is_muted || !status_lock.is_playing {
            0.0
        } else {
            volume * status_lock.global_volume
        };
        drop(status_lock);

        sink.set_volume(effective_volume); // Add to sink
        if should_loop {
            sink.append(decoder.repeat_infinite());
        } else {
            sink.append(decoder);
        }

        // Store sink
        {
            let mut sinks_lock = sinks.lock().unwrap();
            sinks_lock.insert(sound_id.to_string(), sink);
        }

        // Update status
        {
            let mut status_lock = status.lock().unwrap();
            if !status_lock.playing_sounds.contains(&sound_id.to_string()) {
                status_lock.playing_sounds.push(sound_id.to_string());
            }
        }

        println!("🎵 Started playing ambiance sound: {}", sound_id);
        Ok(())
    }

    fn handle_stop_sound_command(
        sinks: &Arc<Mutex<HashMap<String, Sink>>>,
        status: &Arc<Mutex<AmbianceStatus>>,
        sound_id: &str
    ) {
        let mut sinks_lock = sinks.lock().unwrap();
        if let Some(sink) = sinks_lock.remove(sound_id) {
            sink.stop();
            println!("🔇 Stopped ambiance sound: {}", sound_id);
        }
        drop(sinks_lock);

        // Update status
        let mut status_lock = status.lock().unwrap();
        status_lock.playing_sounds.retain(|id| id != sound_id);
    }

    fn handle_stop_all_command(
        sinks: &Arc<Mutex<HashMap<String, Sink>>>,
        status: &Arc<Mutex<AmbianceStatus>>
    ) {
        let mut sinks_lock = sinks.lock().unwrap();
        for sink in sinks_lock.values() {
            sink.stop();
        }
        sinks_lock.clear();
        drop(sinks_lock);

        // Update status
        let mut status_lock = status.lock().unwrap();
        status_lock.playing_sounds.clear();
        println!("🔇 Stopped all ambiance sounds");
    }

    fn handle_pause_all_command(
        sinks: &Arc<Mutex<HashMap<String, Sink>>>,
        status: &Arc<Mutex<AmbianceStatus>>
    ) {
        let sinks_lock = sinks.lock().unwrap();
        for sink in sinks_lock.values() {
            sink.pause();
        }
        drop(sinks_lock);

        // Update status
        let mut status_lock = status.lock().unwrap();
        status_lock.is_playing = false;
        println!("⏸️ Paused all ambiance sounds");
    }

    fn handle_resume_all_command(
        sinks: &Arc<Mutex<HashMap<String, Sink>>>,
        status: &Arc<Mutex<AmbianceStatus>>
    ) {
        let sinks_lock = sinks.lock().unwrap();
        for sink in sinks_lock.values() {
            sink.play();
        }
        drop(sinks_lock);

        // Update status
        let mut status_lock = status.lock().unwrap();
        status_lock.is_playing = true;
        println!("▶️ Resumed all ambiance sounds");
    }

    fn handle_set_sound_volume_command(
        sinks: &Arc<Mutex<HashMap<String, Sink>>>,
        sound_id: &str,
        volume: f32
    ) {
        let sinks_lock = sinks.lock().unwrap();
        if let Some(sink) = sinks_lock.get(sound_id) {
            sink.set_volume(volume.clamp(0.0, 1.0));
        }
    }

    fn handle_set_global_volume_command(
        sinks: &Arc<Mutex<HashMap<String, Sink>>>,
        status: &Arc<Mutex<AmbianceStatus>>,
        volume: f32
    ) {
        let mut status_lock = status.lock().unwrap();
        status_lock.global_volume = volume.clamp(0.0, 1.0);
        drop(status_lock);

        // Update all sink volumes (this is a simplified approach)
        // In a real implementation, you'd want to track individual volumes
        let sinks_lock = sinks.lock().unwrap();
        for sink in sinks_lock.values() {
            sink.set_volume(volume.clamp(0.0, 1.0));
        }
    }

    fn handle_set_global_mute_command(
        sinks: &Arc<Mutex<HashMap<String, Sink>>>,
        status: &Arc<Mutex<AmbianceStatus>>,
        muted: bool
    ) {
        let mut status_lock = status.lock().unwrap();
        status_lock.is_muted = muted;
        drop(status_lock);

        let sinks_lock = sinks.lock().unwrap();
        for sink in sinks_lock.values() {
            if muted {
                sink.pause();
            } else {
                sink.play();
            }
        }
    }

    // Public methods
    pub fn play_sound(
        &self,
        sound_id: String,
        audio_url: String,
        volume: f32
    ) -> Result<(), String> {
        self.command_sender
            .send(AmbianceCommand::PlaySound {
                sound_id,
                audio_url,
                volume,
                should_loop: true, // Ambiance sounds should loop
            })
            .map_err(|e| format!("Failed to send play command: {}", e))
    }

    pub fn stop_sound(&self, sound_id: &str) -> Result<(), String> {
        self.command_sender
            .send(AmbianceCommand::StopSound(sound_id.to_string()))
            .map_err(|e| format!("Failed to send stop command: {}", e))
    }

    pub fn stop_all(&self) -> Result<(), String> {
        self.command_sender
            .send(AmbianceCommand::StopAll)
            .map_err(|e| format!("Failed to send stop all command: {}", e))
    }

    pub fn pause_all(&self) -> Result<(), String> {
        self.command_sender
            .send(AmbianceCommand::PauseAll)
            .map_err(|e| format!("Failed to send pause all command: {}", e))
    }

    pub fn resume_all(&self) -> Result<(), String> {
        self.command_sender
            .send(AmbianceCommand::ResumeAll)
            .map_err(|e| format!("Failed to send resume all command: {}", e))
    }

    pub fn set_sound_volume(&self, sound_id: &str, volume: f32) -> Result<(), String> {
        self.command_sender
            .send(AmbianceCommand::SetSoundVolume {
                sound_id: sound_id.to_string(),
                volume,
            })
            .map_err(|e| format!("Failed to send set sound volume command: {}", e))
    }

    pub fn set_global_volume(&self, volume: f32) -> Result<(), String> {
        self.command_sender
            .send(AmbianceCommand::SetGlobalVolume(volume))
            .map_err(|e| format!("Failed to send set global volume command: {}", e))
    }

    pub fn set_global_mute(&self, muted: bool) -> Result<(), String> {
        self.command_sender
            .send(AmbianceCommand::SetGlobalMute(muted))
            .map_err(|e| format!("Failed to send set global mute command: {}", e))
    }

    pub fn get_status(&self) -> AmbianceStatus {
        let status_lock = self.status.lock().unwrap();
        status_lock.clone()
    }
}
