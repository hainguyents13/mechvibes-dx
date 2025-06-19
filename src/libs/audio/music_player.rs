use std::sync::{ Arc, Mutex };
use std::io::Cursor;
use rodio::{ Decoder, OutputStream, OutputStreamHandle, Sink };
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum MusicCommand {
    Play(String), // URL
    Pause,
    Resume,
    Stop,
    SetVolume(f32), // 0.0 to 1.0
    SetMuted(bool),
    NextTrack,
    GetStatus,
}

#[derive(Debug, Clone)]
pub struct MusicStatus {
    pub is_playing: bool,
    pub is_paused: bool,
    pub volume: f32,
    pub is_muted: bool,
    pub current_url: Option<String>,
}

pub struct RodioMusicPlayer {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sink: Arc<Mutex<Option<Sink>>>,
    status: Arc<Mutex<MusicStatus>>,
    command_sender: mpsc::UnboundedSender<MusicCommand>,
    status_receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<MusicStatus>>>>,
}

impl RodioMusicPlayer {
    pub fn new() -> Result<Self, String> {
        let (_stream, stream_handle) = OutputStream::try_default().map_err(|e|
            format!("Failed to create audio output stream: {}", e)
        )?;

        let sink = Arc::new(Mutex::new(None));
        let status = Arc::new(
            Mutex::new(MusicStatus {
                is_playing: false,
                is_paused: false,
                volume: 0.5,
                is_muted: false,
                current_url: None,
            })
        );

        let (command_sender, mut command_receiver) = mpsc::unbounded_channel::<MusicCommand>();
        let (status_sender, status_receiver) = mpsc::unbounded_channel::<MusicStatus>();

        // Clone references for the background task
        let sink_clone = Arc::clone(&sink);
        let status_clone = Arc::clone(&status);
        let stream_handle_clone = stream_handle.clone();

        // Spawn background task to handle music commands
        tokio::spawn(async move {
            while let Some(command) = command_receiver.recv().await {
                match command {
                    MusicCommand::Play(url) => {
                        if
                            let Err(e) = Self::handle_play_command(
                                &sink_clone,
                                &stream_handle_clone,
                                &status_clone,
                                &url
                            ).await
                        {
                            eprintln!("Failed to play music: {}", e);
                        }
                    }
                    MusicCommand::Pause => {
                        Self::handle_pause_command(&sink_clone, &status_clone);
                    }
                    MusicCommand::Resume => {
                        Self::handle_resume_command(&sink_clone, &status_clone);
                    }
                    MusicCommand::Stop => {
                        Self::handle_stop_command(&sink_clone, &status_clone);
                    }
                    MusicCommand::SetVolume(volume) => {
                        Self::handle_volume_command(&sink_clone, &status_clone, volume);
                    }
                    MusicCommand::SetMuted(muted) => {
                        Self::handle_mute_command(&sink_clone, &status_clone, muted);
                    }
                    MusicCommand::GetStatus => {
                        let current_status = {
                            let status_lock = status_clone.lock().unwrap();
                            status_lock.clone()
                        };
                        let _ = status_sender.send(current_status);
                    }
                    MusicCommand::NextTrack => {
                        // This will be handled by the UI layer
                        Self::handle_stop_command(&sink_clone, &status_clone);
                    }
                }
            }
        });

        Ok(Self {
            _stream,
            stream_handle,
            sink,
            status,
            command_sender,
            status_receiver: Arc::new(Mutex::new(Some(status_receiver))),
        })
    }

    async fn handle_play_command(
        sink: &Arc<Mutex<Option<Sink>>>,
        stream_handle: &OutputStreamHandle,
        status: &Arc<Mutex<MusicStatus>>,
        url: &str
    ) -> Result<(), String> {
        // Download the audio file
        let response = reqwest
            ::get(url).await
            .map_err(|e| format!("Failed to fetch audio: {}", e))?;

        let audio_data = response
            .bytes().await
            .map_err(|e| format!("Failed to read audio data: {}", e))?;

        // Create a cursor from the audio data
        let cursor = Cursor::new(audio_data.to_vec());

        // Create decoder
        let decoder = Decoder::new(cursor).map_err(|e| format!("Failed to decode audio: {}", e))?;

        // Create new sink
        let new_sink = Sink::try_new(stream_handle).map_err(|e|
            format!("Failed to create audio sink: {}", e)
        )?;

        // Update status and sink
        {
            let mut status_lock = status.lock().unwrap();
            status_lock.is_playing = true;
            status_lock.is_paused = false;
            status_lock.current_url = Some(url.to_string());

            let mut sink_lock = sink.lock().unwrap();
            *sink_lock = Some(new_sink);

            if let Some(ref sink) = *sink_lock {
                sink.set_volume(if status_lock.is_muted { 0.0 } else { status_lock.volume });
                sink.append(decoder);
                sink.play();
            }
        }

        Ok(())
    }

