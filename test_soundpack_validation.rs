// Test script to validate soundpack timing issues
use std::env;

// Include the necessary modules
mod src {
    pub mod libs {
        pub mod audio {
            pub mod soundpack_loader;
        }
    }
    pub mod state {
        pub mod soundpack;
    }
    pub mod utils {
        pub mod path;
        pub mod soundpack_timing_fixer;
    }
}

use src::utils::soundpack_timing_fixer::{validate_soundpack_timing, check_all_soundpacks_timing};

fn main() {
    println!("🔍 Testing Soundpack Timing Validation");
    
    // Test the specific problematic soundpack
    let problematic_soundpack = "custom-sound-pack-1203000000067";
    
    println!("\n📋 Testing specific soundpack: {}", problematic_soundpack);
    match validate_soundpack_timing(problematic_soundpack) {
        Ok(issues) => {
            if issues.is_empty() {
                println!("✅ No timing issues found!");
            } else {
                println!("❌ Found {} timing issues:", issues.len());
                for issue in issues {
                    println!("  {}", issue);
                }
            }
        }
        Err(e) => {
            println!("❌ Error validating soundpack: {}", e);
        }
    }

    // Optionally test all soundpacks
    if env::args().any(|arg| arg == "--all") {
        println!("\n🔍 Checking all soundpacks...");
        match check_all_soundpacks_timing() {
            Ok(all_issues) => {
                println!("📊 Total problematic soundpacks: {}", all_issues.len());
            }
            Err(e) => {
                println!("❌ Error checking all soundpacks: {}", e);
            }
        }
    }
}
