use cpal::traits::{ DeviceTrait, HostTrait };
use cpal::{ Device, Host };
use std::sync::{ Mutex, OnceLock };

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

// Global device cache to avoid repeated enumeration
static CACHED_OUTPUT_DEVICES: OnceLock<Mutex<Vec<DeviceInfo>>> = OnceLock::new();
static CACHED_INPUT_DEVICES: OnceLock<Mutex<Vec<DeviceInfo>>> = OnceLock::new();

// ALSA error suppressor for Linux to silence expected enumeration errors
#[cfg(target_os = "linux")]
struct AlsaErrorSuppressor {
    _stderr_fd: std::os::fd::OwnedFd,
}

#[cfg(target_os = "linux")]
impl AlsaErrorSuppressor {
    fn new() -> Self {
        use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

        // Redirect stderr to /dev/null temporarily to suppress ALSA error messages
        // ALSA generates expected errors when probing invalid/misconfigured devices
        unsafe {
            let null_fd = libc::open(
                b"/dev/null\0".as_ptr() as *const libc::c_char,
                libc::O_WRONLY
            );
            let stderr_fd = libc::dup(libc::STDERR_FILENO);
            libc::dup2(null_fd, libc::STDERR_FILENO);
            libc::close(null_fd);

            Self {
                _stderr_fd: OwnedFd::from_raw_fd(stderr_fd),
            }
        }
    }
}

#[cfg(target_os = "linux")]
impl Drop for AlsaErrorSuppressor {
    fn drop(&mut self) {
        use std::os::fd::AsRawFd;

        // Restore original stderr when suppressor is dropped
        unsafe {
            libc::dup2(self._stderr_fd.as_raw_fd(), libc::STDERR_FILENO);
        }
    }
}

pub struct DeviceManager {
    host: Host,
}

impl Clone for DeviceManager {
    fn clone(&self) -> Self {
        Self {
            host: cpal::default_host(),
        }
    }
}

#[allow(dead_code)]
impl DeviceManager {
    pub fn new() -> Self {
        Self {
            host: cpal::default_host(),
        }
    }

    /// Get all available audio output devices
    pub fn get_output_devices(&self) -> Result<Vec<DeviceInfo>, String> {
        println!("🔍 [DeviceManager] Starting audio output device enumeration...");
        let mut devices = Vec::new();
        let default_device = self.host.default_output_device();
        let default_name = default_device
            .as_ref()
            .and_then(|d| d.name().ok())
            .unwrap_or_else(|| "Unknown".to_string());

        println!("🔍 [DeviceManager] Default device: {}", default_name);

        // Suppress ALSA error messages on Linux during device enumeration
        // ALSA probes all possible devices and generates expected errors for invalid/misconfigured ones
        #[cfg(target_os = "linux")]
        let _alsa_suppressor = AlsaErrorSuppressor::new();

        println!("🔍 [DeviceManager] Enumerating output devices via ALSA/cpal...");
        match self.host.output_devices() {
            Ok(device_iter) => {
                for (index, device) in device_iter.enumerate() {
                    if let Ok(name) = device.name() {
                        // Filter out low-level ALSA device aliases
                        // Only show user-friendly device names (default, pipewire, pulse, etc.)
                        #[cfg(target_os = "linux")]
                        {
                            // Skip low-level ALSA aliases (hw:, plughw:, dmix:, dsnoop:, etc.)
                            if name.starts_with("hw:")
                                || name.starts_with("plughw:")
                                || name.starts_with("dmix:")
                                || name.starts_with("dsnoop:")
                                || name.starts_with("front:")
                                || name.starts_with("surround")
                                || name.starts_with("iec958:")
                            {
                                println!("🔍 [DeviceManager] Skipping low-level ALSA alias: {}", name);
                                continue;
                            }
                        }

                        let is_default =
                            Some(&name) ==
                            default_device
                                .as_ref()
                                .and_then(|d| d.name().ok())
                                .as_ref();

                        println!("🔍 [DeviceManager] Found device #{}: {} {}",
                            index,
                            name,
                            if is_default { "(default)" } else { "" }
                        );

                        devices.push(DeviceInfo {
                            id: format!("output_{}", index),
                            name: name.clone(),
                            is_default,
                        });
                    }
                }
                println!("✅ [DeviceManager] Enumeration complete. Found {} devices", devices.len());
            }
            Err(e) => {
                println!("❌ [DeviceManager] Failed to enumerate: {}", e);
                return Err(format!("Failed to enumerate output devices: {}", e));
            }
        }

        // Ensure we have at least the default device
        if devices.is_empty() {
            println!("⚠️ [DeviceManager] No devices found, adding default fallback");
            devices.push(DeviceInfo {
                id: "output_default".to_string(),
                name: default_name,
                is_default: true,
            });
        }

        println!("🔍 [DeviceManager] Returning {} total devices", devices.len());
        Ok(devices)
    }

