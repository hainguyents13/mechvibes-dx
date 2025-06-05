/// Platform-related utility functions

/// Get the current platform identifier
pub fn get_platform() -> String {
    if cfg!(target_os = "windows") {
        "windows".to_string()
    } else if cfg!(target_os = "macos") {
        "macos".to_string()
    } else if cfg!(target_os = "linux") {
        "linux".to_string()
    } else {
        "unknown".to_string()
    }
}

/// Get build type (debug or release)
pub fn get_build_type() -> String {
    if cfg!(debug_assertions) {
        "debug".to_string()
    } else {
        "release".to_string()
    }
}

/// Get supported architectures for the current platform
pub fn get_supported_architectures() -> Vec<String> {
    vec!["x86_64".to_string()]
}

/// Get minimum OS version requirement
pub fn get_min_os_version() -> String {
    if cfg!(target_os = "windows") {
        "Windows 10".to_string()
    } else if cfg!(target_os = "macos") {
        "macOS 10.14".to_string()
    } else if cfg!(target_os = "linux") {
        "Ubuntu 18.04".to_string()
    } else {
        "Unknown".to_string()
    }
}
