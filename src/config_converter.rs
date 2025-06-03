// Standalone config converter utility
use std::env;
use std::fs;
use std::path::Path;
use serde_json::Value;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 3 {
        eprintln!("Usage: {} <input_config.json> <output_config.json>", args[0]);
        std::process::exit(1);
    }
    
    let input_path = &args[1];
    let output_path = &args[2];
    
    if !Path::new(input_path).exists() {
        eprintln!("Error: Input file '{}' does not exist", input_path);
        std::process::exit(1);
    }
    
    match convert_v1_to_v2(input_path, output_path) {
        Ok(_) => {
            println!("Successfully converted '{}' to '{}'", input_path, output_path);
        }
        Err(e) => {
            eprintln!("Error converting config: {}", e);
            std::process::exit(1);
        }
    }
}

fn convert_v1_to_v2(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Read input file
    let input_content = fs::read_to_string(input_path)?;
    let mut config: Value = serde_json::from_str(&input_content)?;

    // Check if it's already V2 or if it has version field
    if config.get("version").is_some() {
        return Err("Config already has version field - might already be V2".into());
    }

    // Convert V1 to V2
    config["version"] = Value::String("2".to_string());
    
    // Set default author to "N/A" if not present
    if config.get("author").is_none() {
        config["author"] = Value::String("N/A".to_string());
    }

    // Write output file
    let output_content = serde_json::to_string_pretty(&config)?;
    fs::write(output_path, output_content)?;

    Ok(())
}
}