    /// Get all available audio input devices
    pub fn get_input_devices(&self) -> Result<Vec<DeviceInfo>, String> {
        let mut devices = Vec::new();
        let default_device = self.host.default_input_device();
        let default_name = default_device
            .as_ref()
            .and_then(|d| d.name().ok())
            .unwrap_or_else(|| "Unknown".to_string());

        // Suppress ALSA error messages on Linux during device enumeration
        #[cfg(target_os = "linux")]
        let _alsa_suppressor = AlsaErrorSuppressor::new();

        match self.host.input_devices() {
            Ok(device_iter) => {
                for (index, device) in device_iter.enumerate() {
                    if let Ok(name) = device.name() {
                        // Filter out low-level ALSA device aliases
                        #[cfg(target_os = "linux")]
                        {
                            if name.starts_with("hw:")
                                || name.starts_with("plughw:")
                                || name.starts_with("dmix:")
                                || name.starts_with("dsnoop:")
                                || name.starts_with("front:")
                                || name.starts_with("surround")
                                || name.starts_with("iec958:")
                            {
                                continue;
                            }
                        }

                        let is_default =
                            Some(&name) ==
                            default_device
                                .as_ref()
                                .and_then(|d| d.name().ok())
                                .as_ref();

                        devices.push(DeviceInfo {
                            id: format!("input_{}", index),
                            name: name.clone(),
                            is_default,
                        });
                    }
                }
            }
            Err(e) => {
                return Err(format!("Failed to enumerate input devices: {}", e));
            }
        }

        // Ensure we have at least the default device
        if devices.is_empty() {
            devices.push(DeviceInfo {
                id: "input_default".to_string(),
                name: default_name,
                is_default: true,
            });
        }

        Ok(devices)
    }

