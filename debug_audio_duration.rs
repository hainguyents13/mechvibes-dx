use rodio::{ Decoder, Source };
use std::fs::File;
use std::io::BufReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let soundpack_dir = "d:\\mechvibes-dx\\soundpacks\\custom-sound-pack-1660581102261";

    // Check combined_audio.wav
    let combined_path = format!("{}\\combined_audio.wav", soundpack_dir);
    if let Ok(file) = File::open(&combined_path) {
        if let Ok(source) = Decoder::new(BufReader::new(file)) {
            let sample_rate = source.sample_rate();
            let channels = source.channels();
            let samples: Vec<f32> = source.convert_samples().collect();
            let duration_seconds =
                (samples.len() as f32) / (sample_rate as f32) / (channels as f32);

            println!("combined_audio.wav:");
            println!("  Duration: {:.3}s ({:.1}ms)", duration_seconds, duration_seconds * 1000.0);
            println!("  Samples: {}", samples.len());
            println!("  Sample rate: {}Hz", sample_rate);
            println!("  Channels: {}", channels);
        }
    }

    // Check individual spacebar files
    let spacebar_files = ["spacebar.mp3", "spacebar2.mp3", "spacebar3.mp3"];
    for file in spacebar_files {
        let file_path = format!("{}\\{}", soundpack_dir, file);
        if let Ok(audio_file) = File::open(&file_path) {
            if let Ok(source) = Decoder::new(BufReader::new(audio_file)) {
                let sample_rate = source.sample_rate();
                let channels = source.channels();
                let samples: Vec<f32> = source.convert_samples().collect();
                let duration_seconds =
                    (samples.len() as f32) / (sample_rate as f32) / (channels as f32);

                println!("{}:", file);
                println!(
                    "  Duration: {:.3}s ({:.1}ms)",
                    duration_seconds,
                    duration_seconds * 1000.0
                );
            }
        }
    }

    Ok(())
}
