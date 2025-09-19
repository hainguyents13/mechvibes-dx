use std::collections::HashMap;
use std::ptr::null_mut;
use std::ffi::OsString;

#[cfg(windows)]
use std::os::windows::ffi::OsStringExt;

#[cfg(windows)]
use winapi::shared::ntdef::HANDLE;
#[cfg(windows)]
use winapi::shared::minwindef::*;
#[cfg(windows)]
use winapi::um::winuser::*;
#[cfg(windows)]
use winapi::um::setupapi::*;
#[cfg(windows)]
use winapi::um::winreg::*;
#[cfg(windows)]
use winapi::shared::devguid::*;
#[cfg(windows)]
use winapi::um::cfgmgr32::*;

#[derive(Debug, Clone, PartialEq)]
pub struct InputDeviceInfo {
    pub id: String,
    pub name: String,
    pub device_type: InputDeviceType,
    pub vendor_id: u16,
    pub product_id: u16,
    pub is_enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputDeviceType {
    Keyboard,
    Mouse,
    Unknown,
}

pub struct InputDeviceManager {
    devices: HashMap<String, InputDeviceInfo>,
    enabled_keyboards: Vec<String>,
    enabled_mice: Vec<String>,
}

impl InputDeviceManager {
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
            enabled_keyboards: Vec::new(),
            enabled_mice: Vec::new(),
        }
    }

    /// Enumerate all input devices
    pub fn enumerate_devices(&mut self) -> Result<(), String> {
        #[cfg(windows)]
        {
            self.enumerate_windows_devices()
        }
        #[cfg(not(windows))]
        {
            // For non-Windows platforms, we'll use a simplified approach
            // that just returns the default device
            self.add_default_devices();
            Ok(())
        }
    }

    #[cfg(windows)]
    fn enumerate_windows_devices(&mut self) -> Result<(), String> {
        unsafe {
            let mut num_devices = 0u32;

            // First call to get the number of devices
            if
                GetRawInputDeviceList(
                    null_mut(),
                    &mut num_devices,
                    std::mem::size_of::<RAWINPUTDEVICELIST>() as u32
                ) == u32::MAX
            {
                return Err("Failed to get raw input device count".to_string());
            }

            if num_devices == 0 {
                return Ok(());
            }

            // Allocate buffer for device list
            let mut devices =
                vec![RAWINPUTDEVICELIST {
                hDevice: null_mut(),
                dwType: 0,
            }; num_devices as usize];

            // Get the actual device list
            if
                GetRawInputDeviceList(
                    devices.as_mut_ptr(),
                    &mut num_devices,
                    std::mem::size_of::<RAWINPUTDEVICELIST>() as u32
                ) == u32::MAX
            {
                return Err("Failed to get raw input device list".to_string());
            }

            // Process each device
            for device in devices.iter().take(num_devices as usize) {
                if let Ok(device_info) = self.get_device_info(device.hDevice) {
                    // Only add keyboards and mice
                    if
                        matches!(
                            device_info.device_type,
                            InputDeviceType::Keyboard | InputDeviceType::Mouse
                        )
                    {
                        self.devices.insert(device_info.id.clone(), device_info);
                    }
                }
            }
        }
        Ok(())
    }

    #[cfg(windows)]
    unsafe fn get_device_info(&self, device_handle: HANDLE) -> Result<InputDeviceInfo, String> {
        let mut name_size = 0u32;

        // Get device name size
        GetRawInputDeviceInfoW(device_handle, RIDI_DEVICENAME as u32, null_mut(), &mut name_size);

        if name_size == 0 {
            return Err("Failed to get device name size".to_string());
        }

        // Get device name
        let mut name_buffer = vec![0u16; name_size as usize];
        if
            GetRawInputDeviceInfoW(
                device_handle,
                RIDI_DEVICENAME as u32,
                name_buffer.as_mut_ptr() as *mut _,
                &mut name_size
            ) == u32::MAX
        {
            return Err("Failed to get device name".to_string());
        }

        // Convert name to String
        let name = String::from_utf16_lossy(&name_buffer[..(name_size as usize) - 1]);

        // Get device info structure
        let mut info_size = std::mem::size_of::<RID_DEVICE_INFO>() as u32;
        let mut device_info = RID_DEVICE_INFO {
            cbSize: info_size,
            dwType: 0,
            u: std::mem::zeroed(),
        };

        if
            GetRawInputDeviceInfoW(
                device_handle,
                RIDI_DEVICEINFO as u32,
                &mut device_info as *mut _ as *mut _,
                &mut info_size
            ) == u32::MAX
        {
            return Err("Failed to get device info".to_string());
        }

        let (device_type, vendor_id, product_id) = match device_info.dwType {
            RIM_TYPEKEYBOARD => {
                let kbd = unsafe { device_info.u.keyboard() };
                (InputDeviceType::Keyboard, 0, kbd.dwKeyboardMode) // Using dwKeyboardMode as product_id
            }
            RIM_TYPEMOUSE => {
                let mouse = unsafe { device_info.u.mouse() };
                (InputDeviceType::Mouse, 0, mouse.dwId)
            }
            _ => (InputDeviceType::Unknown, 0, 0),
        };

        // Create a unique ID from the device path
        let device_id = format!("{:x}", self.hash_string(&name));

        Ok(InputDeviceInfo {
            id: device_id,
            name: self.extract_device_name(&name),
            device_type,
            vendor_id,
            product_id: product_id as u16,
            is_enabled: true, // Default to enabled
        })
    }

    #[cfg(not(windows))]
    fn add_default_devices(&mut self) {
        // Add default keyboard and mouse for non-Windows platforms
        self.devices.insert("default_keyboard".to_string(), InputDeviceInfo {
            id: "default_keyboard".to_string(),
            name: "Default Keyboard".to_string(),
            device_type: InputDeviceType::Keyboard,
            vendor_id: 0,
            product_id: 0,
            is_enabled: true,
        });

        self.devices.insert("default_mouse".to_string(), InputDeviceInfo {
            id: "default_mouse".to_string(),
            name: "Default Mouse".to_string(),
            device_type: InputDeviceType::Mouse,
            vendor_id: 0,
            product_id: 0,
            is_enabled: true,
        });
    }

    fn hash_string(&self, s: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{ Hash, Hasher };

        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }
    fn extract_device_name(&self, device_path: &str) -> String {
        // Try to get friendly name from Windows Registry first
        if let Ok(friendly_name) = self.get_friendly_device_name(device_path) {
            if !friendly_name.is_empty() && friendly_name != "Unknown Device" {
                return friendly_name;
            }
        }

        // Fallback to extracting info from device path
        if device_path.contains("VID_") && device_path.contains("PID_") {
            let vid_start = device_path.find("VID_").unwrap_or(0) + 4;
            let vid_end = device_path[vid_start..].find('&').unwrap_or(4) + vid_start;
            let vid = &device_path[vid_start..vid_end];

            let pid_start = device_path.find("PID_").unwrap_or(0) + 4;
            let pid_end = device_path[pid_start..].find('&').unwrap_or(4) + pid_start;
            let pid = &device_path[pid_start..pid_end];

            // Try to get manufacturer name from VID
            let manufacturer = self.get_manufacturer_name(vid);
            let product = self.get_product_name(vid, pid);

            if !manufacturer.is_empty() && !product.is_empty() {
                format!("{} {}", manufacturer, product)
            } else if !manufacturer.is_empty() {
                format!("{} Device (VID:{} PID:{})", manufacturer, vid, pid)
            } else {
                format!("Device VID:{} PID:{}", vid, pid)
            }
        } else if device_path.contains("HID") {
            if device_path.to_lowercase().contains("keyboard") {
                "HID Keyboard".to_string()
            } else if device_path.to_lowercase().contains("mouse") {
                "HID Mouse".to_string()
            } else {
                "HID Device".to_string()
            }
        } else {
            "Input Device".to_string()
        }
    }

    #[cfg(windows)]
    fn get_friendly_device_name(&self, device_path: &str) -> Result<String, String> {
        use winapi::um::cfgmgr32::*;
        use std::ffi::CString;

        unsafe {
            // Extract instance ID from device path
            let instance_id = if let Some(start) = device_path.find("\\\\?\\") {
                let path_without_prefix = &device_path[4..]; // Remove "\\?\"
                if let Some(end) = path_without_prefix.find("#{") {
                    path_without_prefix[..end].replace("\\", "\\")
                } else {
                    path_without_prefix.to_string()
                }
            } else {
                device_path.to_string()
            }; // Convert to wide string
            let mut wide_instance_id: Vec<u16> = instance_id
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            let mut dev_node = 0u32;
            let mut buffer = vec![0u16; 256];
            let mut buffer_size = buffer.len() as u32;

            // Locate device node
            if
                CM_Locate_DevNodeW(
                    &mut dev_node,
                    wide_instance_id.as_mut_ptr(),
                    CM_LOCATE_DEVNODE_NORMAL
                ) == CR_SUCCESS
            {
                // Try to get friendly name
                if
                    CM_Get_DevNode_Registry_PropertyW(
                        dev_node,
                        CM_DRP_FRIENDLYNAME,
                        null_mut(),
                        buffer.as_mut_ptr() as *mut _,
                        &mut buffer_size,
                        0
                    ) == CR_SUCCESS
                {
                    let friendly_name = OsString::from_wide(
                        &buffer[..((buffer_size as usize) / 2).saturating_sub(1)]
                    )
                        .to_string_lossy()
                        .to_string();
                    if !friendly_name.is_empty() {
                        return Ok(friendly_name);
                    }
                }

                // Fallback to device description
                buffer_size = buffer.len() as u32;
                if
                    CM_Get_DevNode_Registry_PropertyW(
                        dev_node,
                        CM_DRP_DEVICEDESC,
                        null_mut(),
                        buffer.as_mut_ptr() as *mut _,
                        &mut buffer_size,
                        0
                    ) == CR_SUCCESS
                {
                    let device_desc = OsString::from_wide(
                        &buffer[..((buffer_size as usize) / 2).saturating_sub(1)]
                    )
                        .to_string_lossy()
                        .to_string();
                    if !device_desc.is_empty() {
                        return Ok(device_desc);
                    }
                }
            }
        }

        Err("Could not retrieve device name".to_string())
    }

    #[cfg(not(windows))]
    fn get_friendly_device_name(&self, _device_path: &str) -> Result<String, String> {
        Err("Not supported on non-Windows platforms".to_string())
    }

    fn get_manufacturer_name(&self, vid: &str) -> String {
        // Common USB Vendor IDs
        match vid.to_uppercase().as_str() {
            "046D" => "Logitech".to_string(),
            "1532" => "Razer".to_string(),
            "04D9" => "Holtek".to_string(),
            "04F2" => "Chicony".to_string(),
            "04F3" => "Elan".to_string(),
            "0458" => "KYE Systems".to_string(),
            "045E" => "Microsoft".to_string(),
            "05AC" => "Apple".to_string(),
            "04CA" => "Lite-On".to_string(),
            "0B05" => "ASUSTeK".to_string(),
            "1038" => "SteelSeries".to_string(),
            "04B4" => "Cypress".to_string(),
            "0764" => "Cyber Power".to_string(),
            "1EA7" => "SHARKOON".to_string(),
            "04D8" => "Microchip".to_string(),
            "1B1C" => "Corsair".to_string(),
            "2516" => "Cooler Master".to_string(),
            "0C45" => "Microdia".to_string(),
            "413C" => "Dell".to_string(),
            "17EF" => "Lenovo".to_string(),
            "03F0" => "HP".to_string(),
            "0079" => "DragonRise".to_string(),
            "04E8" => "Samsung".to_string(),
            "1D57" => "Xenta".to_string(),
            "0E8F" => "GreenAsia".to_string(),
            "054C" => "Sony".to_string(),
            "28DE" => "Valve".to_string(),
            "045A" => "PixArt".to_string(),
            "0951" => "Kingston".to_string(),
            "3938" => "MOSART".to_string(),
            "04F9" => "Brother".to_string(),
            "0483" => "STMicroelectronics".to_string(),
            "1267" => "Logic3".to_string(),
            "046A" => "Cherry".to_string(),
            "0C70" => "JMTek".to_string(),
            "0403" => "FTDI".to_string(),
            "1050" => "Yubico".to_string(),
            "04B3" => "IBM".to_string(),
            "05FE" => "Chic Technology".to_string(),
            "0A5C" => "Broadcom".to_string(),
            "8087" => "Intel".to_string(),
            "138A" => "Validity Sensors".to_string(),
            "27C6" => "Goodix".to_string(),
            "0BDA" => "Realtek".to_string(),
            "18D1" => "Google".to_string(),
            "2341" => "Arduino".to_string(),
            "16C0" => "Van Ooijen Technische Informatica".to_string(),
            "1209" => "pid.codes Test".to_string(),
            "2E8A" => "Raspberry Pi".to_string(),
            "239A" => "Adafruit".to_string(),
            "04CC" => "ST-Ericsson".to_string(),
            "0424" => "Microchip Technology".to_string(),
            "1FC9" => "NXP".to_string(),
            _ => "".to_string(),
        }
    }

    fn get_product_name(&self, vid: &str, pid: &str) -> String {
        // Some specific well-known products
        let key = format!("{}:{}", vid.to_uppercase(), pid.to_uppercase());
        match key.as_str() {
            // Logitech devices
            "046D:C52B" => "MX Master".to_string(),
            "046D:C52F" => "MX Master 2S".to_string(),
            "046D:4013" => "MX Master 3".to_string(),
            "046D:C33A" => "G413 Mechanical Gaming Keyboard".to_string(),
            "046D:C342" => "G502 Gaming Mouse".to_string(),
            "046D:C085" => "G203 Gaming Mouse".to_string(),

            // Razer devices
            "1532:0067" => "BlackWidow Ultimate".to_string(),
            "1532:0109" => "DeathAdder Elite".to_string(),
            "1532:0084" => "Mamba Wireless".to_string(),

            // Microsoft devices
            "045E:0750" => "Wired Keyboard 600".to_string(),
            "045E:0040" => "Wheel Mouse Optical".to_string(),
            "045E:0823" => "Classic IntelliMouse".to_string(),

            // Generic fallback
            _ => "".to_string(),
        }
    }

    /// Get all keyboard devices
    pub fn get_keyboards(&self) -> Vec<InputDeviceInfo> {
        self.devices
            .values()
            .filter(|device| device.device_type == InputDeviceType::Keyboard)
            .cloned()
            .collect()
    }

    /// Get all mouse devices
    pub fn get_mice(&self) -> Vec<InputDeviceInfo> {
        self.devices
            .values()
            .filter(|device| device.device_type == InputDeviceType::Mouse)
            .cloned()
            .collect()
    }

    /// Enable/disable a specific device
    pub fn set_device_enabled(&mut self, device_id: &str, enabled: bool) {
        if let Some(device) = self.devices.get_mut(device_id) {
            device.is_enabled = enabled;

            match device.device_type {
                InputDeviceType::Keyboard => {
                    if enabled {
                        if !self.enabled_keyboards.contains(&device_id.to_string()) {
                            self.enabled_keyboards.push(device_id.to_string());
                        }
                    } else {
                        self.enabled_keyboards.retain(|id| id != device_id);
                    }
                }
                InputDeviceType::Mouse => {
                    if enabled {
                        if !self.enabled_mice.contains(&device_id.to_string()) {
                            self.enabled_mice.push(device_id.to_string());
                        }
                    } else {
                        self.enabled_mice.retain(|id| id != device_id);
                    }
                }
                _ => {}
            }
        }
    }

    /// Get list of enabled keyboard device IDs
    pub fn get_enabled_keyboards(&self) -> &[String] {
        &self.enabled_keyboards
    }

    /// Get list of enabled mouse device IDs
    pub fn get_enabled_mice(&self) -> &[String] {
        &self.enabled_mice
    }

    /// Check if a device should generate sound events
    pub fn should_process_device(&self, device_id: &str, device_type: InputDeviceType) -> bool {
        match device_type {
            InputDeviceType::Keyboard => {
                self.enabled_keyboards.is_empty() ||
                    self.enabled_keyboards.contains(&device_id.to_string())
            }
            InputDeviceType::Mouse => {
                self.enabled_mice.is_empty() || self.enabled_mice.contains(&device_id.to_string())
            }
            _ => false,
        }
    }
}

impl Default for InputDeviceManager {
    fn default() -> Self {
        Self::new()
    }
}
