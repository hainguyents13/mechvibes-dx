use cpal::traits::{ DeviceTrait, HostTrait };
use cpal::{ Device, Host };

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub is_default: bool,
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

impl DeviceManager {
    pub fn new() -> Self {
        Self {
            host: cpal::default_host(),
        }
    }

    /// Get all available audio output devices
    pub fn get_output_devices(&self) -> Result<Vec<DeviceInfo>, String> {
        let mut devices = Vec::new();
        let default_device = self.host.default_output_device();
        let default_name = default_device
            .as_ref()
            .and_then(|d| d.name().ok())
            .unwrap_or_else(|| "Unknown".to_string());

        match self.host.output_devices() {
            Ok(device_iter) => {
                for (index, device) in device_iter.enumerate() {
                    if let Ok(name) = device.name() {
                        let is_default =
                            Some(&name) ==
                            default_device
                                .as_ref()
                                .and_then(|d| d.name().ok())
                                .as_ref();

                        devices.push(DeviceInfo {
                            id: format!("output_{}", index),
                            name: name.clone(),
                            is_default,
                        });
                    }
                }
            }
            Err(e) => {
                return Err(format!("Failed to enumerate output devices: {}", e));
            }
        }

        // Ensure we have at least the default device
        if devices.is_empty() {
            devices.push(DeviceInfo {
                id: "output_default".to_string(),
                name: default_name,
                is_default: true,
            });
        }

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

        match self.host.input_devices() {
            Ok(device_iter) => {
                for (index, device) in device_iter.enumerate() {
                    if let Ok(name) = device.name() {
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
