# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.5.2] - Unreleased

### Fixed

- **Sound cutting off / clicking when typing fast**: keyboard and mouse sounds now use a proper voice pool (oldest-first eviction) instead of a hashmap keyed by key name, so rapid repeated keys no longer cut each other's tails off. Added short fade-in/fade-out (2ms/5ms) to eliminate clicks/pops at segment boundaries, and soft (ramped) eviction instead of a hard cut when the voice pool is full.
- **Sound continuing to play after releasing all keys ("ghost typing")**: the keyboard/mouse/hotkey event loops now drain their entire backlog every tick instead of processing one event at a time, so a fast burst of keystrokes can no longer queue up and keep playing sound after the user has already lifted their hands.
- **Poor audio quality from realtime resampling**: soundpacks are now resampled once at load time to the output device's sample rate (using a high-quality sinc resampler) instead of relying on the audio backend's realtime linear resampling.
- Removed a redundant device probe on every soundpack load; the output device's sample rate is now probed once at startup and cached, avoiding unnecessary device enumeration (which could briefly interrupt audio on Linux/ALSA).
- Sample-rate lookup failures no longer fall back to a hardcoded 44100 Hz guess (which could cause audio to be resampled twice); they now skip resampling and keep the file's native rate instead.

### Changed

- Increased the keyboard/mouse voice pool limit (`max_voices`) from 20 to 32 to give more headroom for overlapping sound tails during fast typing.

## [0.5.1] and earlier

See git history for changes prior to this changelog's introduction.
