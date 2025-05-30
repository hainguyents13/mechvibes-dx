use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=app.config.json");
    println!("cargo:rerun-if-changed=build.rs");

    // Only generate manifest for release builds
    if env::var("PROFILE").unwrap_or_default() == "release" {
        generate_manifest_for_production();
    }

    // Set git information if available
    if let Ok(output) = Command::new("git").args(&["rev-parse", "HEAD"]).output() {
        if output.status.success() {
            let git_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("cargo:rustc-env=GIT_HASH={}", git_hash);
        }
    }

    if let Ok(output) = Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
    {
        if output.status.success() {
            let git_branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("cargo:rustc-env=GIT_BRANCH={}", git_branch);
        }
    }
}

fn generate_manifest_for_production() {
    println!("🏗️  Generating production manifest...");

    // Create data directory if it doesn't exist
    if !Path::new("data").exists() {
        let _ = fs::create_dir_all("data");
    }

    // Read app.config.json
    let config_content = match fs::read_to_string("app.config.json") {
        Ok(content) => content,
        Err(_) => {
            eprintln!("❌ app.config.json not found! Creating default...");
            create_default_config();
            fs::read_to_string("app.config.json").expect("Failed to read created config")
        }
    };

    // Parse config
    let config: serde_json::Value =
        serde_json::from_str(&config_content).expect("Failed to parse app.config.json"); // Create manifest with build information
    let manifest = serde_json::json!({
        "app": {
            "name": config["app"]["name"],
            "version": config["app"]["version"],
            "description": config["app"]["description"],
            "build_date": chrono::Utc::now().to_rfc3339(),
            "git_commit": env::var("GIT_HASH").ok(),
            "git_branch": env::var("GIT_BRANCH").unwrap_or_else(|_| "main".to_string()),
            "build_type": "release"
        },
        "compatibility": config["compatibility"],
        "paths": config["paths"],
        "metadata": {
            "created_at": chrono::Utc::now().to_rfc3339(),
            "last_updated": chrono::Utc::now().to_rfc3339(),
            "platform": get_target_platform()
        }
    });

    // Write manifest
    let manifest_content =
        serde_json::to_string_pretty(&manifest).expect("Failed to serialize manifest");

    fs::write("data/manifest.json", manifest_content).expect("Failed to write manifest file");

    println!("✅ Production manifest generated");
}

fn create_default_config() {
    let default_config = serde_json::json!({
        "app": {
            "name": "MechvibesDX",
            "version": "0.1.0",
            "description": "Enhanced mechanical keyboard sound simulator"
        },
        "compatibility": {
            "config_version": "1.0",
            "soundpack_version": "1.0",
            "cache_version": "1.0",
            "minimum_app_version": "0.1.0"
        },
        "paths": {
            "config_file": "./data/config.json",
            "soundpack_cache": "./data/soundpacks.json",
            "soundpacks_dir": "./soundpacks",
            "data_dir": "./data"
        }
    });

    let config_content =
        serde_json::to_string_pretty(&default_config).expect("Failed to serialize default config");

    fs::write("app.config.json", config_content).expect("Failed to write default config");
}

fn get_target_platform() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    }
}