    /// Get device by ID for output devices
    pub fn get_output_device_by_id(&self, device_id: &str) -> Result<Option<Device>, String> {
        if device_id == "output_default" {
            return Ok(self.host.default_output_device());
        }

        // Suppress ALSA error messages on Linux
        #[cfg(target_os = "linux")]
        let _alsa_suppressor = AlsaErrorSuppressor::new();

        // Parse index from device_id (format: "output_{index}")
        if let Some(index_str) = device_id.strip_prefix("output_") {
            if let Ok(target_index) = index_str.parse::<usize>() {
                match self.host.output_devices() {
                    Ok(device_iter) => {
                        for (index, device) in device_iter.enumerate() {
                            if index == target_index {
                                return Ok(Some(device));
                            }
                        }
                    }
                    Err(e) => {
                        return Err(format!("Failed to enumerate devices: {}", e));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Get device by ID for input devices
    pub fn get_input_device_by_id(&self, device_id: &str) -> Result<Option<Device>, String> {
        if device_id == "input_default" {
            return Ok(self.host.default_input_device());
        }

        // Suppress ALSA error messages on Linux
        #[cfg(target_os = "linux")]
        let _alsa_suppressor = AlsaErrorSuppressor::new();

        // Parse index from device_id (format: "input_{index}")
        if let Some(index_str) = device_id.strip_prefix("input_") {
            if let Ok(target_index) = index_str.parse::<usize>() {
                match self.host.input_devices() {
                    Ok(device_iter) => {
                        for (index, device) in device_iter.enumerate() {
                            if index == target_index {
                                return Ok(Some(device));
                            }
                        }
                    }
                    Err(e) => {
                        return Err(format!("Failed to enumerate devices: {}", e));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Test if a device is available and working
    pub fn test_output_device(&self, device_id: &str) -> Result<bool, String> {
        // Suppress ALSA error messages on Linux
        #[cfg(target_os = "linux")]
        let _alsa_suppressor = AlsaErrorSuppressor::new();

        match self.get_output_device_by_id(device_id)? {
            Some(device) => {
                // Try to get supported configurations to test device availability
                match device.supported_output_configs() {
                    Ok(mut configs) => Ok(configs.next().is_some()),
                    Err(_) => Ok(false),
                }
            }
            None => Ok(false),
        }
    }

    /// Test if an input device is available and working
    pub fn test_input_device(&self, device_id: &str) -> Result<bool, String> {
        // Suppress ALSA error messages on Linux
        #[cfg(target_os = "linux")]
        let _alsa_suppressor = AlsaErrorSuppressor::new();

        match self.get_input_device_by_id(device_id)? {
            Some(device) => {
                // Try to get supported configurations to test device availability
                match device.supported_input_configs() {
                    Ok(mut configs) => Ok(configs.next().is_some()),
                    Err(_) => Ok(false),
                }
            }
            None => Ok(false),
        }
    }
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

// Global device cache management functions
impl DeviceManager {
    /// Initialize device cache on app startup (enumerate once and cache)
    pub fn initialize_cache() {
        println!("🔍 [DeviceCache] Initializing device cache on app startup...");
        let manager = DeviceManager::new();

        // Load output devices
        match manager.get_output_devices() {
            Ok(devices) => {
                println!("✅ [DeviceCache] Cached {} output devices", devices.len());
                CACHED_OUTPUT_DEVICES.get_or_init(|| Mutex::new(devices));
            }
            Err(e) => {
                println!("❌ [DeviceCache] Failed to cache output devices: {}", e);
                CACHED_OUTPUT_DEVICES.get_or_init(|| Mutex::new(Vec::new()));
            }
        }

        // Load input devices
        match manager.get_input_devices() {
            Ok(devices) => {
                println!("✅ [DeviceCache] Cached {} input devices", devices.len());
                CACHED_INPUT_DEVICES.get_or_init(|| Mutex::new(devices));
            }
            Err(e) => {
                println!("❌ [DeviceCache] Failed to cache input devices: {}", e);
                CACHED_INPUT_DEVICES.get_or_init(|| Mutex::new(Vec::new()));
            }
        }
    }

    /// Get cached output devices (no enumeration - instant)
    pub fn get_cached_output_devices() -> Result<Vec<DeviceInfo>, String> {
        println!("📋 [DeviceCache] Returning cached output devices...");
        if let Some(cache) = CACHED_OUTPUT_DEVICES.get() {
            if let Ok(devices) = cache.lock() {
                println!("✅ [DeviceCache] Found {} cached output devices", devices.len());
                return Ok(devices.clone());
            }
        }

        println!("⚠️ [DeviceCache] Cache not initialized, initializing now...");
        Self::initialize_cache();

        if let Some(cache) = CACHED_OUTPUT_DEVICES.get() {
            if let Ok(devices) = cache.lock() {
                return Ok(devices.clone());
            }
        }

        Err("Failed to access device cache".to_string())
    }

    /// Get cached input devices (no enumeration - instant)
    pub fn get_cached_input_devices() -> Result<Vec<DeviceInfo>, String> {
        println!("📋 [DeviceCache] Returning cached input devices...");
        if let Some(cache) = CACHED_INPUT_DEVICES.get() {
            if let Ok(devices) = cache.lock() {
                println!("✅ [DeviceCache] Found {} cached input devices", devices.len());
                return Ok(devices.clone());
            }
        }

        println!("⚠️ [DeviceCache] Cache not initialized, initializing now...");
        Self::initialize_cache();

        if let Some(cache) = CACHED_INPUT_DEVICES.get() {
            if let Ok(devices) = cache.lock() {
                return Ok(devices.clone());
            }
        }

        Err("Failed to access device cache".to_string())
    }

    /// Force refresh cache (re-enumerate and update)
    pub fn refresh_cache() -> Result<(), String> {
        println!("🔄 [DeviceCache] Force refreshing device cache...");
        let manager = DeviceManager::new();

        // Refresh output devices
        match manager.get_output_devices() {
            Ok(devices) => {
                if let Some(cache) = CACHED_OUTPUT_DEVICES.get() {
                    if let Ok(mut cached) = cache.lock() {
                        *cached = devices;
                        println!("✅ [DeviceCache] Refreshed output devices cache");
                    }
                }
            }
            Err(e) => {
                println!("❌ [DeviceCache] Failed to refresh output devices: {}", e);
                return Err(e);
            }
        }

        // Refresh input devices
        match manager.get_input_devices() {
            Ok(devices) => {
                if let Some(cache) = CACHED_INPUT_DEVICES.get() {
                    if let Ok(mut cached) = cache.lock() {
                        *cached = devices;
                        println!("✅ [DeviceCache] Refreshed input devices cache");
                    }
                }
            }
            Err(e) => {
                println!("❌ [DeviceCache] Failed to refresh input devices: {}", e);
                return Err(e);
            }
        }

        Ok(())
    }
}
