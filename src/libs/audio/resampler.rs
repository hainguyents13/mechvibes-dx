use rubato::{ Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction };

/// Resamples interleaved PCM samples from `from_rate` to `to_rate` using a
/// sinc resampler (rubato). Returns the input unchanged if the rates match.
pub fn resample_interleaved(
    samples: &[f32],
    channels: u16,
    from_rate: u32,
    to_rate: u32
) -> Vec<f32> {
    if from_rate == to_rate || samples.is_empty() {
        return samples.to_vec();
    }

    let channels = channels.max(1) as usize;
    let frame_count = samples.len() / channels;

    // Deinterleave into per-channel buffers.
    let mut deinterleaved: Vec<Vec<f32>> = vec![Vec::with_capacity(frame_count); channels];
    for frame in 0..frame_count {
        for c in 0..channels {
            deinterleaved[c].push(samples[frame * channels + c]);
        }
    }

    let params = SincInterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.95,
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 256,
        window: WindowFunction::BlackmanHarris2,
    };

    let chunk_size = 1024;
    let mut resampler = match
        SincFixedIn::<f32>::new(
            (to_rate as f64) / (from_rate as f64),
            2.0,
            params,
            chunk_size,
            channels
        )
    {
        Ok(r) => r,
        Err(e) => {
            crate::always_eprint!("❌ Failed to create resampler: {}", e);
            return samples.to_vec();
        }
    };

    // rubato's chunked `process()` already returns output that is correctly
    // time-aligned per call (verified empirically: an impulse at input frame
    // N lands at output frame `N * to_rate/from_rate` with no extra shift).
    // `Resampler::output_delay()` describes filter latency for continuous
    // streaming use and is NOT an offset to skip from this buffered output.
    //
    // What *is* needed: the final chunk is zero-padded up to `chunk_size`
    // frames (rubato requires fixed-size input), so its output must be
    // truncated to the real (non-padded) length - otherwise the padded tail
    // leaks a sinc-smeared artifact into the resampled audio.
    let mut output: Vec<Vec<f32>> = vec![Vec::new(); channels];
    let mut pos = 0;

    while pos < frame_count {
        let remaining = frame_count - pos;
        let this_chunk = remaining.min(chunk_size);

        let input_frames: Vec<Vec<f32>> = deinterleaved
            .iter()
            .map(|ch| {
                let mut buf = ch[pos..pos + this_chunk].to_vec();
                if this_chunk < chunk_size {
                    // Pad the final partial chunk with zeros; rubato requires
                    // fixed-size input frames.
                    buf.resize(chunk_size, 0.0);
                }
                buf
            })
            .collect();

        match resampler.process(&input_frames, None) {
            Ok(resampled_chunks) => {
                for (c, chunk) in resampled_chunks.into_iter().enumerate() {
                    output[c].extend(chunk);
                }
            }
            Err(e) => {
                crate::always_eprint!("❌ Resample chunk failed: {}", e);
                break;
            }
        }

        pos += this_chunk;
    }

    // Truncate to the length implied by the real (non-padded) input frame
    // count.
    let expected_frame_count = (
        ((frame_count as f64) * (to_rate as f64)) / (from_rate as f64)
    ).round() as usize;
    let out_frame_count = output
        .first()
        .map(|c| c.len())
        .unwrap_or(0)
        .min(expected_frame_count);

    // Re-interleave.
    let mut interleaved = Vec::with_capacity(out_frame_count * channels);
    for frame in 0..out_frame_count {
        for ch in output.iter() {
            interleaved.push(ch[frame]);
        }
    }

    interleaved
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resample_upsamples_mono_sine_to_expected_length() {
        let from_rate = 44_100;
        let to_rate = 48_000;
        let duration_secs = 1.0;
        let frame_count = (from_rate as f32 * duration_secs) as usize;

        let samples: Vec<f32> = (0..frame_count)
            .map(|i| {
                let t = (i as f32) / (from_rate as f32);
                (2.0 * std::f32::consts::PI * 440.0 * t).sin()
            })
            .collect();

        let resampled = resample_interleaved(&samples, 1, from_rate, to_rate);

        // Length must match the real (non-padded) input frame count scaled
        // by the rate ratio, exactly (within rounding) - not just "close to
        // it within a chunk's worth of slack", which would let a padded,
        // untruncated tail slip through undetected.
        let expected_len = ((frame_count as f64) * (to_rate as f64) / (from_rate as f64)).round() as usize;
        let diff = (resampled.len() as i64 - expected_len as i64).unsigned_abs() as usize;
        assert!(
            diff <= 2,
            "resampled length {} too far from expected {} (tail not truncated?)",
            resampled.len(),
            expected_len
        );
    }

    #[test]
    fn resample_does_not_leak_padded_tail_artifact() {
        // Use an input length that is NOT a multiple of the internal chunk
        // size (1024) so the final chunk is zero-padded - this is exactly
        // the case where an untruncated resampler leaks sinc-smeared
        // padding into the output tail.
        let from_rate = 44_100;
        let to_rate = 48_000;
        let frame_count = 1024 * 3 + 200; // partial final chunk

        let samples: Vec<f32> = (0..frame_count)
            .map(|i| {
                let t = (i as f32) / (from_rate as f32);
                (2.0 * std::f32::consts::PI * 440.0 * t).sin()
            })
            .collect();

        let resampled = resample_interleaved(&samples, 1, from_rate, to_rate);

        let expected_len = ((frame_count as f64) * (to_rate as f64) / (from_rate as f64)).round() as usize;
        let diff = (resampled.len() as i64 - expected_len as i64).unsigned_abs() as usize;
        assert!(
            diff <= 2,
            "resampled length {} too far from expected {} for non-chunk-aligned input (tail not truncated?)",
            resampled.len(),
            expected_len
        );
    }

    #[test]
    fn resample_preserves_impulse_timing() {
        // An impulse at a known frame should land at the equivalent
        // rate-scaled frame in the output, with no extra shift from the
        // sinc filter's internal latency leaking into buffered (non-streaming)
        // output.
        let from_rate = 44_100;
        let to_rate = 48_000;
        let impulse_frame = 1000;
        let frame_count = 4096;

        let mut samples = vec![0.0f32; frame_count];
        samples[impulse_frame] = 1.0;

        let resampled = resample_interleaved(&samples, 1, from_rate, to_rate);

        let expected_peak_frame =
            ((impulse_frame as f64) * (to_rate as f64) / (from_rate as f64)).round() as usize;

        let peak_frame = resampled
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.abs().partial_cmp(&b.abs()).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);

        let diff = (peak_frame as i64 - expected_peak_frame as i64).unsigned_abs() as usize;
        assert!(
            diff <= 16,
            "peak at frame {} too far from expected frame {} (filter delay not compensated?)",
            peak_frame,
            expected_peak_frame
        );
    }

    #[test]
    fn resample_skips_when_rates_match() {
        let samples = vec![0.1, 0.2, 0.3, 0.4];
        let resampled = resample_interleaved(&samples, 2, 44_100, 44_100);
        assert_eq!(resampled, samples);
    }
}
