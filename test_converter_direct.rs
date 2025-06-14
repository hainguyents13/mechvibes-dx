use std::path::Path;
use std::fs;

// Import the converter function directly
use mechvibes_dx::utils::config_converter::convert_v1_to_v2;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let soundpack_dir = r"E:\mechvibes-dx\soundpacks\keyboard\Super Paper Mario Talk";
    let v1_config_path = format!("{}/config.json", soundpack_dir);
    let v2_config_path = format!("{}/config_v2_converted.json", soundpack_dir);
    
    println!("🧪 Testing V1 to V2 converter...");
    println!("Input: {}", v1_config_path);
    println!("Output: {}", v2_config_path);
    
    // Check if V1 config exists
    if !Path::new(&v1_config_path).exists() {
        return Err("V1 config file not found".into());
    }
    
    // Read V1 config to confirm it's V1
    let v1_content = fs::read_to_string(&v1_config_path)?;
    println!("📄 V1 config file size: {} bytes", v1_content.len());
    
    // Run the converter
    println!("🔄 Converting...");
    convert_v1_to_v2(&v1_config_path, &v2_config_path, soundpack_dir)?;
    
    println!("✅ Conversion completed!");
    
    // Check output file
    if Path::new(&v2_config_path).exists() {
        let v2_content = fs::read_to_string(&v2_config_path)?;
        println!("📄 V2 config file size: {} bytes", v2_content.len());
    }
    
    Ok(())
}
