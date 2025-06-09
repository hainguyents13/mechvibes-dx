// Quick test to validate soundpack timing
use std::env;

mod src {
    pub mod utils {
        pub mod soundpack_timing_fixer;
        pub mod path;
    }
    pub mod libs {
        pub mod audio {
            pub mod soundpack_loader;
        }
    }
    pub mod state {
        pub mod soundpack;
    }
}

use src::utils::soundpack_timing_fixer::validate_and_fix_soundpack_timing;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <soundpack_id>", args[0]);
        return;
    }

    let soundpack_id = &args[1];
    println!("🔍 Validating soundpack: {}", soundpack_id);

    match validate_and_fix_soundpack_timing(soundpack_id) {
        Ok(fixed) => {
            if fixed {
                println!("✅ Timing issues were found and fixed!");
            } else {
                println!("✅ No timing issues found.");
            }
        }
        Err(e) => {
            eprintln!("❌ Error: {}", e);
        }
    }
}