    fn handle_pause_command(sink: &Arc<Mutex<Option<Sink>>>, status: &Arc<Mutex<MusicStatus>>) {
        let mut status_lock = status.lock().unwrap();
        let sink_lock = sink.lock().unwrap();

        if let Some(ref sink) = *sink_lock {
            sink.pause();
            status_lock.is_playing = false;
            status_lock.is_paused = true;
        }
    }

    fn handle_resume_command(sink: &Arc<Mutex<Option<Sink>>>, status: &Arc<Mutex<MusicStatus>>) {
        let mut status_lock = status.lock().unwrap();
        let sink_lock = sink.lock().unwrap();

        if let Some(ref sink) = *sink_lock {
            sink.play();
            status_lock.is_playing = true;
            status_lock.is_paused = false;
        }
    }

    fn handle_stop_command(sink: &Arc<Mutex<Option<Sink>>>, status: &Arc<Mutex<MusicStatus>>) {
        let mut status_lock = status.lock().unwrap();
        let mut sink_lock = sink.lock().unwrap();

        if let Some(sink) = sink_lock.take() {
            sink.stop();
        }

        status_lock.is_playing = false;
        status_lock.is_paused = false;
        status_lock.current_url = None;
    }

    fn handle_volume_command(
        sink: &Arc<Mutex<Option<Sink>>>,
        status: &Arc<Mutex<MusicStatus>>,
        volume: f32
    ) {
        let mut status_lock = status.lock().unwrap();
        let sink_lock = sink.lock().unwrap();

        status_lock.volume = volume.clamp(0.0, 1.0);

        if let Some(ref sink) = *sink_lock {
            let actual_volume = if status_lock.is_muted { 0.0 } else { status_lock.volume };
            sink.set_volume(actual_volume);
        }
    }

    fn handle_mute_command(
        sink: &Arc<Mutex<Option<Sink>>>,
        status: &Arc<Mutex<MusicStatus>>,
        muted: bool
    ) {
        let mut status_lock = status.lock().unwrap();
        let sink_lock = sink.lock().unwrap();

        status_lock.is_muted = muted;

        if let Some(ref sink) = *sink_lock {
            let actual_volume = if muted { 0.0 } else { status_lock.volume };
            sink.set_volume(actual_volume);
        }
    }

    pub async fn play(&self, url: &str) -> Result<(), String> {
        self.command_sender
            .send(MusicCommand::Play(url.to_string()))
            .map_err(|e| format!("Failed to send play command: {}", e))
    }

    pub fn pause(&self) -> Result<(), String> {
        self.command_sender
            .send(MusicCommand::Pause)
            .map_err(|e| format!("Failed to send pause command: {}", e))
    }

    pub fn resume(&self) -> Result<(), String> {
        self.command_sender
            .send(MusicCommand::Resume)
            .map_err(|e| format!("Failed to send resume command: {}", e))
    }

    pub fn stop(&self) -> Result<(), String> {
        self.command_sender
            .send(MusicCommand::Stop)
            .map_err(|e| format!("Failed to send stop command: {}", e))
    }

    pub fn set_volume(&self, volume: f32) -> Result<(), String> {
        self.command_sender
            .send(MusicCommand::SetVolume(volume))
            .map_err(|e| format!("Failed to send volume command: {}", e))
    }

    pub fn set_muted(&self, muted: bool) -> Result<(), String> {
        self.command_sender
            .send(MusicCommand::SetMuted(muted))
            .map_err(|e| format!("Failed to send mute command: {}", e))
    }

    pub fn get_status(&self) -> MusicStatus {
        let status_lock = self.status.lock().unwrap();
        status_lock.clone()
    }

    pub fn is_finished(&self) -> bool {
        let sink_lock = self.sink.lock().unwrap();
        if let Some(ref sink) = *sink_lock {
            sink.empty()
        } else {
            true
        }
    }
}
